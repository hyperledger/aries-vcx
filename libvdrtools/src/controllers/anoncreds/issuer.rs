use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

// use async_std::task::spawn_blocking;
use indy_api_types::{domain::wallet::Tags, errors::prelude::*, WalletHandle};
use indy_wallet::{RecordOptions, WalletService};
use ursa::cl::{
    new_nonce, CredentialKeyCorrectnessProof, CredentialPrivateKey,
    RevocationRegistryDelta as CryptoRevocationRegistryDelta, Witness,
};

use super::tails::{store_tails_from_generator, SDKTailsAccessor};
pub use crate::{
    domain::{
        anoncreds::{
            credential::{Credential, CredentialValues},
            credential_definition::{
                CredentialDefinition, CredentialDefinitionConfig, CredentialDefinitionCorrectnessProof,
                CredentialDefinitionData, CredentialDefinitionId, CredentialDefinitionPrivateKey,
                CredentialDefinitionV1, SignatureType, TemporaryCredentialDefinition,
            },
            credential_offer::CredentialOffer,
            credential_request::CredentialRequest,
            revocation_registry::{RevocationRegistry, RevocationRegistryV1},
            revocation_registry_definition::{
                IssuanceType, RegistryType, RevocationRegistryConfig, RevocationRegistryDefinition,
                RevocationRegistryDefinitionPrivate, RevocationRegistryDefinitionV1, RevocationRegistryDefinitionValue,
                RevocationRegistryId, RevocationRegistryInfo,
            },
            revocation_registry_delta::{RevocationRegistryDelta, RevocationRegistryDeltaV1},
            schema::{AttributeNames, Schema, SchemaId, SchemaV1},
        },
        crypto::did::DidValue,
    },
    services::{AnoncredsHelpers, BlobStorageService, CryptoService, IssuerService},
};

pub struct IssuerController {
    pub issuer_service: Arc<IssuerService>,
    pub blob_storage_service: Arc<BlobStorageService>,
    pub wallet_service: Arc<WalletService>,
    pub crypto_service: Arc<CryptoService>,
}

impl IssuerController {
    pub fn new(
        issuer_service: Arc<IssuerService>,
        blob_storage_service: Arc<BlobStorageService>,
        wallet_service: Arc<WalletService>,
        crypto_service: Arc<CryptoService>,
    ) -> IssuerController {
        IssuerController {
            issuer_service,
            blob_storage_service,
            wallet_service,
            crypto_service,
        }
    }

    /*
    These functions wrap the Ursa algorithm as documented in this paper:
    https://github.com/hyperledger/ursa/blob/master/libursa/docs/AnonCred.pdf

    And is documented in this HIPE:
    https://github.com/hyperledger/indy-hipe/blob/c761c583b1e01c1e9d3ceda2b03b35336fdc8cc1/text/anoncreds-protocol/README.md
    */

    /// Create credential schema entity that describes credential attributes list and allows
    /// credentials interoperability.
    ///
    /// Schema is public and intended to be shared with all anoncreds workflow actors usually by
    /// publishing SCHEMA transaction to Indy distributed ledger.
    ///
    /// It is IMPORTANT for current version POST Schema in Ledger and after that GET it from Ledger
    /// with correct seq_no to save compatibility with Ledger.
    /// After that can call indy_issuer_create_and_store_credential_def to build corresponding
    /// Credential Definition.
    ///
    /// #Params

    /// issuer_did: DID of schema issuer
    /// name: a name the schema
    /// version: a version of the schema
    /// attrs: a list of schema attributes descriptions (the number of attributes should be less or
    /// equal than 125)     `["attr1", "attr2"]`
    ///
    /// #Returns
    /// schema_id: identifier of created schema
    /// schema_json: schema as json:
    /// {
    ///     id: identifier of schema
    ///     attrNames: array of attribute name strings
    ///     name: schema's name string
    ///     version: schema's version string,
    ///     ver: version of the Schema json
    /// }
    ///
    /// #Errors
    /// Common*
    /// Anoncreds*
    pub fn create_schema(
        &self,
        issuer_did: DidValue,
        name: String,
        version: String,
        attrs: AttributeNames,
    ) -> IndyResult<(String, String)> {
        trace!(
            "create_schema > issuer_did {:?} name {:?} version {:?} attrs {:?}",
            issuer_did,
            name,
            version,
            attrs
        );

        self.crypto_service.validate_did(&issuer_did)?;

        let schema_id = SchemaId::new(&issuer_did, &name, &version)?;

        let schema = Schema::SchemaV1(SchemaV1 {
            id: schema_id.clone(),
            name,
            version,
            attr_names: attrs,
            seq_no: None,
        });

        let schema_json =
            serde_json::to_string(&schema).to_indy(IndyErrorKind::InvalidState, "Cannot serialize Schema")?;

        let res = Ok((schema_id.0, schema_json));
        trace!("create_schema < {:?}", res);
        res
    }

    /// Create credential definition entity that encapsulates credentials issuer DID, credential
    /// schema, secrets used for signing credentials and secrets used for credentials
    /// revocation.
    ///
    /// Credential definition entity contains private and public parts. Private part will be stored
    /// in the wallet. Public part will be returned as json intended to be shared with all
    /// anoncreds workflow actors usually by publishing CRED_DEF transaction to Indy distributed
    /// ledger.
    ///
    /// It is IMPORTANT for current version GET Schema from Ledger with correct seq_no to save
    /// compatibility with Ledger.
    ///
    /// Note: Use combination of `indy_issuer_rotate_credential_def_start` and
    /// `indy_issuer_rotate_credential_def_apply` functions to generate new keys for an existing
    /// credential definition.
    ///
    /// #Params

    /// wallet_handle: wallet handle (created by open_wallet).
    /// issuer_did: a DID of the issuer
    /// schema_json: credential schema as a json: {
    ///     id: identifier of schema
    ///     attrNames: array of attribute name strings
    ///     name: schema's name string
    ///     version: schema's version string,
    ///     seqNo: (Optional) schema's sequence number on the ledger,
    ///     ver: version of the Schema json
    /// }
    /// tag: any string that allows to distinguish between credential definitions for the same
    /// issuer and schema signature_type: credential definition type (optional, 'CL' by default)
    /// that defines credentials signature and revocation math. Supported signature types:
    /// - 'CL': Camenisch-Lysyanskaya credential signature type that is implemented according to the
    ///   algorithm in this paper: https://github.com/hyperledger/ursa/blob/master/libursa/docs/AnonCred.pdf
    ///   And is documented in this HIPE: https://github.com/hyperledger/indy-hipe/blob/c761c583b1e01c1e9d3ceda2b03b35336fdc8cc1/text/anoncreds-protocol/README.md
    /// config_json: (optional) type-specific configuration of credential definition as json:
    /// - 'CL': { "support_revocation" - bool (optional, default false) whether to request
    ///   non-revocation credential }
    ///
    /// #Returns
    /// cred_def_id: identifier of created credential definition
    /// cred_def_json: public part of created credential definition
    /// {
    ///     id: string - identifier of credential definition
    ///     schemaId: string - identifier of stored in ledger schema
    ///     type: string - type of the credential definition. CL is the only supported type now.
    ///     tag: string - allows to distinct between credential definitions for the same issuer and
    /// schema     value: Dictionary with Credential Definition's data is depended on the
    /// signature type: {         primary: primary credential public key,
    ///         Optional<revocation>: revocation credential public key
    ///     },
    ///     ver: Version of the CredDef json
    /// }
    ///
    /// Note: `primary` and `revocation` fields of credential definition are complex opaque types
    /// that contain data structures internal to Ursa. They should not be parsed and are likely
    /// to change in future versions.
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Anoncreds*
    pub async fn create_and_store_credential_definition(
        &self,
        wallet_handle: WalletHandle,
        issuer_did: DidValue,
        schema: Schema,
        tag: String,
        type_: Option<String>,
        config: Option<CredentialDefinitionConfig>,
    ) -> IndyResult<(String, String)> {
        trace!(
            "create_and_store_credential_definition > wallet_handle {:?} issuer_did {:?} schema {:?} tag {:?} type_ \
             {:?}, config {:?}",
            wallet_handle,
            issuer_did,
            schema,
            tag,
            type_,
            config
        );

        let mut schema = SchemaV1::from(schema);

        match (issuer_did.get_method(), schema.id.get_method()) {
            (None, Some(_)) => {
                return Err(IndyError::from_msg(
                    IndyErrorKind::InvalidStructure,
                    "You can't use unqualified Did with fully qualified Schema",
                ));
            }
            (Some(prefix_), None) => {
                schema.id = schema.id.qualify(&prefix_)?;
            }
            _ => {}
        };

        let cred_def_config = config.unwrap_or_default();

        let signature_type = if let Some(type_) = type_ {
            serde_json::from_str::<SignatureType>(&format!("\"{}\"", type_))
                .to_indy(IndyErrorKind::InvalidStructure, "Invalid Signature Type format")?
        } else {
            SignatureType::CL
        };

        let schema_id = schema
            .seq_no
            .map(|n| SchemaId(n.to_string()))
            .unwrap_or_else(|| schema.id.clone());

        let cred_def_id = CredentialDefinitionId::new(&issuer_did, &schema_id, signature_type.to_str(), &tag)?;

        let cred_def = self
            .wallet_service
            .get_indy_record_value::<CredentialDefinition>(wallet_handle, &cred_def_id.0, &RecordOptions::id_value())
            .await;

        if let Ok(cred_def) = cred_def {
            let res = Ok((cred_def_id.0, cred_def));

            trace!("create_and_store_credential_definition < already exists {:?}", res);

            return res;
        }

        let tag = tag.to_string();
        let attr_names = schema.attr_names.clone();

        let (credential_definition_value, cred_priv_key, cred_key_correctness_proof) = self
            ._create_credential_definition(&attr_names, cred_def_config.support_revocation)
            .await?;

        let cred_def = CredentialDefinition::CredentialDefinitionV1(CredentialDefinitionV1 {
            id: cred_def_id.clone(),
            schema_id: schema_id.clone(),
            signature_type,
            tag,
            value: credential_definition_value,
        });

        let cred_def_priv_key = CredentialDefinitionPrivateKey { value: cred_priv_key };

        let cred_def_correctness_proof = CredentialDefinitionCorrectnessProof {
            value: cred_key_correctness_proof,
        };

        let schema_ = Schema::SchemaV1(schema.clone());

        let cred_def_json = self
            .wallet_service
            .add_indy_object(wallet_handle, &cred_def_id.0, &cred_def, &HashMap::new())
            .await?;

        self.wallet_service
            .add_indy_object(wallet_handle, &cred_def_id.0, &cred_def_priv_key, &HashMap::new())
            .await?;

        self.wallet_service
            .add_indy_object(
                wallet_handle,
                &cred_def_id.0,
                &cred_def_correctness_proof,
                &HashMap::new(),
            )
            .await?;

        let _ = self
            .wallet_service
            .add_indy_object(wallet_handle, &schema_id.0, &schema_, &HashMap::new())
            .await
            .ok();

        let schema_id = schema.id.clone();

        self._wallet_set_schema_id(wallet_handle, &cred_def_id.0, &schema_id)
            .await?; // TODO: FIXME delete temporary storing of schema id

        let res = Ok((cred_def_id.0, cred_def_json));
        trace!("create_and_store_credential_definition < {:?}", res);
        res
    }

    async fn _create_credential_definition(
        &self,
        attr_names: &AttributeNames,
        support_revocation: bool,
    ) -> IndyResult<(
        CredentialDefinitionData,
        CredentialPrivateKey,
        CredentialKeyCorrectnessProof,
    )> {
        // let attr_names = attr_names.clone();

        IssuerService::new_credential_definition(attr_names, support_revocation)
        // let res = spawn_blocking(move || {
        //     IssuerService::new_credential_definition(&attr_names, support_revocation)
        // })
        // .await?;

        // Ok(res)
    }

    /// Generate temporary credential definitional keys for an existing one (owned by the caller of
    /// the library).
    ///
    /// Use `indy_issuer_rotate_credential_def_apply` function to set generated temporary keys as
    /// the main.
    ///
    /// WARNING: Rotating the credential definitional keys will result in making all credentials
    /// issued under the previous keys unverifiable.
    ///
    /// #Params

    /// wallet_handle: wallet handle (created by open_wallet).
    /// cred_def_id: an identifier of created credential definition stored in the wallet
    /// config_json: (optional) type-specific configuration of credential definition as json:
    /// - 'CL': { "support_revocation" - bool (optional, default false) whether to request
    ///   non-revocation credential }
    ///
    /// #Returns
    /// cred_def_json: public part of temporary created credential definition
    /// {
    ///     id: string - identifier of credential definition
    ///     schemaId: string - identifier of stored in ledger schema
    ///     type: string - type of the credential definition. CL is the only supported type now.
    ///     tag: string - allows to distinct between credential definitions for the same issuer and
    /// schema     value: Dictionary with Credential Definition's data is depended on the
    /// signature type: {         primary: primary credential public key,
    ///         Optional<revocation>: revocation credential public key
    ///     }, - only this field differs from the original credential definition
    ///     ver: Version of the CredDef json
    /// }
    ///
    /// Note: `primary` and `revocation` fields of credential definition are complex opaque types
    /// that contain data structures internal to Ursa. They should not be parsed and are likely
    /// to change in future versions.
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Anoncreds*
    pub async fn rotate_credential_definition_start(
        &self,
        wallet_handle: WalletHandle,
        cred_def_id: CredentialDefinitionId,
        cred_def_config: Option<CredentialDefinitionConfig>,
    ) -> IndyResult<String> {
        trace!(
            "rotate_credential_definition_start > wallet_handle {:?} cred_def_id {:?} cred_def_config {:?}",
            wallet_handle,
            cred_def_id,
            cred_def_config
        );

        let cred_def = self
            .wallet_service
            .get_indy_object::<CredentialDefinition>(wallet_handle, &cred_def_id.0, &RecordOptions::id_value())
            .await?;

        let cred_def = CredentialDefinitionV1::from(cred_def);

        let temp_cred_def = self
            .wallet_service
            .get_indy_object::<TemporaryCredentialDefinition>(wallet_handle, &cred_def_id.0, &RecordOptions::id_value())
            .await;

        if let Ok(temp_cred_def) = temp_cred_def {
            let cred_def_json = serde_json::to_string(&temp_cred_def.cred_def)
                .to_indy(IndyErrorKind::InvalidState, "Can't serialize CredentialDefinition")?;

            let res = Ok(cred_def_json);

            trace!("rotate_credential_definition_start < already exists {:?}", res);

            return res;
        }

        let schema = self
            .wallet_service
            .get_indy_object::<Schema>(wallet_handle, &cred_def.schema_id.0, &RecordOptions::id_value())
            .await?;

        let schema = SchemaV1::from(schema);

        let support_revocation = cred_def_config
            .map(|config| config.support_revocation)
            .unwrap_or_default();

        let (credential_definition_value, cred_priv_key, cred_key_correctness_proof) = self
            ._create_credential_definition(&schema.attr_names, support_revocation)
            .await?;

        let cred_def = CredentialDefinition::CredentialDefinitionV1(CredentialDefinitionV1 {
            id: cred_def_id.clone(),
            schema_id: cred_def.schema_id.clone(),
            signature_type: cred_def.signature_type.clone(),
            tag: cred_def.tag.clone(),
            value: credential_definition_value,
        });

        let cred_def_priv_key = CredentialDefinitionPrivateKey { value: cred_priv_key };

        let cred_def_correctness_proof = CredentialDefinitionCorrectnessProof {
            value: cred_key_correctness_proof,
        };

        let cred_def_json = ::serde_json::to_string(&cred_def)
            .to_indy(IndyErrorKind::InvalidState, "Cannot serialize CredentialDefinition")?;

        let temp_cred_def = TemporaryCredentialDefinition {
            cred_def,
            cred_def_priv_key,
            cred_def_correctness_proof,
        };

        self.wallet_service
            .add_indy_object(wallet_handle, &cred_def_id.0, &temp_cred_def, &HashMap::new())
            .await?;

        let res = Ok(cred_def_json);
        trace!("rotate_credential_definition_start < {:?}", res);
        res
    }

    ///  Apply temporary keys as main for an existing Credential Definition (owned by the caller of
    /// the library).
    ///
    /// WARNING: Rotating the credential definitional keys will result in making all credentials
    /// issued under the previous keys unverifiable.
    ///
    /// #Params

    /// wallet_handle: wallet handle (created by open_wallet).
    /// cred_def_id: an identifier of created credential definition stored in the wallet
    ///
    /// #Returns
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Anoncreds*
    pub async fn rotate_credential_definition_apply(
        &self,
        wallet_handle: WalletHandle,
        cred_def_id: CredentialDefinitionId,
    ) -> IndyResult<()> {
        trace!(
            "rotate_credential_definition_apply > wallet_handle {:?} cred_def_id {:?}",
            wallet_handle,
            cred_def_id
        );

        let _cred_def: CredentialDefinition = self
            .wallet_service
            .get_indy_object(wallet_handle, &cred_def_id.0, &RecordOptions::id_value())
            .await?;

        let temp_cred_def: TemporaryCredentialDefinition = self
            .wallet_service
            .get_indy_object(wallet_handle, &cred_def_id.0, &RecordOptions::id_value())
            .await?;

        self.wallet_service
            .update_indy_object(wallet_handle, &cred_def_id.0, &temp_cred_def.cred_def)
            .await?;

        self.wallet_service
            .update_indy_object(wallet_handle, &cred_def_id.0, &temp_cred_def.cred_def_priv_key)
            .await?;

        self.wallet_service
            .update_indy_object(wallet_handle, &cred_def_id.0, &temp_cred_def.cred_def_correctness_proof)
            .await?;

        self.wallet_service
            .delete_indy_record::<TemporaryCredentialDefinition>(wallet_handle, &cred_def_id.0)
            .await?;

        trace!("rotate_credential_definition_apply <<<");
        Ok(())
    }

    /// Create a new revocation registry for the given credential definition as tuple of entities
    /// - Revocation registry definition that encapsulates credentials definition reference,
    ///   revocation type specific configuration and secrets used for credentials revocation
    /// - Revocation registry state that stores the information about revoked entities in a
    ///   non-disclosing way. The state can be represented as ordered list of revocation registry
    ///   entries were each entry represents the list of revocation or issuance operations.
    ///
    /// Revocation registry definition entity contains private and public parts. Private part will
    /// be stored in the wallet. Public part will be returned as json intended to be shared with
    /// all anoncreds workflow actors usually by publishing REVOC_REG_DEF transaction
    /// to Indy distributed ledger.
    ///
    /// Revocation registry state is stored on the wallet and also intended to be shared as the
    /// ordered list of REVOC_REG_ENTRY transactions. This call initializes the state in the
    /// wallet and returns the initial entry.
    ///
    /// Some revocation registry types (for example, 'CL_ACCUM') can require generation of binary
    /// blob called tails used to hide information about revoked credentials in public
    /// revocation registry and intended to be distributed out of leger (REVOC_REG_DEF transaction
    /// will still contain uri and hash of tails). This call requires access to pre-configured
    /// blob storage writer instance handle that will allow to write generated tails.
    ///
    /// #Params

    /// wallet_handle: wallet handle (created by open_wallet).
    /// issuer_did: a DID of the issuer
    /// revoc_def_type: revocation registry type (optional, default value depends on credential
    /// definition type). Supported types are:
    /// - 'CL_ACCUM': Type-3 pairing based accumulator implemented according to the algorithm in this paper:
    ///                   https://github.com/hyperledger/ursa/blob/master/libursa/docs/AnonCred.pdf
    ///               This type is default for 'CL' credential definition type.
    /// tag: any string that allows to distinct between revocation registries for the same issuer
    /// and credential definition cred_def_id: id of stored in ledger credential definition
    /// config_json: type-specific configuration of revocation registry as json:
    /// - 'CL_ACCUM': { "issuance_type": (optional) type of issuance. Currently supported: 1)
    ///   ISSUANCE_BY_DEFAULT: all indices are assumed to be issued and initial accumulator is
    ///   calculated over all indices; Revocation Registry is updated only during revocation. 2)
    ///   ISSUANCE_ON_DEMAND: nothing is issued initially accumulator is 1 (used by default);
    ///   "max_cred_num": maximum number of credentials the new registry can process (optional,
    ///   default 100000)
    /// }
    /// tails_writer_handle: handle of blob storage to store tails (returned by
    /// `indy_open_blob_storage_writer`).
    ///
    /// NOTE:
    ///     Recursive creation of folder for Default Tails Writer (correspondent to
    /// `tails_writer_handle`)     in the system-wide temporary directory may fail in some setup
    /// due to permissions: `IO error: Permission denied`.     In this case use `TMPDIR`
    /// environment variable to define temporary directory specific for an application.
    ///
    /// #Returns
    /// revoc_reg_id: identifier of created revocation registry definition
    /// revoc_reg_def_json: public part of revocation registry definition
    ///     {
    ///         "id": string - ID of the Revocation Registry,
    ///         "revocDefType": string - Revocation Registry type (only CL_ACCUM is supported for
    /// now),         "tag": string - Unique descriptive ID of the Registry,
    ///         "credDefId": string - ID of the corresponding CredentialDefinition,
    ///         "value": Registry-specific data {
    ///             "issuanceType": string - Type of Issuance(ISSUANCE_BY_DEFAULT or
    /// ISSUANCE_ON_DEMAND),             "maxCredNum": number - Maximum number of credentials
    /// the Registry can serve.             "tailsHash": string - Hash of tails.
    ///             "tailsLocation": string - Location of tails file.
    ///             "publicKeys": <public_keys> - Registry's public key (opaque type that contains
    /// data structures internal to Ursa.                                                       
    /// It should not be parsed and are likely to change in future versions).         },
    ///         "ver": string - version of revocation registry definition json.
    ///     }
    /// revoc_reg_entry_json: revocation registry entry that defines initial state of revocation
    /// registry {
    ///     value: {
    ///         prevAccum: string - previous accumulator value.
    ///         accum: string - current accumulator value.
    ///         issued: array<number> - an array of issued indices.
    ///         revoked: array<number> an array of revoked indices.
    ///     },
    ///     ver: string - version revocation registry entry json
    /// }
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Anoncreds*
    pub async fn create_and_store_revocation_registry(
        &self,
        wallet_handle: WalletHandle,
        issuer_did: DidValue,
        type_: Option<String>,
        tag: String,
        cred_def_id: CredentialDefinitionId,
        config: RevocationRegistryConfig,
        tails_writer_handle: i32,
    ) -> IndyResult<(String, String, String)> {
        trace!(
            "create_and_store_revocation_registry > wallet_handle {:?} issuer_did {:?} type_ {:?} tag: {:?} \
             cred_def_id {:?} config: {:?} tails_handle {:?}",
            wallet_handle,
            issuer_did,
            type_,
            tag,
            cred_def_id,
            config,
            tails_writer_handle
        );

        match (issuer_did.get_method(), cred_def_id.get_method()) {
            (None, Some(_)) => {
                return Err(IndyError::from_msg(
                    IndyErrorKind::InvalidStructure,
                    "You can't use unqualified Did with fully qualified Credential Definition",
                ));
            }
            (Some(_), None) => {
                return Err(IndyError::from_msg(
                    IndyErrorKind::InvalidStructure,
                    "You can't use fully qualified Did with unqualified Credential Definition",
                ));
            }
            _ => {}
        };

        let rev_reg_type = if let Some(type_) = type_ {
            serde_json::from_str::<RegistryType>(&format!("\"{}\"", type_))
                .to_indy(IndyErrorKind::InvalidStructure, "Invalid Registry Type format")?
        } else {
            RegistryType::CL_ACCUM
        };

        let issuance_type = config.issuance_type.clone().unwrap_or(IssuanceType::ISSUANCE_ON_DEMAND);

        let max_cred_num = config.max_cred_num.unwrap_or(100000);

        let rev_reg_id = RevocationRegistryId::new(&issuer_did, &cred_def_id, &rev_reg_type.to_str(), &tag)?;

        if let (Ok(rev_reg_def), Ok(rev_reg)) = (
            self.wallet_service
                .get_indy_record_value::<RevocationRegistryDefinition>(
                    wallet_handle,
                    &rev_reg_id.0,
                    &RecordOptions::id_value(),
                )
                .await,
            self.wallet_service
                .get_indy_record_value::<RevocationRegistry>(wallet_handle, &rev_reg_id.0, &RecordOptions::id_value())
                .await,
        ) {
            let res = Ok((cred_def_id.0.to_string(), rev_reg_def, rev_reg));

            trace!("create_and_store_revocation_registry < already exists {:?}", res);

            return res;
        }

        let cred_def: CredentialDefinition = self
            .wallet_service
            .get_indy_object(wallet_handle, &cred_def_id.0, &RecordOptions::id_value())
            .await?;

        let (revoc_public_keys, revoc_key_private, revoc_registry, mut revoc_tails_generator) =
            self.issuer_service.new_revocation_registry(
                &CredentialDefinitionV1::from(cred_def),
                max_cred_num,
                issuance_type.to_bool(),
                &issuer_did,
            )?;

        let (tails_location, tails_hash) = store_tails_from_generator(
            self.blob_storage_service.clone(),
            tails_writer_handle,
            &mut revoc_tails_generator,
        )
        .await?;

        let revoc_reg_def_value = RevocationRegistryDefinitionValue {
            max_cred_num,
            issuance_type,
            public_keys: revoc_public_keys,
            tails_location,
            tails_hash,
        };

        let revoc_reg_def =
            RevocationRegistryDefinition::RevocationRegistryDefinitionV1(RevocationRegistryDefinitionV1 {
                id: rev_reg_id.clone(),
                revoc_def_type: rev_reg_type,
                tag: tag.to_string(),
                cred_def_id: cred_def_id.clone(),
                value: revoc_reg_def_value,
            });

        let revoc_reg = RevocationRegistry::RevocationRegistryV1(RevocationRegistryV1 { value: revoc_registry });

        let revoc_reg_def_priv = RevocationRegistryDefinitionPrivate {
            value: revoc_key_private,
        };

        let revoc_reg_def_json = self
            .wallet_service
            .add_indy_object(wallet_handle, &rev_reg_id.0, &revoc_reg_def, &HashMap::new())
            .await?;

        let revoc_reg_json = self
            .wallet_service
            .add_indy_object(wallet_handle, &rev_reg_id.0, &revoc_reg, &HashMap::new())
            .await?;

        self.wallet_service
            .add_indy_object(wallet_handle, &rev_reg_id.0, &revoc_reg_def_priv, &HashMap::new())
            .await?;

        let rev_reg_info = RevocationRegistryInfo {
            id: rev_reg_id.clone(),
            curr_id: 0,
            used_ids: HashSet::new(),
        };

        self.wallet_service
            .add_indy_object(wallet_handle, &rev_reg_id.0, &rev_reg_info, &HashMap::new())
            .await?;

        let res = Ok((rev_reg_id.0, revoc_reg_def_json, revoc_reg_json));
        trace!("create_and_store_revocation_registry < {:?}", res);
        res
    }

    /// Create credential offer that will be used by Prover for
    /// credential request creation. Offer includes nonce and key correctness proof
    /// for authentication between protocol steps and integrity checking.
    ///
    /// #Params

    /// wallet_handle: wallet handle (created by open_wallet)
    /// cred_def_id: id of credential definition stored in the wallet
    ///
    /// #Returns
    /// credential offer json:
    ///     {
    ///         "schema_id": string, - identifier of schema
    ///         "cred_def_id": string, - identifier of credential definition
    ///         // Fields below can depend on Credential Definition type
    ///         "nonce": string,
    ///         "key_correctness_proof" : key correctness proof for credential definition
    /// correspondent to cred_def_id                                   (opaque type that
    /// contains data structures internal to Ursa.                                   It should
    /// not be parsed and are likely to change in future versions).     }
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Anoncreds*
    pub async fn create_credential_offer(
        &self,
        wallet_handle: WalletHandle,
        cred_def_id: CredentialDefinitionId,
    ) -> IndyResult<String> {
        trace!(
            "create_credential_offer > wallet_handle {:?} cred_def_id {:?}",
            wallet_handle,
            cred_def_id
        );

        let cred_def_correctness_proof: CredentialDefinitionCorrectnessProof = self
            .wallet_service
            .get_indy_object(wallet_handle, &cred_def_id.0, &RecordOptions::id_value())
            .await?;

        let nonce = new_nonce()?;

        let schema_id = self._wallet_get_schema_id(wallet_handle, &cred_def_id.0).await?; // TODO: FIXME get CredDef from wallet and use CredDef.schema_id

        let credential_offer = CredentialOffer {
            schema_id,
            cred_def_id: cred_def_id.clone(),
            key_correctness_proof: cred_def_correctness_proof.value,
            nonce,
            method_name: None,
        };

        let credential_offer_json = serde_json::to_string(&credential_offer)
            .to_indy(IndyErrorKind::InvalidState, "Cannot serialize CredentialOffer")?;

        let res = Ok(credential_offer_json);
        trace!("create_credential_offer < {:?}", res);
        res
    }

    /// Check Cred Request for the given Cred Offer and issue Credential for the given Cred Request.
    ///
    /// Cred Request must match Cred Offer. The credential definition and revocation registry
    /// definition referenced in Cred Offer and Cred Request must be already created and stored
    /// into the wallet.
    ///
    /// Information for this credential revocation will be store in the wallet as part of revocation
    /// registry under generated cred_revoc_id local for this wallet.
    ///
    /// This call returns revoc registry delta as json file intended to be shared as REVOC_REG_ENTRY
    /// transaction. Note that it is possible to accumulate deltas to reduce ledger load.
    ///
    /// #Params

    /// wallet_handle: wallet handle (created by open_wallet).
    /// cred_offer_json: a cred offer created by indy_issuer_create_credential_offer
    /// cred_req_json: a credential request created by indy_prover_create_credential_req
    /// cred_values_json: a credential containing attribute values for each of requested attribute
    /// names.     Example:
    ///     {
    ///      "attr1" : {"raw": "value1", "encoded": "value1_as_int" },
    ///      "attr2" : {"raw": "value1", "encoded": "value1_as_int" }
    ///     }
    ///   If you want to use empty value for some credential field, you should set "raw" to "" and
    /// "encoded" should not be empty rev_reg_id: id of revocation registry stored in the wallet
    /// blob_storage_reader_handle: configuration of blob storage reader handle that will allow to
    /// read revocation tails (returned by `indy_open_blob_storage_reader`)
    ///
    /// #Returns
    /// cred_json: Credential json containing signed credential values
    ///     {
    ///         "schema_id": string, - identifier of schema
    ///         "cred_def_id": string, - identifier of credential definition
    ///         "rev_reg_def_id", Optional<string>, - identifier of revocation registry
    ///         "values": <see cred_values_json above>, - credential values.
    ///         // Fields below can depend on Cred Def type
    ///         "signature": <credential signature>,
    ///                      (opaque type that contains data structures internal to Ursa.
    ///                       It should not be parsed and are likely to change in future versions).
    ///         "signature_correctness_proof": credential signature correctness proof
    ///                      (opaque type that contains data structures internal to Ursa.
    ///                       It should not be parsed and are likely to change in future versions).
    ///         "rev_reg" - (Optional) revocation registry accumulator value on the issuing moment.
    ///                      (opaque type that contains data structures internal to Ursa.
    ///                       It should not be parsed and are likely to change in future versions).
    ///         "witness" - (Optional) revocation related data
    ///                      (opaque type that contains data structures internal to Ursa.
    ///                       It should not be parsed and are likely to change in future versions).
    ///     }
    /// cred_revoc_id: local id for revocation info (Can be used for revocation of this credential)
    /// revoc_reg_delta_json: Revocation registry delta json with a newly issued credential
    ///
    /// #Errors
    /// Anoncreds*
    /// Common*
    /// Wallet*
    pub async fn new_credential(
        &self,
        wallet_handle: WalletHandle,
        cred_offer: CredentialOffer,
        cred_request: CredentialRequest,
        cred_values: CredentialValues,
        rev_reg_id: Option<RevocationRegistryId>,
        blob_storage_reader_handle: Option<i32>,
    ) -> IndyResult<(String, Option<String>, Option<String>)> {
        trace!(
            "new_credential > wallet_handle {:?} cred_offer {:?} cred_request {:?} cred_values {:?} rev_reg_id {:?} \
             blob_storage_reader_handle {:?}",
            wallet_handle,
            secret!(&cred_offer),
            secret!(&cred_request),
            secret!(&cred_values),
            rev_reg_id,
            blob_storage_reader_handle
        );

        let cred_def_id = match cred_offer.method_name {
            Some(ref method_name) => cred_offer.cred_def_id.qualify(method_name)?,
            None => cred_offer.cred_def_id.clone(),
        };

        let cred_def: CredentialDefinitionV1 = CredentialDefinitionV1::from(
            self.wallet_service
                .get_indy_object::<CredentialDefinition>(wallet_handle, &cred_def_id.0, &RecordOptions::id_value())
                .await?,
        );

        let cred_def_priv_key: CredentialDefinitionPrivateKey = self
            .wallet_service
            .get_indy_object(wallet_handle, &cred_def_id.0, &RecordOptions::id_value())
            .await?;

        let (rev_reg_def, mut rev_reg, rev_reg_def_priv, sdk_tails_accessor, rev_reg_info) = match rev_reg_id {
            Some(ref r_reg_id) => {
                let rev_reg_def: RevocationRegistryDefinitionV1 =
                    RevocationRegistryDefinitionV1::from(self._wallet_get_rev_reg_def(wallet_handle, &r_reg_id).await?);

                let rev_reg: RevocationRegistryV1 =
                    RevocationRegistryV1::from(self._wallet_get_rev_reg(wallet_handle, &r_reg_id).await?);

                let rev_key_priv: RevocationRegistryDefinitionPrivate = self
                    .wallet_service
                    .get_indy_object(wallet_handle, &r_reg_id.0, &RecordOptions::id_value())
                    .await?;

                let mut rev_reg_info = self._wallet_get_rev_reg_info(wallet_handle, &r_reg_id).await?;

                rev_reg_info.curr_id += 1;

                if rev_reg_info.curr_id > rev_reg_def.value.max_cred_num {
                    return Err(err_msg(
                        IndyErrorKind::RevocationRegistryFull,
                        "RevocationRegistryAccumulator is full",
                    ));
                }

                if rev_reg_def.value.issuance_type == IssuanceType::ISSUANCE_ON_DEMAND {
                    rev_reg_info.used_ids.insert(rev_reg_info.curr_id);
                }

                // TODO: FIXME: Review error kind!
                let blob_storage_reader_handle = blob_storage_reader_handle
                    .ok_or_else(|| err_msg(IndyErrorKind::InvalidStructure, "TailsReaderHandle not found"))?;

                let sdk_tails_accessor = SDKTailsAccessor::new(
                    self.blob_storage_service.clone(),
                    blob_storage_reader_handle,
                    &rev_reg_def,
                )
                .await?;

                (
                    Some(rev_reg_def),
                    Some(rev_reg),
                    Some(rev_key_priv),
                    Some(sdk_tails_accessor),
                    Some(rev_reg_info),
                )
            }
            None => (None, None, None, None, None),
        };

        let (credential_signature, signature_correctness_proof, rev_reg_delta) = self.issuer_service.new_credential(
            &cred_def,
            &cred_def_priv_key.value,
            &cred_offer.nonce,
            &cred_request,
            &cred_values,
            rev_reg_info.as_ref().map(|r_reg_info| r_reg_info.curr_id),
            rev_reg_def.as_ref(),
            rev_reg.as_mut().map(|r_reg| &mut r_reg.value),
            rev_reg_def_priv.as_ref().map(|r_reg_def_priv| &r_reg_def_priv.value),
            sdk_tails_accessor.as_ref(),
        )?;

        let witness =
            if let (&Some(ref r_reg_def), &Some(ref r_reg), &Some(ref rev_tails_accessor), &Some(ref rev_reg_info)) =
                (&rev_reg_def, &rev_reg, &sdk_tails_accessor, &rev_reg_info)
            {
                let (issued, revoked) = match r_reg_def.value.issuance_type {
                    IssuanceType::ISSUANCE_ON_DEMAND => (rev_reg_info.used_ids.clone(), HashSet::new()),
                    IssuanceType::ISSUANCE_BY_DEFAULT => (HashSet::new(), rev_reg_info.used_ids.clone()),
                };

                let rev_reg_delta = CryptoRevocationRegistryDelta::from_parts(None, &r_reg.value, &issued, &revoked);

                Some(Witness::new(
                    rev_reg_info.curr_id,
                    r_reg_def.value.max_cred_num,
                    r_reg_def.value.issuance_type.to_bool(),
                    &rev_reg_delta,
                    rev_tails_accessor,
                )?)
            } else {
                None
            };

        let cred_rev_reg_id = match (rev_reg_id.as_ref(), cred_offer.method_name.as_ref()) {
            (Some(rev_reg_id), Some(ref _method_name)) => Some(rev_reg_id.to_unqualified()),
            (rev_reg_id, _) => rev_reg_id.cloned(),
        };

        let credential = Credential {
            schema_id: cred_offer.schema_id.clone(),
            cred_def_id: cred_offer.cred_def_id.clone(),
            rev_reg_id: cred_rev_reg_id,
            values: cred_values.clone(),
            signature: credential_signature,
            signature_correctness_proof,
            rev_reg: rev_reg.map(|r_reg| r_reg.value),
            witness,
        };

        let cred_json =
            serde_json::to_string(&credential).to_indy(IndyErrorKind::InvalidState, "Cannot serialize Credential")?;

        let rev_reg_delta_json = rev_reg_delta
            .map(|r_reg_delta| {
                RevocationRegistryDelta::RevocationRegistryDeltaV1(RevocationRegistryDeltaV1 { value: r_reg_delta })
            })
            .as_ref()
            .map(serde_json::to_string)
            .map_or(Ok(None), |v| v.map(Some))
            .to_indy(IndyErrorKind::InvalidState, "Cannot serialize RevocationRegistryDelta")?;

        if let (Some(r_reg), Some(r_reg_id), Some(r_reg_info)) = (credential.rev_reg, rev_reg_id, rev_reg_info.clone())
        {
            let revoc_reg = RevocationRegistry::RevocationRegistryV1(RevocationRegistryV1 { value: r_reg });

            self.wallet_service
                .update_indy_object(wallet_handle, &r_reg_id.0, &revoc_reg)
                .await?;
            self.wallet_service
                .update_indy_object(wallet_handle, &r_reg_id.0, &r_reg_info)
                .await?;
        };

        let cred_rev_id = rev_reg_info.map(|r_reg_info| r_reg_info.curr_id.to_string());

        let res = Ok((cred_json, cred_rev_id, rev_reg_delta_json));
        trace!("new_credential < {:?}", secret!(&res));
        res
    }

    /// Revoke a credential identified by a cred_revoc_id (returned by
    /// indy_issuer_create_credential).
    ///
    /// The corresponding credential definition and revocation registry must be already
    /// created an stored into the wallet.
    ///
    /// This call returns revoc registry delta as json file intended to be shared as REVOC_REG_ENTRY
    /// transaction. Note that it is possible to accumulate deltas to reduce ledger load.
    ///
    /// #Params

    /// wallet_handle: wallet handle (created by open_wallet).
    /// blob_storage_reader_cfg_handle: configuration of blob storage reader handle that will allow
    /// to read revocation tails (returned by `indy_open_blob_storage_reader`). rev_reg_id: id
    /// of revocation registry stored in wallet cred_revoc_id: local id for revocation info
    /// related to issued credential
    ///
    /// #Returns
    /// revoc_reg_delta_json: Revocation registry delta json with a revoked credential
    /// {
    ///     value: {
    ///         prevAccum: string - previous accumulator value.
    ///         accum: string - current accumulator value.
    ///         revoked: array<number> an array of revoked indices.
    ///     },
    ///     ver: string - version revocation registry delta json
    /// }
    ///
    /// #Errors
    /// Anoncreds*
    /// Common*
    /// Wallet*
    pub async fn revoke_credential(
        &self,
        wallet_handle: WalletHandle,
        blob_storage_reader_handle: i32,
        rev_reg_id: RevocationRegistryId,
        cred_revoc_id: String,
    ) -> IndyResult<String> {
        trace!(
            "revoke_credential > wallet_handle {:?} blob_storage_reader_handle {:?} rev_reg_id {:?} cred_revoc_id {:?}",
            wallet_handle,
            blob_storage_reader_handle,
            rev_reg_id,
            secret!(&cred_revoc_id)
        );

        let cred_revoc_id = AnoncredsHelpers::parse_cred_rev_id(&cred_revoc_id)?;

        let revocation_registry_definition: RevocationRegistryDefinitionV1 =
            RevocationRegistryDefinitionV1::from(self._wallet_get_rev_reg_def(wallet_handle, &rev_reg_id).await?);

        let mut rev_reg: RevocationRegistryV1 =
            RevocationRegistryV1::from(self._wallet_get_rev_reg(wallet_handle, &rev_reg_id).await?);

        let sdk_tails_accessor = SDKTailsAccessor::new(
            self.blob_storage_service.clone(),
            blob_storage_reader_handle,
            &revocation_registry_definition,
        )
        .await?;

        if cred_revoc_id > revocation_registry_definition.value.max_cred_num + 1 {
            return Err(err_msg(
                IndyErrorKind::InvalidUserRevocId,
                format!("Revocation id: {:?} not found in RevocationRegistry", cred_revoc_id),
            ));
        }

        let mut rev_reg_info = self._wallet_get_rev_reg_info(wallet_handle, &rev_reg_id).await?;

        match revocation_registry_definition.value.issuance_type {
            IssuanceType::ISSUANCE_ON_DEMAND => {
                if !rev_reg_info.used_ids.remove(&cred_revoc_id) {
                    return Err(err_msg(
                        IndyErrorKind::InvalidUserRevocId,
                        format!("Revocation id: {:?} not found in RevocationRegistry", cred_revoc_id),
                    ));
                };
            }
            IssuanceType::ISSUANCE_BY_DEFAULT => {
                if !rev_reg_info.used_ids.insert(cred_revoc_id) {
                    return Err(err_msg(
                        IndyErrorKind::InvalidUserRevocId,
                        format!("Revocation id: {:?} not found in RevocationRegistry", cred_revoc_id),
                    ));
                }
            }
        };

        let rev_reg_delta = self.issuer_service.revoke(
            &mut rev_reg.value,
            revocation_registry_definition.value.max_cred_num,
            cred_revoc_id,
            &sdk_tails_accessor,
        )?;

        let rev_reg_delta =
            RevocationRegistryDelta::RevocationRegistryDeltaV1(RevocationRegistryDeltaV1 { value: rev_reg_delta });

        let rev_reg_delta_json = serde_json::to_string(&rev_reg_delta)
            .to_indy(IndyErrorKind::InvalidState, "Cannot serialize RevocationRegistryDelta")?;

        let rev_reg = RevocationRegistry::RevocationRegistryV1(rev_reg);

        self.wallet_service
            .update_indy_object(wallet_handle, &rev_reg_id.0, &rev_reg)
            .await?;

        self.wallet_service
            .update_indy_object(wallet_handle, &rev_reg_id.0, &rev_reg_info)
            .await?;

        let res = Ok(rev_reg_delta_json);
        trace!("revoke_credential < {:?}", res);
        res
    }

    async fn _recovery_credential(
        &self,
        wallet_handle: WalletHandle,
        blob_storage_reader_handle: i32,
        rev_reg_id: &RevocationRegistryId,
        cred_revoc_id: &str,
    ) -> IndyResult<String> {
        trace!(
            "recovery_credential >>> wallet_handle: {:?}, blob_storage_reader_handle: {:?}, rev_reg_id: {:?}, \
             cred_revoc_id: {:?}",
            wallet_handle,
            blob_storage_reader_handle,
            rev_reg_id,
            secret!(cred_revoc_id)
        );

        let cred_revoc_id = AnoncredsHelpers::parse_cred_rev_id(cred_revoc_id)?;

        let revocation_registry_definition: RevocationRegistryDefinitionV1 =
            RevocationRegistryDefinitionV1::from(self._wallet_get_rev_reg_def(wallet_handle, &rev_reg_id).await?);

        let mut rev_reg: RevocationRegistryV1 =
            RevocationRegistryV1::from(self._wallet_get_rev_reg(wallet_handle, &rev_reg_id).await?);

        let sdk_tails_accessor = SDKTailsAccessor::new(
            self.blob_storage_service.clone(),
            blob_storage_reader_handle,
            &revocation_registry_definition,
        )
        .await?;

        if cred_revoc_id > revocation_registry_definition.value.max_cred_num + 1 {
            return Err(err_msg(
                IndyErrorKind::InvalidUserRevocId,
                format!("Revocation id: {:?} not found in RevocationRegistry", cred_revoc_id),
            ));
        }

        let mut rev_reg_info = self._wallet_get_rev_reg_info(wallet_handle, &rev_reg_id).await?;

        match revocation_registry_definition.value.issuance_type {
            IssuanceType::ISSUANCE_ON_DEMAND => {
                if !rev_reg_info.used_ids.insert(cred_revoc_id) {
                    return Err(err_msg(
                        IndyErrorKind::InvalidUserRevocId,
                        format!("Revocation id: {:?} not found in RevocationRegistry", cred_revoc_id),
                    ));
                }
            }
            IssuanceType::ISSUANCE_BY_DEFAULT => {
                if !rev_reg_info.used_ids.remove(&cred_revoc_id) {
                    return Err(err_msg(
                        IndyErrorKind::InvalidUserRevocId,
                        format!("Revocation id: {:?} not found in RevocationRegistry", cred_revoc_id),
                    ));
                }
            }
        };

        let revocation_registry_delta = self.issuer_service.recovery(
            &mut rev_reg.value,
            revocation_registry_definition.value.max_cred_num,
            cred_revoc_id,
            &sdk_tails_accessor,
        )?;

        let rev_reg_delta = RevocationRegistryDelta::RevocationRegistryDeltaV1(RevocationRegistryDeltaV1 {
            value: revocation_registry_delta,
        });

        let rev_reg_delta_json = serde_json::to_string(&rev_reg_delta).to_indy(
            IndyErrorKind::InvalidState,
            "Cannot serialize RevocationRegistryDelta: {:?}",
        )?;

        let rev_reg = RevocationRegistry::RevocationRegistryV1(rev_reg);

        self.wallet_service
            .update_indy_object(wallet_handle, &rev_reg_id.0, &rev_reg)
            .await?;

        self.wallet_service
            .update_indy_object(wallet_handle, &rev_reg_id.0, &rev_reg_info)
            .await?;

        let res = Ok(rev_reg_delta_json);
        trace!("recovery_credential < {:?}", res);
        res
    }

    /// Merge two revocation registry deltas (returned by indy_issuer_create_credential or
    /// indy_issuer_revoke_credential) to accumulate common delta. Send common delta to ledger
    /// to reduce the load.
    ///
    /// #Params

    /// rev_reg_delta_json: revocation registry delta.
    /// {
    ///     value: {
    ///         prevAccum: string - previous accumulator value.
    ///         accum: string - current accumulator value.
    ///         issued: array<number> an array of issued indices.
    ///         revoked: array<number> an array of revoked indices.
    ///     },
    ///     ver: string - version revocation registry delta json
    /// }
    ///
    /// other_rev_reg_delta_json: revocation registry delta for which PrevAccum value is equal to
    /// value of accum field of rev_reg_delta_json parameter.
    ///
    /// #Returns
    /// merged_rev_reg_delta: Merged revocation registry delta
    /// {
    ///     value: {
    ///         prevAccum: string - previous accumulator value.
    ///         accum: string - current accumulator value.
    ///         issued: array<number> an array of issued indices.
    ///         revoked: array<number> an array of revoked indices.
    ///     },
    ///     ver: string - version revocation registry delta json
    /// }
    ///
    /// #Errors
    /// Anoncreds*
    /// Common*
    /// Wallet*
    pub fn merge_revocation_registry_deltas(
        &self,
        rev_reg_delta: RevocationRegistryDelta,
        other_rev_reg_delta: RevocationRegistryDelta,
    ) -> IndyResult<String> {
        trace!(
            "merge_revocation_registry_deltas > rev_reg_delta {:?} other_rev_reg_delta {:?}",
            rev_reg_delta,
            other_rev_reg_delta
        );

        let mut rev_reg_delta = RevocationRegistryDeltaV1::from(rev_reg_delta);
        let other_rev_reg_delta = RevocationRegistryDeltaV1::from(other_rev_reg_delta);

        rev_reg_delta.value.merge(&other_rev_reg_delta.value)?;

        let rev_reg_delta = RevocationRegistryDelta::RevocationRegistryDeltaV1(rev_reg_delta.clone());

        let merged_rev_reg_delta_json = serde_json::to_string(&rev_reg_delta)
            .to_indy(IndyErrorKind::InvalidState, "Cannot serialize RevocationRegistryDelta")?;

        let res = Ok(merged_rev_reg_delta_json);
        trace!("merge_revocation_registry_deltas < {:?}", res);
        res
    }

    // TODO: DELETE IT
    async fn _wallet_set_schema_id(
        &self,
        wallet_handle: WalletHandle,
        id: &str,
        schema_id: &SchemaId,
    ) -> IndyResult<()> {
        self.wallet_service
            .add_record(
                wallet_handle,
                &self.wallet_service.add_prefix("SchemaId"),
                id,
                &schema_id.0,
                &Tags::new(),
            )
            .await
    }

    // TODO: DELETE IT
    async fn _wallet_get_schema_id(&self, wallet_handle: WalletHandle, key: &str) -> IndyResult<SchemaId> {
        let schema_id_record = self
            .wallet_service
            .get_record(
                wallet_handle,
                &self.wallet_service.add_prefix("SchemaId"),
                &key,
                &RecordOptions::id_value(),
            )
            .await?;

        schema_id_record
            .get_value()
            .map(|id| SchemaId(id.to_string()))
            .ok_or_else(|| {
                err_msg(
                    IndyErrorKind::InvalidStructure,
                    format!("SchemaId not found for id: {}", key),
                )
            })
    }

    async fn _wallet_get_rev_reg_def(
        &self,
        wallet_handle: WalletHandle,
        key: &RevocationRegistryId,
    ) -> IndyResult<RevocationRegistryDefinition> {
        self.wallet_service
            .get_indy_object(wallet_handle, &key.0, &RecordOptions::id_value())
            .await
    }

    async fn _wallet_get_rev_reg(
        &self,
        wallet_handle: WalletHandle,
        key: &RevocationRegistryId,
    ) -> IndyResult<RevocationRegistry> {
        self.wallet_service
            .get_indy_object(wallet_handle, &key.0, &RecordOptions::id_value())
            .await
    }

    async fn _wallet_get_rev_reg_info(
        &self,
        wallet_handle: WalletHandle,
        key: &RevocationRegistryId,
    ) -> IndyResult<RevocationRegistryInfo> {
        self.wallet_service
            .get_indy_object(wallet_handle, &key.0, &RecordOptions::id_value())
            .await
    }
}
