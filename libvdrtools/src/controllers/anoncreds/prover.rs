use std::{
    collections::{HashMap, HashSet},
    ops::DerefMut,
    sync::Arc,
};

use futures::lock::Mutex;
use indy_api_types::{errors::prelude::*, SearchHandle, WalletHandle};
use indy_utils::next_search_handle;
use indy_wallet::{RecordOptions, SearchOptions, WalletRecord, WalletSearch, WalletService};
use log::trace;
use serde_json::Value;
use ursa::cl::{new_nonce, RevocationRegistry, Witness};

use crate::{
    domain::{
        anoncreds::{
            credential::{Credential, CredentialInfo},
            credential_attr_tag_policy::CredentialAttrTagPolicy,
            credential_definition::{
                cred_defs_map_to_cred_defs_v1_map, CredentialDefinition, CredentialDefinitionId,
                CredentialDefinitionV1, CredentialDefinitions,
            },
            credential_for_proof_request::{CredentialsForProofRequest, RequestedCredential},
            credential_offer::CredentialOffer,
            credential_request::{CredentialRequest, CredentialRequestMetadata},
            master_secret::MasterSecret,
            proof_request::{
                NonRevocedInterval, PredicateInfo, ProofRequest, ProofRequestExtraQuery,
            },
            requested_credential::RequestedCredentials,
            revocation_registry_definition::{
                RevocationRegistryDefinition, RevocationRegistryDefinitionV1,
            },
            revocation_registry_delta::{RevocationRegistryDelta, RevocationRegistryDeltaV1},
            revocation_state::{RevocationState, RevocationStates},
            schema::{schemas_map_to_schemas_v1_map, Schemas},
        },
        crypto::did::DidValue,
    },
    services::{AnoncredsHelpers, BlobStorageService, CryptoService, ProverService},
    utils::wql::Query,
};

use super::tails::SDKTailsAccessor;

struct SearchForProofRequest {
    search: WalletSearch,
    interval: Option<NonRevocedInterval>,
    predicate_info: Option<PredicateInfo>,
}

impl SearchForProofRequest {
    fn new(
        search: WalletSearch,
        interval: Option<NonRevocedInterval>,
        predicate_info: Option<PredicateInfo>,
    ) -> Self {
        Self {
            search,
            interval,
            predicate_info,
        }
    }
}

pub struct ProverController {
    prover_service: Arc<ProverService>,
    wallet_service: Arc<WalletService>,
    crypto_service: Arc<CryptoService>,
    blob_storage_service: Arc<BlobStorageService>,
    searches: Mutex<HashMap<SearchHandle, Box<WalletSearch>>>,
    searches_for_proof_requests:
        Mutex<HashMap<SearchHandle, HashMap<String, Arc<Mutex<SearchForProofRequest>>>>>,
}

impl ProverController {
    pub(crate) fn new(
        prover_service: Arc<ProverService>,
        wallet_service: Arc<WalletService>,
        crypto_service: Arc<CryptoService>,
        blob_storage_service: Arc<BlobStorageService>,
    ) -> ProverController {
        ProverController {
            prover_service,
            wallet_service,
            crypto_service,
            blob_storage_service,
            searches: Mutex::new(HashMap::new()),
            searches_for_proof_requests: Mutex::new(HashMap::new()),
        }
    }

    /// Creates a master secret with a given id and stores it in the wallet.
    /// The id must be unique.
    ///
    /// #Params

    /// wallet_handle: wallet handle (created by open_wallet).
    /// master_secret_id: (optional, if not present random one will be generated) new master id
    ///
    /// #Returns
    /// out_master_secret_id: Id of generated master secret
    ///
    /// #Errors
    /// Anoncreds*
    /// Common*
    /// Wallet*
    pub async fn create_master_secret(
        &self,
        wallet_handle: WalletHandle,
        master_secret_id: Option<String>,
    ) -> IndyResult<String> {
        trace!(
            "create_master_secret > wallet_handle {:?} master_secret_id {:?}",
            wallet_handle,
            master_secret_id
        );

        let master_secret_id = master_secret_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        if self
            .wallet_service
            .record_exists::<MasterSecret>(wallet_handle, &master_secret_id)
            .await?
        {
            return Err(err_msg(
                IndyErrorKind::MasterSecretDuplicateName,
                format!("MasterSecret already exists {}", master_secret_id),
            ));
        }

        let master_secret = self.prover_service.new_master_secret()?;

        let master_secret = MasterSecret {
            value: master_secret,
        };

        self.wallet_service
            .add_indy_object(
                wallet_handle,
                &master_secret_id,
                &master_secret,
                &HashMap::new(),
            )
            .await?;

        let res = Ok(master_secret_id);
        trace!("create_master_secret < {:?}", res);
        res
    }

    /// Creates a credential request for the given credential offer.
    ///
    /// The method creates a blinded master secret for a master secret identified by a provided name.
    /// The master secret identified by the name must be already stored in the secure wallet (see prover_create_master_secret)
    /// The blinded master secret is a part of the credential request.
    ///
    /// #Params

    /// wallet_handle: wallet handle (created by open_wallet)
    /// prover_did: a DID of the prover
    /// cred_offer_json: credential offer as a json containing information about the issuer and a credential
    ///     {
    ///         "schema_id": string, - identifier of schema
    ///         "cred_def_id": string, - identifier of credential definition
    ///          ...
    ///         Other fields that contains data structures internal to Ursa.
    ///         These fields should not be parsed and are likely to change in future versions.
    ///     }
    /// cred_def_json: credential definition json related to <cred_def_id> in <cred_offer_json>
    /// master_secret_id: the id of the master secret stored in the wallet
    ///
    /// #Returns
    /// cred_req_json: Credential request json for creation of credential by Issuer
    ///     {
    ///      "prover_did" : string,
    ///      "cred_def_id" : string,
    ///         // Fields below can depend on Cred Def type
    ///      "blinded_ms" : <blinded_master_secret>,
    ///                     (opaque type that contains data structures internal to Ursa.
    ///                      It should not be parsed and are likely to change in future versions).
    ///      "blinded_ms_correctness_proof" : <blinded_ms_correctness_proof>,
    ///                     (opaque type that contains data structures internal to Ursa.
    ///                      It should not be parsed and are likely to change in future versions).
    ///      "nonce": string
    ///    }
    /// cred_req_metadata_json: Credential request metadata json for further processing of received form Issuer credential.
    ///     Credential request metadata contains data structures internal to Ursa.
    ///     Credential request metadata mustn't be shared with Issuer.
    ///
    /// #Errors
    /// Anoncreds*
    /// Common*
    /// Wallet*
    pub async fn create_credential_request(
        &self,
        wallet_handle: WalletHandle,
        prover_did: DidValue,
        cred_offer: CredentialOffer,
        cred_def: CredentialDefinition,
        master_secret_id: String,
    ) -> IndyResult<(String, String)> {
        trace!(
            "create_credential_request > wallet_handle {:?} \
                prover_did {:?} cred_offer {:?} cred_def {:?} \
                master_secret_id: {:?}",
            wallet_handle,
            prover_did,
            cred_offer,
            cred_def,
            master_secret_id
        );

        let cred_def = CredentialDefinitionV1::from(cred_def);

        self.crypto_service.validate_did(&prover_did)?;

        let master_secret: MasterSecret = self
            ._wallet_get_master_secret(wallet_handle, &master_secret_id)
            .await?;

        let (blinded_ms, ms_blinding_data, blinded_ms_correctness_proof) = self
            .prover_service
            .new_credential_request(&cred_def, &master_secret.value, &cred_offer)?;

        let nonce = new_nonce()?;

        let credential_request = CredentialRequest {
            prover_did,
            cred_def_id: cred_offer.cred_def_id.clone(),
            blinded_ms,
            blinded_ms_correctness_proof,
            nonce,
        };

        let credential_request_metadata = CredentialRequestMetadata {
            master_secret_blinding_data: ms_blinding_data,
            nonce: credential_request.nonce.try_clone()?,
            master_secret_name: master_secret_id.to_string(),
        };

        let cred_req_json = serde_json::to_string(&credential_request).to_indy(
            IndyErrorKind::InvalidState,
            "Cannot serialize CredentialRequest",
        )?;

        let cred_req_metadata_json = serde_json::to_string(&credential_request_metadata).to_indy(
            IndyErrorKind::InvalidState,
            "Cannot serialize CredentialRequestMetadata",
        )?;

        let res = Ok((cred_req_json, cred_req_metadata_json));
        trace!("create_credential_request < {:?}", res);
        res
    }

    /// Set credential attribute tagging policy.
    /// Writes a non-secret record marking attributes to tag, and optionally
    /// updates tags on existing credentials on the credential definition to match.
    ///
    /// EXPERIMENTAL
    ///
    /// The following tags are always present on write:
    ///     {
    ///         "schema_id": <credential schema id>,
    ///         "schema_issuer_did": <credential schema issuer did>,
    ///         "schema_name": <credential schema name>,
    ///         "schema_version": <credential schema version>,
    ///         "issuer_did": <credential issuer did>,
    ///         "cred_def_id": <credential definition id>,
    ///         "rev_reg_id": <credential revocation registry id>, // "None" as string if not present
    ///     }
    ///
    /// The policy sets the following tags for each attribute it marks taggable, written to subsequent
    /// credentials and (optionally) all existing credentials on the credential definition:
    ///     {
    ///         "attr::<attribute name>::marker": "1",
    ///         "attr::<attribute name>::value": <attribute raw value>,
    ///     }
    ///
    /// #Params

    /// wallet_handle: wallet handle (created by open_wallet).
    /// cred_def_id: credential definition id
    /// tag_attrs_json: JSON array with names of attributes to tag by policy, or null for all
    /// retroactive: boolean, whether to apply policy to existing credentials on credential definition identifier
    ///
    /// #Errors
    /// Anoncreds*
    /// Common*
    /// Wallet*
    pub async fn set_credential_attr_tag_policy(
        &self,
        wallet_handle: WalletHandle,
        cred_def_id: CredentialDefinitionId,
        catpol: Option<CredentialAttrTagPolicy>,
        retroactive: bool,
    ) -> IndyResult<()> {
        trace!(
            "set_credential_attr_tag_policy > wallet_handle {:?} \
                cred_def_id {:?} catpol {:?} retroactive {:?}",
            wallet_handle,
            cred_def_id,
            catpol,
            retroactive
        );

        match catpol {
            Some(ref pol) => {
                self.wallet_service
                    .upsert_indy_object(wallet_handle, &cred_def_id.0, pol)
                    .await?;
            }
            None => {
                if self
                    .wallet_service
                    .record_exists::<CredentialAttrTagPolicy>(wallet_handle, &cred_def_id.0)
                    .await?
                {
                    self.wallet_service
                        .delete_indy_record::<CredentialAttrTagPolicy>(
                            wallet_handle,
                            &cred_def_id.0,
                        )
                        .await?;
                }
            }
        };

        // Cascade whether we updated policy or not: could be a retroactive cred attr tags reset to existing policy
        if retroactive {
            let query_json = format!(r#"{{"cred_def_id": "{}"}}"#, cred_def_id.0);

            let mut credentials_search = self
                .wallet_service
                .search_indy_records::<Credential>(
                    wallet_handle,
                    query_json.as_str(),
                    &SearchOptions::id_value(),
                )
                .await?;

            while let Some(credential_record) = credentials_search.fetch_next_record().await? {
                let (_, credential) = self._get_credential(&credential_record)?;

                let cred_tags = self
                    .prover_service
                    .build_credential_tags(&credential, catpol.as_ref())?;

                self.wallet_service
                    .update_record_tags(
                        wallet_handle,
                        self.wallet_service.add_prefix("Credential").as_str(),
                        credential_record.get_id(),
                        &cred_tags,
                    )
                    .await?;
            }
        }

        let res = Ok(());
        trace!("set_credential_attr_tag_policy < {:?}", res);
        res
    }

    /// Get credential attribute tagging policy by credential definition id.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params

    /// wallet_handle: wallet handle (created by open_wallet).
    /// cred_def_id: credential definition id
    ///
    /// #Returns
    /// JSON array with all attributes that current policy marks taggable;
    /// null for default policy (tag all credential attributes).
    ///
    /// #Errors
    /// Anoncreds*
    /// Common*
    /// Wallet*
    pub async fn get_credential_attr_tag_policy(
        &self,
        wallet_handle: WalletHandle,
        cred_def_id: CredentialDefinitionId,
    ) -> IndyResult<String> {
        trace!(
            "get_credential_attr_tag_policy > wallet_handle {:?} \
                cred_def_id {:?}",
            wallet_handle,
            cred_def_id
        );

        let catpol = self
            ._get_credential_attr_tag_policy(wallet_handle, &cred_def_id)
            .await?;

        let res = Ok(catpol);
        trace!("get_credential_attr_tag_policy < {:?}", res);
        res
    }

    /// Check credential provided by Issuer for the given credential request,
    /// updates the credential by a master secret and stores in a secure wallet.
    ///
    /// To support efficient and flexible search the following tags will be created for stored credential:
    ///     {
    ///         "schema_id": <credential schema id>,
    ///         "schema_issuer_did": <credential schema issuer did>,
    ///         "schema_name": <credential schema name>,
    ///         "schema_version": <credential schema version>,
    ///         "issuer_did": <credential issuer did>,
    ///         "cred_def_id": <credential definition id>,
    ///         "rev_reg_id": <credential revocation registry id>, // "None" as string if not present
    ///         // for every attribute in <credential values> that credential attribute tagging policy marks taggable
    ///         "attr::<attribute name>::marker": "1",
    ///         "attr::<attribute name>::value": <attribute raw value>,
    ///     }
    ///
    /// #Params

    /// wallet_handle: wallet handle (created by open_wallet).
    /// cred_id: (optional, default is a random one) identifier by which credential will be stored in the wallet
    /// cred_req_metadata_json: a credential request metadata created by indy_prover_create_credential_req
    /// cred_json: credential json received from issuer
    ///     {
    ///         "schema_id": string, - identifier of schema
    ///         "cred_def_id": string, - identifier of credential definition
    ///         "rev_reg_def_id", Optional<string>, - identifier of revocation registry
    ///         "values": - credential values
    ///             {
    ///                 "attr1" : {"raw": "value1", "encoded": "value1_as_int" },
    ///                 "attr2" : {"raw": "value1", "encoded": "value1_as_int" }
    ///             }
    ///         // Fields below can depend on Cred Def type
    ///         Other fields that contains data structures internal to Ursa.
    ///         These fields should not be parsed and are likely to change in future versions.
    ///     }
    /// cred_def_json: credential definition json related to <cred_def_id> in <cred_json>
    /// rev_reg_def_json: revocation registry definition json related to <rev_reg_def_id> in <cred_json>
    ///
    /// #Returns
    /// out_cred_id: identifier by which credential is stored in the wallet
    ///
    /// #Errors
    /// Anoncreds*
    /// Common*
    /// Wallet*
    pub async fn store_credential(
        &self,
        wallet_handle: WalletHandle,
        cred_id: Option<String>,
        cred_req_metadata: CredentialRequestMetadata,
        mut credential: Credential,
        cred_def: CredentialDefinition,
        rev_reg_def: Option<RevocationRegistryDefinition>,
    ) -> IndyResult<String> {
        trace!(
            "store_credential > wallet_handle {:?} \
                cred_id {:?} cred_req_metadata {:?} \
                credential {:?} cred_def {:?} \
                rev_reg_def {:?}",
            wallet_handle,
            cred_id,
            cred_req_metadata,
            credential,
            cred_def,
            rev_reg_def
        );

        let cred_def = CredentialDefinitionV1::from(cred_def);
        let rev_reg_def = rev_reg_def.map(RevocationRegistryDefinitionV1::from);

        let master_secret: MasterSecret = self
            ._wallet_get_master_secret(wallet_handle, &cred_req_metadata.master_secret_name)
            .await?;

        self.prover_service.process_credential(
            &mut credential,
            &cred_req_metadata,
            &master_secret.value,
            &cred_def,
            rev_reg_def.as_ref(),
        )?;

        credential.rev_reg = None;
        credential.witness = None;

        let out_cred_id = cred_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let catpol_json = self
            ._get_credential_attr_tag_policy(wallet_handle, &credential.cred_def_id)
            .await?;

        let catpol: Option<CredentialAttrTagPolicy> = if catpol_json.ne("null") {
            Some(serde_json::from_str(catpol_json.as_str()).to_indy(
                IndyErrorKind::InvalidState,
                "Cannot deserialize CredentialAttrTagPolicy",
            )?)
        } else {
            None
        };

        let cred_tags = self
            .prover_service
            .build_credential_tags(&credential, catpol.as_ref())?;

        self.wallet_service
            .add_indy_object(wallet_handle, &out_cred_id, &credential, &cred_tags)
            .await?;

        let res = Ok(out_cred_id);
        trace!("store_credential < {:?}", res);
        res
    }

    /// Gets human readable credentials according to the filter.
    /// If filter is NULL, then all credentials are returned.
    /// Credentials can be filtered by Issuer, credential_def and/or Schema.
    ///
    /// #Params
    /// wallet_handle: wallet handle (created by open_wallet).
    /// filter_json: filter for credentials
    ///        {
    ///            "schema_id": string, (Optional)
    ///            "schema_issuer_did": string, (Optional)
    ///            "schema_name": string, (Optional)
    ///            "schema_version": string, (Optional)
    ///            "issuer_did": string, (Optional)
    ///            "cred_def_id": string, (Optional)
    ///        }
    ///
    /// #Returns
    /// credentials json
    ///     [{
    ///         "referent": string, - id of credential in the wallet
    ///         "attrs": {"key1":"raw_value1", "key2":"raw_value2"}, - credential attributes
    ///         "schema_id": string, - identifier of schema
    ///         "cred_def_id": string, - identifier of credential definition
    ///         "rev_reg_id": Optional<string>, - identifier of revocation registry definition
    ///         "cred_rev_id": Optional<string> - identifier of credential in the revocation registry definition
    ///     }]
    ///
    /// #Errors
    /// Anoncreds*
    /// Common*
    /// Wallet*
    #[no_mangle]
    pub async fn get_credentials(
        &self,
        wallet_handle: WalletHandle,
        filter_json: Option<String>,
    ) -> IndyResult<String> {
        trace!(
            "get_credentials > wallet_handle {:?} filter_json {:?}",
            wallet_handle,
            filter_json
        );

        let filter_json = filter_json.as_deref().unwrap_or("{}");
        let mut credentials_info: Vec<CredentialInfo> = Vec::new();

        let mut credentials_search = self
            .wallet_service
            .search_indy_records::<Credential>(
                wallet_handle,
                filter_json,
                &SearchOptions::id_value(),
            )
            .await?;

        while let Some(credential_record) = credentials_search.fetch_next_record().await? {
            let (referent, credential) = self._get_credential(&credential_record)?;
            credentials_info.push(self._get_credential_info(&referent, credential))
        }

        let credentials_info_json = serde_json::to_string(&credentials_info).to_indy(
            IndyErrorKind::InvalidState,
            "Cannot serialize list of CredentialInfo",
        )?;

        let res = Ok(credentials_info_json);
        trace!("get_credentials < {:?}", res);
        res
    }

    /// Gets human readable credential by the given id.
    ///
    /// #Params
    /// wallet_handle: wallet handle (created by open_wallet).
    /// cred_id: Identifier by which requested credential is stored in the wallet
    ///
    /// #Returns
    /// credential json:
    ///     {
    ///         "referent": string, - id of credential in the wallet
    ///         "attrs": {"key1":"raw_value1", "key2":"raw_value2"}, - credential attributes
    ///         "schema_id": string, - identifier of schema
    ///         "cred_def_id": string, - identifier of credential definition
    ///         "rev_reg_id": Optional<string>, - identifier of revocation registry definition
    ///         "cred_rev_id": Optional<string> - identifier of credential in the revocation registry definition
    ///     }
    ///
    /// #Errors
    /// Anoncreds*
    /// Common*
    /// Wallet*
    pub async fn get_credential(
        &self,
        wallet_handle: WalletHandle,
        cred_id: String,
    ) -> IndyResult<String> {
        trace!(
            "get_credentials > wallet_handle {:?} cred_id {:?}",
            wallet_handle,
            cred_id
        );

        let credential: Credential = self
            .wallet_service
            .get_indy_object(wallet_handle, &cred_id, &RecordOptions::id_value())
            .await?;

        let credential_info = self._get_credential_info(&cred_id, credential);

        let credential_info_json = serde_json::to_string(&credential_info).to_indy(
            IndyErrorKind::InvalidState,
            "Cannot serialize CredentialInfo",
        )?;

        let res = Ok(credential_info_json);
        trace!("get_credential < {:?}", res);
        res
    }

    /// Search for credentials stored in wallet.
    /// Credentials can be filtered by tags created during saving of credential.
    ///
    /// Instead of immediately returning of fetched credentials
    /// this call returns search_handle that can be used later
    /// to fetch records by small batches (with indy_prover_fetch_credentials).
    ///
    /// #Params
    /// wallet_handle: wallet handle (created by open_wallet).
    /// query_json: Wql query filter for credentials searching based on tags.
    ///     where query: indy-sdk/docs/design/011-wallet-query-language/README.md
    ///
    /// #Returns
    /// search_handle: Search handle that can be used later to fetch records by small batches (with indy_prover_fetch_credentials)
    /// total_count: Total count of records
    ///
    /// #Errors
    /// Anoncreds*
    /// Common*
    /// Wallet*
    pub async fn search_credentials(
        &self,
        wallet_handle: WalletHandle,
        query_json: Option<String>,
    ) -> IndyResult<(SearchHandle, usize)> {
        trace!(
            "search_credentials > wallet_handle {:?} query_json {:?}",
            wallet_handle,
            query_json
        );

        let credentials_search = self
            .wallet_service
            .search_indy_records::<Credential>(
                wallet_handle,
                query_json.as_deref().unwrap_or("{}"),
                &SearchOptions::id_value(),
            )
            .await?;

        let total_count = credentials_search.get_total_count()?.unwrap_or(0);

        let handle: SearchHandle = next_search_handle();

        self.searches
            .lock()
            .await
            .insert(handle, Box::new(credentials_search));

        let res = (handle, total_count);
        trace!("search_credentials < {:?}", res);
        Ok(res)
    }

    /// Fetch next credentials for search.
    ///
    /// #Params
    /// search_handle: Search handle (created by indy_prover_search_credentials)
    /// count: Count of credentials to fetch
    ///
    /// #Returns
    /// credentials_json: List of human readable credentials:
    ///     [{
    ///         "referent": string, - id of credential in the wallet
    ///         "attrs": {"key1":"raw_value1", "key2":"raw_value2"}, - credential attributes
    ///         "schema_id": string, - identifier of schema
    ///         "cred_def_id": string, - identifier of credential definition
    ///         "rev_reg_id": Optional<string>, - identifier of revocation registry definition
    ///         "cred_rev_id": Optional<string> - identifier of credential in the revocation registry definition
    ///     }]
    /// NOTE: The list of length less than the requested count means credentials search iterator is completed.
    ///
    /// #Errors
    /// Anoncreds*
    /// Common*
    /// Wallet*
    pub async fn fetch_credentials(
        &self,
        search_handle: SearchHandle,
        count: usize,
    ) -> IndyResult<String> {
        trace!(
            "fetch_credentials > search_handle {:?} count {:?}",
            search_handle,
            count
        );

        let mut searches = self.searches.lock().await;

        let search = searches.get_mut(&search_handle).ok_or_else(|| {
            err_msg(
                IndyErrorKind::InvalidWalletHandle,
                "Unknown CredentialsSearch handle",
            )
        })?;

        let mut credentials_info: Vec<CredentialInfo> = Vec::new();

        for _ in 0..count {
            match search.fetch_next_record().await? {
                Some(credential_record) => {
                    let (referent, credential) = self._get_credential(&credential_record)?;
                    credentials_info.push(self._get_credential_info(&referent, credential))
                }
                None => break,
            }
        }

        let credentials_info_json = serde_json::to_string(&credentials_info).to_indy(
            IndyErrorKind::InvalidState,
            "Cannot serialize list of CredentialInfo",
        )?;

        let res = Ok(credentials_info_json);
        trace!("fetch_credentials < {:?}", res);
        res
    }

    /// Close credentials search (make search handle invalid)
    ///
    /// #Params
    /// search_handle: Search handle (created by indy_prover_search_credentials)
    ///
    /// #Errors
    /// Anoncreds*
    /// Common*
    /// Wallet*
    pub async fn close_credentials_search(&self, search_handle: SearchHandle) -> IndyResult<()> {
        trace!(
            "close_credentials_search > search_handle {:?}",
            search_handle
        );

        self.searches
            .lock()
            .await
            .remove(&search_handle)
            .ok_or_else(|| {
                err_msg(
                    IndyErrorKind::InvalidWalletHandle,
                    "Unknown CredentialsSearch handle",
                )
            })?;

        let res = Ok(());
        trace!("close_credentials_search < {:?}", res);
        res
    }

    /// Gets human readable credentials matching the given proof request.
    ///
    /// NOTE: This method is deprecated because immediately returns all fetched credentials.
    /// Use <indy_prover_search_credentials_for_proof_req> to fetch records by small batches.
    ///
    /// #Params
    /// wallet_handle: wallet handle (created by open_wallet).
    /// proof_request_json: proof request json
    ///     {
    ///         "name": string,
    ///         "version": string,
    ///         "nonce": string, - a decimal number represented as a string (use `indy_generate_nonce` function to generate 80-bit number)
    ///         "requested_attributes": { // set of requested attributes
    ///              "<attr_referent>": <attr_info>, // see below
    ///              ...,
    ///         },
    ///         "requested_predicates": { // set of requested predicates
    ///              "<predicate_referent>": <predicate_info>, // see below
    ///              ...,
    ///          },
    ///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
    ///                        // If specified prover must proof non-revocation
    ///                        // for date in this interval for each attribute
    ///                        // (applies to every attribute and predicate but can be overridden on attribute level),
    ///         "ver": Optional<str>  - proof request version:
    ///             - omit or "1.0" to use unqualified identifiers for restrictions
    ///             - "2.0" to use fully qualified identifiers for restrictions
    ///     }
    ///
    /// where
    /// attr_referent: Proof-request local identifier of requested attribute
    /// attr_info: Describes requested attribute
    ///     {
    ///         "name": Optional<string>, // attribute name, (case insensitive and ignore spaces)
    ///         "names": Optional<[string, string]>, // attribute names, (case insensitive and ignore spaces)
    ///                                              // NOTE: should either be "name" or "names", not both and not none of them.
    ///                                              // Use "names" to specify several attributes that have to match a single credential.
    ///         "restrictions": Optional<filter_json>, // see below
    ///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
    ///                        // If specified prover must proof non-revocation
    ///                        // for date in this interval this attribute
    ///                        // (overrides proof level interval)
    ///     }
    /// predicate_referent: Proof-request local identifier of requested attribute predicate
    /// predicate_info: Describes requested attribute predicate
    ///     {
    ///         "name": attribute name, (case insensitive and ignore spaces)
    ///         "p_type": predicate type (">=", ">", "<=", "<")
    ///         "p_value": int predicate value
    ///         "restrictions": Optional<filter_json>, // see below
    ///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
    ///                        // If specified prover must proof non-revocation
    ///                        // for date in this interval this attribute
    ///                        // (overrides proof level interval)
    ///     }
    /// non_revoc_interval: Defines non-revocation interval
    ///     {
    ///         "from": Optional<int>, // timestamp of interval beginning
    ///         "to": Optional<int>, // timestamp of interval ending
    ///     }
    ///  filter_json:
    ///     {
    ///        "schema_id": string, (Optional)
    ///        "schema_issuer_did": string, (Optional)
    ///        "schema_name": string, (Optional)
    ///        "schema_version": string, (Optional)
    ///        "issuer_did": string, (Optional)
    ///        "cred_def_id": string, (Optional)
    ///     }
    ///
    /// #Returns
    /// credentials_json: json with credentials for the given proof request.
    ///     {
    ///         "attrs": {
    ///             "<attr_referent>": [{ cred_info: <credential_info>, interval: Optional<non_revoc_interval> }],
    ///             ...,
    ///         },
    ///         "predicates": {
    ///             "requested_predicates": [{ cred_info: <credential_info>, timestamp: Optional<integer> }, { cred_info: <credential_2_info>, timestamp: Optional<integer> }],
    ///             "requested_predicate_2_referent": [{ cred_info: <credential_2_info>, timestamp: Optional<integer> }]
    ///         }
    ///     }, where <credential_info> is
    ///     {
    ///         "referent": string, - id of credential in the wallet
    ///         "attrs": {"key1":"raw_value1", "key2":"raw_value2"}, - credential attributes
    ///         "schema_id": string, - identifier of schema
    ///         "cred_def_id": string, - identifier of credential definition
    ///         "rev_reg_id": Optional<string>, - identifier of revocation registry definition
    ///         "cred_rev_id": Optional<string> - identifier of credential in the revocation registry definition
    ///     }
    ///
    /// #Errors
    /// Anoncreds*
    /// Common*
    /// Wallet*
    #[deprecated(
        since = "1.6.1",
        note = "Please use indy_prover_search_credentials_for_proof_req instead!"
    )]
    #[no_mangle]
    pub async fn get_credentials_for_proof_req(
        &self,
        wallet_handle: WalletHandle,
        proof_request: ProofRequest,
    ) -> IndyResult<String> {
        trace!(
            "get_credentials_for_proof_req > wallet_handle {:?} proof_request {:?}",
            wallet_handle,
            proof_request
        );

        let proof_req = proof_request.value();
        let proof_req_version = proof_request.version();

        let mut credentials_for_proof_request: CredentialsForProofRequest =
            CredentialsForProofRequest::default();

        for (attr_id, requested_attr) in proof_req.requested_attributes.iter() {
            let query = self.prover_service.process_proof_request_restrictions(
                &proof_req_version,
                &requested_attr.name,
                &requested_attr.names,
                &attr_id,
                &requested_attr.restrictions,
                &None,
            )?;

            let interval = AnoncredsHelpers::get_non_revoc_interval(
                &proof_req.non_revoked,
                &requested_attr.non_revoked,
            );

            let credentials_for_attribute = self
                ._query_requested_credentials(wallet_handle, &query, None, &interval)
                .await?;

            credentials_for_proof_request
                .attrs
                .insert(attr_id.to_string(), credentials_for_attribute);
        }

        for (predicate_id, requested_predicate) in proof_req.requested_predicates.iter() {
            let query = self.prover_service.process_proof_request_restrictions(
                &proof_req_version,
                &Some(requested_predicate.name.clone()),
                &None,
                &predicate_id,
                &requested_predicate.restrictions,
                &None,
            )?;

            let interval = AnoncredsHelpers::get_non_revoc_interval(
                &proof_req.non_revoked,
                &requested_predicate.non_revoked,
            );

            let credentials_for_predicate = self
                ._query_requested_credentials(
                    wallet_handle,
                    &query,
                    Some(&requested_predicate),
                    &interval,
                )
                .await?;

            credentials_for_proof_request
                .predicates
                .insert(predicate_id.to_string(), credentials_for_predicate);
        }

        let credentials_for_proof_request_json =
            serde_json::to_string(&credentials_for_proof_request).to_indy(
                IndyErrorKind::InvalidState,
                "Cannot serialize CredentialsForProofRequest",
            )?;

        let res = Ok(credentials_for_proof_request_json);
        trace!("get_credentials_for_proof_req < {:?}", res);
        res
    }

    /// Search for credentials matching the given proof request.
    ///
    /// Instead of immediately returning of fetched credentials
    /// this call returns search_handle that can be used later
    /// to fetch records by small batches (with indy_prover_fetch_credentials_for_proof_req).
    ///
    /// #Params
    /// wallet_handle: wallet handle (created by open_wallet).
    /// proof_request_json: proof request json
    ///     {
    ///         "name": string,
    ///         "version": string,
    ///         "nonce": string, - a decimal number represented as a string (use `indy_generate_nonce` function to generate 80-bit number)
    ///         "requested_attributes": { // set of requested attributes
    ///              "<attr_referent>": <attr_info>, // see below
    ///              ...,
    ///         },
    ///         "requested_predicates": { // set of requested predicates
    ///              "<predicate_referent>": <predicate_info>, // see below
    ///              ...,
    ///          },
    ///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
    ///                        // If specified prover must proof non-revocation
    ///                        // for date in this interval for each attribute
    ///                        // (applies to every attribute and predicate but can be overridden on attribute level)
    ///                        // (can be overridden on attribute level)
    ///         "ver": Optional<str>  - proof request version:
    ///             - omit or "1.0" to use unqualified identifiers for restrictions
    ///             - "2.0" to use fully qualified identifiers for restrictions
    ///     }
    ///
    /// where
    /// attr_info: Describes requested attribute
    ///     {
    ///         "name": Optional<string>, // attribute name, (case insensitive and ignore spaces)
    ///         "names": Optional<[string, string]>, // attribute names, (case insensitive and ignore spaces)
    ///                                              // NOTE: should either be "name" or "names", not both and not none of them.
    ///                                              // Use "names" to specify several attributes that have to match a single credential.
    ///         "restrictions": Optional<wql query>, // see below
    ///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
    ///                        // If specified prover must proof non-revocation
    ///                        // for date in this interval this attribute
    ///                        // (overrides proof level interval)
    ///     }
    /// predicate_referent: Proof-request local identifier of requested attribute predicate
    /// predicate_info: Describes requested attribute predicate
    ///     {
    ///         "name": attribute name, (case insensitive and ignore spaces)
    ///         "p_type": predicate type (">=", ">", "<=", "<")
    ///         "p_value": predicate value
    ///         "restrictions": Optional<wql query>, // see below
    ///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
    ///                        // If specified prover must proof non-revocation
    ///                        // for date in this interval this attribute
    ///                        // (overrides proof level interval)
    ///     }
    /// non_revoc_interval: Defines non-revocation interval
    ///     {
    ///         "from": Optional<int>, // timestamp of interval beginning
    ///         "to": Optional<int>, // timestamp of interval ending
    ///     }
    /// extra_query_json:(Optional) List of extra queries that will be applied to correspondent attribute/predicate:
    ///     {
    ///         "<attr_referent>": <wql query>,
    ///         "<predicate_referent>": <wql query>,
    ///     }
    /// where wql query: indy-sdk/docs/design/011-wallet-query-language/README.md
    ///     The list of allowed keys that can be combine into complex queries.
    ///         "schema_id": <credential schema id>,
    ///         "schema_issuer_did": <credential schema issuer did>,
    ///         "schema_name": <credential schema name>,
    ///         "schema_version": <credential schema version>,
    ///         "issuer_did": <credential issuer did>,
    ///         "cred_def_id": <credential definition id>,
    ///         "rev_reg_id": <credential revocation registry id>, // "None" as string if not present
    ///         // the following keys can be used for every `attribute name` in credential.
    ///         "attr::<attribute name>::marker": "1", - to filter based on existence of a specific attribute
    ///         "attr::<attribute name>::value": <attribute raw value>, - to filter based on value of a specific attribute
    ///
    ///
    /// #Returns
    /// search_handle: Search handle that can be used later to fetch records by small batches (with indy_prover_fetch_credentials_for_proof_req)
    ///
    /// #Errors
    /// Anoncreds*
    /// Common*
    /// Wallet*
    pub async fn search_credentials_for_proof_req(
        &self,
        wallet_handle: WalletHandle,
        proof_request: ProofRequest,
        extra_query: Option<ProofRequestExtraQuery>,
    ) -> IndyResult<SearchHandle> {
        trace!(
            "search_credentials_for_proof_req > wallet_handle {:?} \
                proof_request {:?} extra_query {:?}",
            wallet_handle,
            proof_request,
            extra_query
        );

        let proof_req = proof_request.value();
        let version = proof_request.version();

        let mut credentials_for_proof_request_search =
            HashMap::<String, Arc<Mutex<SearchForProofRequest>>>::new();

        for (attr_id, requested_attr) in proof_req.requested_attributes.iter() {
            let query = self.prover_service.process_proof_request_restrictions(
                &version,
                &requested_attr.name,
                &requested_attr.names,
                &attr_id,
                &requested_attr.restrictions,
                &extra_query.as_ref(),
            )?;

            let credentials_search = self
                .wallet_service
                .search_indy_records::<Credential>(
                    wallet_handle,
                    &query.to_string(),
                    &SearchOptions::id_value(),
                )
                .await?;

            let interval = AnoncredsHelpers::get_non_revoc_interval(
                &proof_req.non_revoked,
                &requested_attr.non_revoked,
            );

            credentials_for_proof_request_search.insert(
                attr_id.to_string(),
                Arc::new(Mutex::new(SearchForProofRequest::new(
                    credentials_search,
                    interval,
                    None,
                ))),
            );
        }

        for (predicate_id, requested_predicate) in proof_req.requested_predicates.iter() {
            let query = self.prover_service.process_proof_request_restrictions(
                &version,
                &Some(requested_predicate.name.clone()),
                &None,
                &predicate_id,
                &requested_predicate.restrictions,
                &extra_query.as_ref(),
            )?;

            let credentials_search = self
                .wallet_service
                .search_indy_records::<Credential>(
                    wallet_handle,
                    &query.to_string(),
                    &SearchOptions::id_value(),
                )
                .await?;

            let interval = AnoncredsHelpers::get_non_revoc_interval(
                &proof_req.non_revoked,
                &requested_predicate.non_revoked,
            );

            credentials_for_proof_request_search.insert(
                predicate_id.to_string(),
                Arc::new(Mutex::new(SearchForProofRequest::new(
                    credentials_search,
                    interval,
                    Some(requested_predicate.clone()),
                ))),
            );
        }

        let search_handle = next_search_handle();

        self.searches_for_proof_requests
            .lock()
            .await
            .insert(search_handle, credentials_for_proof_request_search);

        let res = Ok(search_handle);
        trace!("search_credentials_for_proof_req < {:?}", search_handle);
        res
    }

    /// Fetch next credentials for the requested item using proof request search
    /// handle (created by indy_prover_search_credentials_for_proof_req).
    ///
    /// #Params
    /// search_handle: Search handle (created by indy_prover_search_credentials_for_proof_req)
    /// item_referent: Referent of attribute/predicate in the proof request
    /// count: Count of credentials to fetch
    ///
    /// #Returns
    /// credentials_json: List of credentials for the given proof request.
    ///     [{
    ///         cred_info: <credential_info>,
    ///         interval: Optional<non_revoc_interval>
    ///     }]
    /// where
    /// credential_info:
    ///     {
    ///         "referent": string, - id of credential in the wallet
    ///         "attrs": {"key1":"raw_value1", "key2":"raw_value2"}, - credential attributes
    ///         "schema_id": string, - identifier of schema
    ///         "cred_def_id": string, - identifier of credential definition
    ///         "rev_reg_id": Optional<string>, - identifier of revocation registry definition
    ///         "cred_rev_id": Optional<string> - identifier of credential in the revocation registry definition
    ///     }
    /// non_revoc_interval:
    ///     {
    ///         "from": Optional<int>, // timestamp of interval beginning
    ///         "to": Optional<int>, // timestamp of interval ending
    ///     }
    /// NOTE: The list of length less than the requested count means that search iterator
    /// correspondent to the requested <item_referent> is completed.
    ///
    /// #Errors
    /// Anoncreds*
    /// Common*
    /// Wallet*
    pub async fn fetch_credential_for_proof_request(
        &self,
        search_handle: SearchHandle,
        item_referent: String,
        count: usize,
    ) -> IndyResult<String> {
        trace!(
            "fetch_credential_for_proof_request > search_handle {:?} \
                item_referent {:?} count {:?}",
            search_handle,
            item_referent,
            count
        );

        let search_mut = {
            let mut searches = self.searches_for_proof_requests.lock().await;

            searches
                .get_mut(&search_handle)
                .ok_or_else(|| {
                    err_msg(
                        IndyErrorKind::InvalidWalletHandle,
                        "Unknown CredentialsSearch",
                    )
                })?
                .get(&item_referent)
                .ok_or_else(|| {
                    err_msg(
                        IndyErrorKind::InvalidWalletHandle,
                        "Unknown item referent for CredentialsSearch handle",
                    )
                })?
                .clone()
        };

        let mut search_lock = search_mut.lock().await;
        let search: &mut SearchForProofRequest = search_lock.deref_mut();

        let requested_credentials: Vec<RequestedCredential> = self
            ._get_requested_credentials(
                &mut search.search,
                search.predicate_info.as_ref(),
                &search.interval,
                Some(count),
            )
            .await?;

        let requested_credentials_json = serde_json::to_string(&requested_credentials).to_indy(
            IndyErrorKind::InvalidState,
            "Cannot serialize list of RequestedCredential",
        )?;

        let res = Ok(requested_credentials_json);
        trace!("fetch_credential_for_proof_request < {:?}", res);
        res
    }

    /// Close credentials search for proof request (make search handle invalid)
    ///
    /// #Params
    /// search_handle: Search handle (created by indy_prover_search_credentials_for_proof_req)
    ///
    /// #Errors
    /// Anoncreds*
    /// Common*
    /// Wallet*
    pub async fn close_credentials_search_for_proof_req(
        &self,
        search_handle: SearchHandle,
    ) -> IndyResult<()> {
        trace!(
            "close_credentials_search_for_proof_req > search_handle {:?}",
            search_handle
        );

        self.searches_for_proof_requests
            .lock()
            .await
            .remove(&search_handle)
            .ok_or_else(|| {
                err_msg(
                    IndyErrorKind::InvalidWalletHandle,
                    "Unknown CredentialsSearch handle",
                )
            })?;

        let res = Ok(());
        trace!("close_credentials_search_for_proof_req < {:?}", res);
        res
    }

    /// Deletes credential by given id.
    ///
    /// #Params
    /// wallet_handle: wallet handle (created by open_wallet).
    /// cred_id: Identifier by which requested credential is stored in the wallet
    ///
    /// #Errors
    /// Anoncreds*
    /// Common*
    /// Wallet*
    pub async fn delete_credential(
        &self,
        wallet_handle: WalletHandle,
        cred_id: String,
    ) -> IndyResult<()> {
        trace!(
            "delete_credential > wallet_handle {:?} cred_id {:?}",
            wallet_handle,
            cred_id
        );

        if !self
            .wallet_service
            .record_exists::<Credential>(wallet_handle, &cred_id)
            .await?
        {
            return Err(err_msg(
                IndyErrorKind::WalletItemNotFound,
                "Credential not found",
            ));
        }

        self.wallet_service
            .delete_indy_record::<Credential>(wallet_handle, &cred_id)
            .await?;

        let res = Ok(());
        trace!("delete_credential < {:?}", res);
        res
    }

    /// Creates a proof according to the given proof request
    /// Either a corresponding credential with optionally revealed attributes or self-attested attribute must be provided
    /// for each requested attribute (see indy_prover_get_credentials_for_pool_req).
    /// A proof request may request multiple credentials from different schemas and different issuers.
    /// All required schemas, public keys and revocation registries must be provided.
    /// The proof request also contains nonce.
    /// The proof contains either proof or self-attested attribute value for each requested attribute.
    ///
    /// #Params
    /// wallet_handle: wallet handle (created by open_wallet).

    /// proof_request_json: proof request json
    ///     {
    ///         "name": string,
    ///         "version": string,
    ///         "nonce": string, - a decimal number represented as a string (use `indy_generate_nonce` function to generate 80-bit number)
    ///         "requested_attributes": { // set of requested attributes
    ///              "<attr_referent>": <attr_info>, // see below
    ///              ...,
    ///         },
    ///         "requested_predicates": { // set of requested predicates
    ///              "<predicate_referent>": <predicate_info>, // see below
    ///              ...,
    ///          },
    ///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
    ///                        // If specified prover must proof non-revocation
    ///                        // for date in this interval for each attribute
    ///                        // (applies to every attribute and predicate but can be overridden on attribute level)
    ///                        // (can be overridden on attribute level)
    ///         "ver": Optional<str>  - proof request version:
    ///             - omit or "1.0" to use unqualified identifiers for restrictions
    ///             - "2.0" to use fully qualified identifiers for restrictions
    ///     }
    /// requested_credentials_json: either a credential or self-attested attribute for each requested attribute
    ///     {
    ///         "self_attested_attributes": {
    ///             "self_attested_attribute_referent": string
    ///         },
    ///         "requested_attributes": {
    ///             "requested_attribute_referent_1": {"cred_id": string, "timestamp": Optional<number>, revealed: <bool> }},
    ///             "requested_attribute_referent_2": {"cred_id": string, "timestamp": Optional<number>, revealed: <bool> }}
    ///         },
    ///         "requested_predicates": {
    ///             "requested_predicates_referent_1": {"cred_id": string, "timestamp": Optional<number> }},
    ///         }
    ///     }
    /// master_secret_id: the id of the master secret stored in the wallet
    /// schemas_json: all schemas participating in the proof request
    ///     {
    ///         <schema1_id>: <schema1>,
    ///         <schema2_id>: <schema2>,
    ///         <schema3_id>: <schema3>,
    ///     }
    /// credential_defs_json: all credential definitions participating in the proof request
    ///     {
    ///         "cred_def1_id": <credential_def1>,
    ///         "cred_def2_id": <credential_def2>,
    ///         "cred_def3_id": <credential_def3>,
    ///     }
    /// rev_states_json: all revocation states participating in the proof request
    ///     {
    ///         "rev_reg_def1_id or credential_1_id": {
    ///             "timestamp1": <rev_state1>,
    ///             "timestamp2": <rev_state2>,
    ///         },
    ///         "rev_reg_def2_id or credential_1_id"": {
    ///             "timestamp3": <rev_state3>
    ///         },
    ///         "rev_reg_def3_id or credential_1_id"": {
    ///             "timestamp4": <rev_state4>
    ///         },
    ///     }
    /// Note: use credential_id instead rev_reg_id in case proving several credentials from the same revocation registry.
    ///
    /// where
    /// attr_referent: Proof-request local identifier of requested attribute
    /// attr_info: Describes requested attribute
    ///     {
    ///         "name": Optional<string>, // attribute name, (case insensitive and ignore spaces)
    ///         "names": Optional<[string, string]>, // attribute names, (case insensitive and ignore spaces)
    ///                                              // NOTE: should either be "name" or "names", not both and not none of them.
    ///                                              // Use "names" to specify several attributes that have to match a single credential.
    ///         "restrictions": Optional<wql query>, // see below
    ///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
    ///                        // If specified prover must proof non-revocation
    ///                        // for date in this interval this attribute
    ///                        // (overrides proof level interval)
    ///     }
    /// predicate_referent: Proof-request local identifier of requested attribute predicate
    /// predicate_info: Describes requested attribute predicate
    ///     {
    ///         "name": attribute name, (case insensitive and ignore spaces)
    ///         "p_type": predicate type (">=", ">", "<=", "<")
    ///         "p_value": predicate value
    ///         "restrictions": Optional<wql query>, // see below
    ///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
    ///                        // If specified prover must proof non-revocation
    ///                        // for date in this interval this attribute
    ///                        // (overrides proof level interval)
    ///     }
    /// non_revoc_interval: Defines non-revocation interval
    ///     {
    ///         "from": Optional<int>, // timestamp of interval beginning
    ///         "to": Optional<int>, // timestamp of interval ending
    ///     }
    /// where wql query: indy-sdk/docs/design/011-wallet-query-language/README.md
    ///     The list of allowed keys that can be combine into complex queries.
    ///         "schema_id": <credential schema id>,
    ///         "schema_issuer_did": <credential schema issuer did>,
    ///         "schema_name": <credential schema name>,
    ///         "schema_version": <credential schema version>,
    ///         "issuer_did": <credential issuer did>,
    ///         "cred_def_id": <credential definition id>,
    ///         "rev_reg_id": <credential revocation registry id>, // "None" as string if not present
    ///         // the following keys can be used for every `attribute name` in credential.
    ///         "attr::<attribute name>::marker": "1", - to filter based on existence of a specific attribute
    ///         "attr::<attribute name>::value": <attribute raw value>, - to filter based on value of a specific attribute
    ///
    /// #Returns
    /// Proof json
    /// For each requested attribute either a proof (with optionally revealed attribute value) or
    /// self-attested attribute value is provided.
    /// Each proof is associated with a credential and corresponding schema_id, cred_def_id, rev_reg_id and timestamp.
    /// There is also aggregated proof part common for all credential proofs.
    ///     {
    ///         "requested_proof": {
    ///             "revealed_attrs": {
    ///                 "requested_attr1_id": {sub_proof_index: number, raw: string, encoded: string},
    ///                 "requested_attr4_id": {sub_proof_index: number: string, encoded: string},
    ///             },
    ///             "revealed_attr_groups": {
    ///                 "requested_attr5_id": {
    ///                     "sub_proof_index": number,
    ///                     "values": {
    ///                         "attribute_name": {
    ///                             "raw": string,
    ///                             "encoded": string
    ///                         }
    ///                     },
    ///                 }
    ///             },
    ///             "unrevealed_attrs": {
    ///                 "requested_attr3_id": {sub_proof_index: number}
    ///             },
    ///             "self_attested_attrs": {
    ///                 "requested_attr2_id": self_attested_value,
    ///             },
    ///             "predicates": {
    ///                 "requested_predicate_1_referent": {sub_proof_index: int},
    ///                 "requested_predicate_2_referent": {sub_proof_index: int},
    ///             }
    ///         }
    ///         "proof": {
    ///             "proofs": [ <credential_proof>, <credential_proof>, <credential_proof> ],
    ///             "aggregated_proof": <aggregated_proof>
    ///         } (opaque type that contains data structures internal to Ursa.
    ///           It should not be parsed and are likely to change in future versions).
    ///         "identifiers": [{schema_id, cred_def_id, Optional<rev_reg_id>, Optional<timestamp>}]
    ///     }
    ///
    /// #Errors
    /// Anoncreds*
    /// Common*
    /// Wallet*
    pub async fn create_proof(
        &self,
        wallet_handle: WalletHandle,
        proof_req: ProofRequest,
        requested_credentials: RequestedCredentials,
        master_secret_id: String,
        schemas: Schemas,
        cred_defs: CredentialDefinitions,
        rev_states: RevocationStates,
    ) -> IndyResult<String> {
        trace!(
            "create_proof > wallet_handle {:?} \
                proof_req {:?} requested_credentials {:?} \
                master_secret_id {:?} schemas {:?} \
                cred_defs {:?} rev_states {:?}",
            wallet_handle,
            proof_req,
            requested_credentials,
            master_secret_id,
            schemas,
            cred_defs,
            rev_states
        );

        let schemas = schemas_map_to_schemas_v1_map(schemas);
        let cred_defs = cred_defs_map_to_cred_defs_v1_map(cred_defs);

        let master_secret = self
            ._wallet_get_master_secret(wallet_handle, &master_secret_id)
            .await?;

        let cred_refs_for_attrs = requested_credentials
            .requested_attributes
            .values()
            .map(|requested_attr| requested_attr.cred_id.clone())
            .collect::<HashSet<String>>();

        let cred_refs_for_predicates = requested_credentials
            .requested_predicates
            .values()
            .map(|requested_predicate| requested_predicate.cred_id.clone())
            .collect::<HashSet<String>>();

        let cred_referents = cred_refs_for_attrs
            .union(&cred_refs_for_predicates)
            .cloned()
            .collect::<Vec<String>>();

        let mut credentials: HashMap<String, Credential> =
            HashMap::with_capacity(cred_referents.len());

        for cred_referent in cred_referents.into_iter() {
            let credential: Credential = self
                .wallet_service
                .get_indy_object(wallet_handle, &cred_referent, &RecordOptions::id_value())
                .await?;
            credentials.insert(cred_referent, credential);
        }

        let proof = self.prover_service.create_proof(
            &credentials,
            &proof_req,
            &requested_credentials,
            &master_secret.value,
            &schemas,
            &cred_defs,
            &rev_states,
        )?;

        let proof_json = serde_json::to_string(&proof)
            .to_indy(IndyErrorKind::InvalidState, "Cannot serialize FullProof")?;

        let res = Ok(proof_json);
        trace!("create_proof <{:?}", res);
        res
    }

    /// Create revocation state for a credential that corresponds to a particular time.
    ///
    /// Note that revocation delta must cover the whole registry existence time.
    /// You can use `from`: `0` and `to`: `needed_time` as parameters for building request to get correct revocation delta.
    ///
    /// The resulting revocation state and provided timestamp can be saved and reused later with applying a new
    /// revocation delta with `indy_update_revocation_state` function.
    /// This new delta should be received with parameters: `from`: `timestamp` and `to`: `needed_time`.
    ///
    /// #Params

    /// blob_storage_reader_handle: configuration of blob storage reader handle that will allow to read revocation tails (returned by `indy_open_blob_storage_reader`)
    /// rev_reg_def_json: revocation registry definition json related to `rev_reg_id` in a credential
    /// rev_reg_delta_json: revocation registry delta which covers the whole registry existence time
    /// timestamp: time represented as a total number of seconds from Unix Epoch.
    /// cred_rev_id: user credential revocation id in revocation registry (match to `cred_rev_id` in a credential)
    ///
    /// #Returns
    /// revocation state json:
    ///     {
    ///         "rev_reg": <revocation registry>,
    ///         "witness": <witness>,  (opaque type that contains data structures internal to Ursa.
    ///                                 It should not be parsed and are likely to change in future versions).
    ///         "timestamp" : integer
    ///     }
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Anoncreds*
    pub async fn create_revocation_state(
        &self,
        blob_storage_reader_handle: i32,
        revoc_reg_def: RevocationRegistryDefinition,
        rev_reg_delta: RevocationRegistryDelta,
        timestamp: u64,
        cred_rev_id: String,
    ) -> IndyResult<String> {
        trace!(
            "create_revocation_state > blob_storage_reader_handle {:?} \
                revoc_reg_def {:?} rev_reg_delta {:?} timestamp {:?} \
                cred_rev_id {:?}",
            blob_storage_reader_handle,
            revoc_reg_def,
            rev_reg_delta,
            timestamp,
            cred_rev_id
        );

        let revoc_reg_def = RevocationRegistryDefinitionV1::from(revoc_reg_def);
        let rev_idx = AnoncredsHelpers::parse_cred_rev_id(&cred_rev_id)?;

        let sdk_tails_accessor = SDKTailsAccessor::new(
            self.blob_storage_service.clone(),
            blob_storage_reader_handle,
            &revoc_reg_def,
        )
        .await?;

        let rev_reg_delta = RevocationRegistryDeltaV1::from(rev_reg_delta);

        let witness = Witness::new(
            rev_idx,
            revoc_reg_def.value.max_cred_num,
            revoc_reg_def.value.issuance_type.to_bool(),
            &rev_reg_delta.value,
            &sdk_tails_accessor,
        )?;

        let revocation_state = RevocationState {
            witness,
            rev_reg: RevocationRegistry::from(rev_reg_delta.value),
            timestamp,
        };

        let revocation_state_json = serde_json::to_string(&revocation_state).to_indy(
            IndyErrorKind::InvalidState,
            "Cannot serialize RevocationState",
        )?;

        let res = Ok(revocation_state_json);
        trace!("create_revocation_state < {:?}", res);
        res
    }

    /// Create a new revocation state for a credential based on a revocation state created before.
    /// Note that provided revocation delta must cover the registry gap from based state creation until the specified time
    /// (this new delta should be received with parameters: `from`: `state_timestamp` and `to`: `needed_time`).
    ///
    /// This function reduces the calculation time.
    ///
    /// The resulting revocation state and provided timestamp can be saved and reused later by applying a new revocation delta again.
    ///
    /// #Params

    /// blob_storage_reader_handle: configuration of blob storage reader handle that will allow to read revocation tails (returned by `indy_open_blob_storage_reader`)
    /// rev_state_json: revocation registry state json
    /// rev_reg_def_json: revocation registry definition json related to `rev_reg_id` in a credential
    /// rev_reg_delta_json: revocation registry definition delta which covers the gap form original `rev_state_json` creation till the requested timestamp
    /// timestamp: time represented as a total number of seconds from Unix Epoch
    /// cred_rev_id: user credential revocation id in revocation registry (match to `cred_rev_id` in a credential)
    ///
    /// #Returns
    /// revocation state json:
    ///     {
    ///         "rev_reg": <revocation registry>,
    ///         "witness": <witness>,  (opaque type that contains data structures internal to Ursa.
    ///                                 It should not be parsed and are likely to change in future versions).
    ///         "timestamp" : integer
    ///     }
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Anoncreds*
    pub async fn update_revocation_state(
        &self,
        blob_storage_reader_handle: i32,
        mut rev_state: RevocationState,
        rev_reg_def: RevocationRegistryDefinition,
        rev_reg_delta: RevocationRegistryDelta,
        timestamp: u64,
        cred_rev_id: String,
    ) -> IndyResult<String> {
        trace!(
            "update_revocation_state > blob_storage_reader_handle {:?} \
                rev_state {:?} rev_reg_def {:?} rev_reg_delta {:?} \
                timestamp {:?} cred_rev_id {:?}",
            blob_storage_reader_handle,
            rev_state,
            rev_reg_def,
            rev_reg_delta,
            timestamp,
            cred_rev_id
        );

        let revocation_registry_definition = RevocationRegistryDefinitionV1::from(rev_reg_def);
        let rev_reg_delta = RevocationRegistryDeltaV1::from(rev_reg_delta);
        let rev_idx = AnoncredsHelpers::parse_cred_rev_id(&cred_rev_id)?;

        let sdk_tails_accessor = SDKTailsAccessor::new(
            self.blob_storage_service.clone(),
            blob_storage_reader_handle,
            &revocation_registry_definition,
        )
        .await?;

        rev_state.witness.update(
            rev_idx,
            revocation_registry_definition.value.max_cred_num,
            &rev_reg_delta.value,
            &sdk_tails_accessor,
        )?;

        rev_state.rev_reg = RevocationRegistry::from(rev_reg_delta.value);
        rev_state.timestamp = timestamp;

        let rev_state_json = serde_json::to_string(&rev_state).to_indy(
            IndyErrorKind::InvalidState,
            "Cannot serialize RevocationState",
        )?;

        let res = Ok(rev_state_json);
        trace!("update_revocation_state < {:?}", res);
        res
    }

    fn _get_credential_info(&self, referent: &str, credential: Credential) -> CredentialInfo {
        let credential_values: HashMap<String, String> = credential
            .values
            .0
            .into_iter()
            .map(|(attr, values)| (attr, values.raw))
            .collect();

        CredentialInfo {
            referent: referent.to_string(),
            attrs: credential_values,
            schema_id: credential.schema_id,
            cred_def_id: credential.cred_def_id,
            rev_reg_id: credential.rev_reg_id,
            cred_rev_id: credential
                .signature
                .extract_index()
                .map(|idx| idx.to_string()),
        }
    }

    fn _get_credential(&self, record: &WalletRecord) -> IndyResult<(String, Credential)> {
        let referent = record.get_id();

        let value = record.get_value().ok_or_else(|| {
            err_msg(
                IndyErrorKind::InvalidState,
                "Credential not found for id: {}",
            )
        })?;

        let credential: Credential = serde_json::from_str(value)
            .to_indy(IndyErrorKind::InvalidState, "Cannot deserialize Credential")?;

        Ok((referent.to_string(), credential))
    }

    async fn _query_requested_credentials(
        &self,
        wallet_handle: WalletHandle,
        query_json: &Query,
        predicate_info: Option<&PredicateInfo>,
        interval: &Option<NonRevocedInterval>,
    ) -> IndyResult<Vec<RequestedCredential>> {
        trace!(
            "_query_requested_credentials > wallet_handle {:?} \
                query_json {:?} predicate_info {:?}",
            wallet_handle,
            query_json,
            predicate_info
        );

        let mut credentials_search = self
            .wallet_service
            .search_indy_records::<Credential>(
                wallet_handle,
                &query_json.to_string(),
                &SearchOptions::id_value(),
            )
            .await?;

        let credentials = self
            ._get_requested_credentials(&mut credentials_search, predicate_info, interval, None)
            .await?;

        let res = Ok(credentials);
        trace!("_query_requested_credentials < {:?}", res);
        res
    }

    async fn _get_requested_credentials(
        &self,
        credentials_search: &mut WalletSearch,
        predicate_info: Option<&PredicateInfo>,
        interval: &Option<NonRevocedInterval>,
        max_count: Option<usize>,
    ) -> IndyResult<Vec<RequestedCredential>> {
        let mut credentials: Vec<RequestedCredential> = Vec::new();

        if let Some(0) = max_count {
            return Ok(vec![]);
        }

        while let Some(credential_record) = credentials_search.fetch_next_record().await? {
            let (referent, credential) = self._get_credential(&credential_record)?;

            if let Some(predicate) = predicate_info {
                let values = self
                    .prover_service
                    .get_credential_values_for_attribute(&credential.values.0, &predicate.name)
                    .ok_or_else(|| {
                        err_msg(IndyErrorKind::InvalidState, "Credential values not found")
                    })?;

                let satisfy = self
                    .prover_service
                    .attribute_satisfy_predicate(predicate, &values.encoded)?;
                if !satisfy {
                    continue;
                }
            }

            credentials.push(RequestedCredential {
                cred_info: self._get_credential_info(&referent, credential),
                interval: interval.clone(),
            });

            if let Some(mut count) = max_count {
                count -= 1;
                if count == 0 {
                    break;
                }
            }
        }

        Ok(credentials)
    }

    async fn _wallet_get_master_secret(
        &self,
        wallet_handle: WalletHandle,
        key: &str,
    ) -> IndyResult<MasterSecret> {
        self.wallet_service
            .get_indy_object(wallet_handle, &key, &RecordOptions::id_value())
            .await
    }

    async fn _get_credential_attr_tag_policy(
        &self,
        wallet_handle: WalletHandle,
        cred_def_id: &CredentialDefinitionId,
    ) -> IndyResult<String> {
        let catpol = self
            .wallet_service
            .get_indy_opt_object::<CredentialAttrTagPolicy>(
                wallet_handle,
                &cred_def_id.0,
                &RecordOptions::id_value(),
            )
            .await?
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .to_indy(
                IndyErrorKind::InvalidState,
                "Cannot serialize CredentialAttrTagPolicy",
            )?
            .unwrap_or_else(|| Value::Null.to_string());

        Ok(catpol)
    }
}
