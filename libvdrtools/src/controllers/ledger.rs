use std::{string::ToString, sync::Arc};

use crate::utils::crypto::base58::ToBase58;
use indy_api_types::{errors::prelude::*, PoolHandle, WalletHandle};
use indy_wallet::{RecordOptions, WalletService};
use serde_json::{self, Value};

#[cfg(feature = "ffi_api")]
use crate::api::ledger::{CustomFree, CustomTransactionParser};

use crate::{
    domain::{
        anoncreds::{
            credential_definition::{CredentialDefinition, CredentialDefinitionId},
            revocation_registry_definition::{
                RevocationRegistryDefinition, RevocationRegistryDefinitionV1, RevocationRegistryId,
            },
            revocation_registry_delta::{RevocationRegistryDelta, RevocationRegistryDeltaV1},
            schema::{Schema, SchemaId},
        },
        crypto::{
            did::{Did, DidValue},
            key::Key,
        },
        ledger::{
            auth_rule::{AuthRules, Constraint},
            author_agreement::{AcceptanceMechanisms, GetTxnAuthorAgreementData},
            node::NodeOperationData,
            pool::Schedule,
            request::Request,
        },
    },
    services::{CryptoService, LedgerService, PoolService},
};

enum SignatureType {
    Single,
    Multi,
}

pub struct LedgerController {
    pool_service: Arc<PoolService>,
    crypto_service: Arc<CryptoService>,
    wallet_service: Arc<WalletService>,
    ledger_service: Arc<LedgerService>,
}

impl LedgerController {
    pub(crate) fn new(
        pool_service: Arc<PoolService>,
        crypto_service: Arc<CryptoService>,
        wallet_service: Arc<WalletService>,
        ledger_service: Arc<LedgerService>,
    ) -> LedgerController {
        LedgerController {
            pool_service,
            crypto_service,
            wallet_service,
            ledger_service,
        }
    }

    #[cfg(feature = "ffi_api")]
    #[allow(dead_code)] // FIXME [async] TODO implement external SP parsers
    pub(crate) fn register_sp_parser(
        &self,
        txn_type: String,
        parser: CustomTransactionParser,
        free: CustomFree,
    ) -> IndyResult<()> {
        debug!(
            "register_sp_parser > txn_type {:?} parser {:?} free {:?}",
            txn_type, parser, free
        );

        unimplemented!();
        // FIXME: !!!
        // PoolService::register_sp_parser(txn_type, parser, free)
        //     .map_err(IndyError::from)
    }

    /// Signs and submits request message to validator pool.
    ///
    /// Adds submitter information to passed request json, signs it with submitter
    /// sign key (see wallet_sign), and sends signed request message
    /// to validator pool (see write_request).
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// pool_handle: pool handle (created by open_pool_ledger).
    /// wallet_handle: wallet handle (created by open_wallet).
    /// submitter_did: Id of Identity stored in secured Wallet.
    /// request_json: Request data json.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Ledger*
    /// Crypto*
    pub async fn sign_and_submit_request(
        &self,
        pool_handle: PoolHandle,
        wallet_handle: WalletHandle,
        submitter_did: DidValue,
        request_json: String,
    ) -> IndyResult<String> {
        debug!(
            "sign_and_submit_request > pool_handle {:?} \
                wallet_handle {:?} submitter_did {:?} request_json {:?}",
            pool_handle, wallet_handle, submitter_did, request_json
        );

        let signed_request = self
            ._sign_request(
                wallet_handle,
                &submitter_did,
                &request_json,
                SignatureType::Single,
            )
            .await?;

        let res = self
            ._submit_request(pool_handle, signed_request.as_str())
            .await?;

        let res = Ok(res);
        debug!("sign_and_submit_request < {:?}", res);
        res
    }

    /// Publishes request message to validator pool (no signing, unlike sign_and_submit_request).
    ///
    /// The request is sent to the validator pool as is. It's assumed that it's already prepared.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// pool_handle: pool handle (created by open_pool_ledger).
    /// request_json: Request data json.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    /// Ledger*
    pub async fn submit_request(
        &self,
        handle: PoolHandle,
        request_json: String,
    ) -> IndyResult<String> {
        debug!(
            "submit_request > handle {:?} request_json {:?}",
            handle, request_json
        );

        let res = self._submit_request(handle, &request_json).await?;

        let res = Ok(res);
        debug!("submit_request < {:?}", res);
        res
    }

    /// Send action to particular nodes of validator pool.
    ///
    /// The list of requests can be send:
    ///     POOL_RESTART
    ///     GET_VALIDATOR_INFO
    ///
    /// The request is sent to the nodes as is. It's assumed that it's already prepared.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// pool_handle: pool handle (created by open_pool_ledger).
    /// request_json: Request data json.
    /// nodes: (Optional) List of node names to send the request.
    ///        ["Node1", "Node2",...."NodeN"]
    /// timeout: (Optional) Time to wait respond from nodes (override the default timeout) (in sec).
    ///                     Pass -1 to use default timeout
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    /// Ledger*
    pub async fn submit_action(
        &self,
        handle: PoolHandle,
        request_json: String,
        nodes: Option<String>,
        timeout: Option<i32>,
    ) -> IndyResult<String> {
        debug!(
            "submit_action > handle {:?} request_json {:?} nodes {:?} timeout {:?}",
            handle, request_json, nodes, timeout
        );

        self.ledger_service.validate_action(&request_json)?;

        let res = self
            .pool_service
            .send_action(handle, &request_json, nodes.as_deref(), timeout)
            .await?;

        let res = Ok(res);
        debug!("submit_action < {:?}", res);
        res
    }

    /// Signs request message.
    ///
    /// Adds submitter information to passed request json, signs it with submitter
    /// sign key (see wallet_sign).
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// wallet_handle: wallet handle (created by open_wallet).
    /// submitter_did: Id of Identity stored in secured Wallet.
    /// request_json: Request data json.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Signed request json.
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Ledger*
    /// Crypto*
    pub async fn sign_request(
        &self,
        wallet_handle: WalletHandle,
        submitter_did: DidValue,
        request_json: String,
    ) -> IndyResult<String> {
        debug!(
            "sign_request > wallet_handle {:?} submitter_did {:?} request_json {:?}",
            wallet_handle, submitter_did, request_json
        );

        let res = self
            ._sign_request(
                wallet_handle,
                &submitter_did,
                &request_json,
                SignatureType::Single,
            )
            .await?;

        let res = Ok(res);
        debug!("sign_request < {:?}", res);
        res
    }

    /// Multi signs request message.
    ///
    /// Adds submitter information to passed request json, signs it with submitter
    /// sign key (see wallet_sign).
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// wallet_handle: wallet handle (created by open_wallet).
    /// submitter_did: Id of Identity stored in secured Wallet.
    /// request_json: Request data json.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Signed request json.
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Ledger*
    /// Crypto*
    pub async fn multi_sign_request(
        &self,
        wallet_handle: WalletHandle,
        submitter_did: DidValue,
        request_json: String,
    ) -> IndyResult<String> {
        debug!(
            "multi_sign_request > wallet_handle {:?} submitter_did {:?} request_json {:?}",
            wallet_handle, submitter_did, request_json
        );

        let res = self
            ._sign_request(
                wallet_handle,
                &submitter_did,
                &request_json,
                SignatureType::Multi,
            )
            .await?;

        let res = Ok(res);
        debug!("multi_sign_request < {:?}", res);
        res
    }

    /// Builds a request to get a DDO.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
    /// target_did: Target DID as base58-encoded string for 16 or 32 bit DID value.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_get_ddo_request(
        &self,
        submitter_did: Option<DidValue>,
        target_did: DidValue,
    ) -> IndyResult<String> {
        debug!(
            "build_get_ddo_request > submitter_did {:?} target_did {:?}",
            submitter_did, target_did
        );

        let res = self
            .ledger_service
            .build_get_ddo_request(submitter_did.as_ref(), &target_did)?;

        let res = Ok(res);
        debug!("build_get_ddo_request < {:?}", res);
        res
    }

    /// Builds a NYM request to write simplified DID Doc. Request to create a new DID record for a specific user.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
    ///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
    /// target_did: Target DID as base58-encoded string for 16 or 32 bit DID value.
    /// verkey: Target identity verification key as base58-encoded string.
    /// alias: DID's alias.
    /// role: Role of a user DID record:
    ///                             null (common USER)
    ///                             TRUSTEE
    ///                             STEWARD
    ///                             TRUST_ANCHOR
    ///                             ENDORSER - equal to TRUST_ANCHOR that will be removed soon
    ///                             NETWORK_MONITOR
    ///                             empty string to reset role
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub async fn build_nym_request(
        &self,
        submitter_did: DidValue,
        target_did: DidValue,
        verkey: Option<String>,
        alias: Option<String>,
        role: Option<String>,
    ) -> IndyResult<String> {
        debug!(
            "build_nym_request > submitter_did {:?} \
                target_did {:?} verkey {:?} alias {:?} role {:?}",
            submitter_did, target_did, verkey, alias, role
        );

        self.crypto_service.validate_did(&submitter_did)?;
        self.crypto_service.validate_did(&target_did)?;

        if let Some(ref vk) = verkey {
            self.crypto_service.validate_key(vk).await?;
        }

        let res = self.ledger_service.build_nym_request(
            &submitter_did,
            &target_did,
            verkey.as_deref(),
            alias.as_deref(),
            role.as_deref(),
        )?;

        let res = Ok(res);
        debug!("build_nym_request < {:?}", res);
        res
    }

    /// Builds an ATTRIB request. Request to add attribute to a NYM (DID) record.
    ///
    /// Note: one of the fields `hash`, `raw`, `enc` must be specified.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
    ///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
    /// target_did: Target DID as base58-encoded string for 16 or 32 bit DID value.
    /// hash: (Optional) Hash of attribute data.
    /// raw: (Optional) Json, where key is attribute name and value is attribute value.
    /// enc: (Optional) Encrypted value attribute data.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_attrib_request(
        &self,
        submitter_did: DidValue,
        target_did: DidValue,
        hash: Option<String>,
        raw: Option<serde_json::Value>,
        enc: Option<String>,
    ) -> IndyResult<String> {
        debug!(
            "build_attrib_request > submitter_did {:?} \
                target_did {:?} hash {:?} raw {:?} enc {:?}",
            submitter_did, target_did, hash, raw, enc
        );

        self.crypto_service.validate_did(&submitter_did)?;
        self.crypto_service.validate_did(&target_did)?;

        let res = self.ledger_service.build_attrib_request(
            &submitter_did,
            &target_did,
            hash.as_deref(),
            raw.as_ref(),
            enc.as_deref(),
        )?;

        let res = Ok(res);
        debug!("build_attrib_request < {:?}", res);
        res
    }

    /// Builds a GET_ATTRIB request. Request to get information about an Attribute for the specified DID.
    ///
    /// Note: one of the fields `hash`, `raw`, `enc` must be specified.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
    /// target_did: Target DID as base58-encoded string for 16 or 32 bit DID value.
    /// raw: (Optional) Requested attribute name.
    /// hash: (Optional) Requested attribute hash.
    /// enc: (Optional) Requested attribute encrypted value.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_get_attrib_request(
        &self,
        submitter_did: Option<DidValue>,
        target_did: DidValue,
        raw: Option<String>,
        hash: Option<String>,
        enc: Option<String>,
    ) -> IndyResult<String> {
        debug!(
            "build_get_attrib_request > submitter_did {:?} \
                target_did {:?} raw {:?} hash {:?} enc {:?}",
            submitter_did, target_did, raw, hash, enc
        );

        self._validate_opt_did(submitter_did.as_ref())?;
        self.crypto_service.validate_did(&target_did)?;

        let res = self.ledger_service.build_get_attrib_request(
            submitter_did.as_ref(),
            &target_did,
            raw.as_deref(),
            hash.as_deref(),
            enc.as_deref(),
        )?;

        let res = Ok(res);
        debug!("build_get_attrib_request < {:?}", res);
        res
    }

    /// Builds a GET_NYM request. Request to get information about a DID (NYM).
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
    /// target_did: Target DID as base58-encoded string for 16 or 32 bit DID value.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_get_nym_request(
        &self,
        submitter_did: Option<DidValue>,
        target_did: DidValue,
    ) -> IndyResult<String> {
        debug!(
            "build_get_nym_request > submitter_did {:?} target_did {:?}",
            submitter_did, target_did
        );

        self._validate_opt_did(submitter_did.as_ref())?;
        self.crypto_service.validate_did(&target_did)?;

        let res = self
            .ledger_service
            .build_get_nym_request(submitter_did.as_ref(), &target_did)?;

        let res = Ok(res);
        debug!("build_get_attrib_request < {:?}", res);
        res
    }

    /// Parse a GET_NYM response to get DID (NYM) data.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// get_nym_response: response on GET_NYM request.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// DID (NYM) data
    /// {
    ///     did: DID as base58-encoded string for 16 or 32 bit DID value.
    ///     verkey: verification key as base58-encoded string.
    ///     role: Role associated number
    ///                             null (common USER)
    ///                             0 - TRUSTEE
    ///                             2 - STEWARD
    ///                             101 - TRUST_ANCHOR
    ///                             101 - ENDORSER - equal to TRUST_ANCHOR that will be removed soon
    ///                             201 - NETWORK_MONITOR
    /// }
    ///
    ///
    /// #Errors
    /// Common*
    pub fn parse_get_nym_response(&self, get_nym_response: String) -> IndyResult<String> {
        debug!(
            "parse_get_nym_response > get_nym_response {:?}",
            get_nym_response
        );

        let res = self
            .ledger_service
            .parse_get_nym_response(&get_nym_response)?;

        let res = Ok(res);
        debug!("parse_get_nym_response < {:?}", res);
        res
    }

    /// Builds a SCHEMA request. Request to add Credential's schema.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
    ///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
    /// data: Credential schema.
    /// {
    ///     id: identifier of schema
    ///     attrNames: array of attribute name strings (the number of attributes should be less or equal than 125)
    ///     name: Schema's name string
    ///     version: Schema's version string,
    ///     ver: Version of the Schema json
    /// }
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_schema_request(
        &self,
        submitter_did: DidValue,
        schema: Schema,
    ) -> IndyResult<String> {
        debug!(
            "build_schema_request > submitter_did {:?} schema {:?}",
            submitter_did, schema
        );

        self.crypto_service.validate_did(&submitter_did)?;

        let res = self
            .ledger_service
            .build_schema_request(&submitter_did, schema)?;

        let res = Ok(res);
        debug!("build_schema_request < {:?}", res);
        res
    }

    /// Builds a GET_SCHEMA request. Request to get Credential's Schema.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
    /// id: Schema ID in ledger
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_get_schema_request(
        &self,
        submitter_did: Option<DidValue>,
        id: SchemaId,
    ) -> IndyResult<String> {
        debug!(
            "build_get_schema_request > submitter_did {:?} id {:?}",
            submitter_did, id
        );

        self._validate_opt_did(submitter_did.as_ref())?;

        let res = self
            .ledger_service
            .build_get_schema_request(submitter_did.as_ref(), &id)?;

        let res = Ok(res);
        debug!("build_get_schema_request < {:?}", res);
        res
    }

    /// Parse a GET_SCHEMA response to get Schema in the format compatible with Anoncreds API.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// get_schema_response: response of GET_SCHEMA request.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Schema Id and Schema json.
    /// {
    ///     id: identifier of schema
    ///     attrNames: array of attribute name strings
    ///     name: Schema's name string
    ///     version: Schema's version string
    ///     ver: Version of the Schema json
    /// }
    ///
    /// #Errors
    /// Common*
    pub fn parse_get_schema_response(
        &self,
        get_schema_response: String,
    ) -> IndyResult<(String, String)> {
        debug!(
            "parse_get_schema_response > get_schema_response {:?}",
            get_schema_response
        );

        let res = self
            .ledger_service
            .parse_get_schema_response(&get_schema_response, None)?;

        let res = Ok(res);
        debug!("parse_get_schema_response < {:?}", res);
        res
    }

    /// Builds an CRED_DEF request. Request to add a Credential Definition (in particular, public key),
    /// that Issuer creates for a particular Credential Schema.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
    ///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
    /// data: credential definition json
    /// {
    ///     id: string - identifier of credential definition
    ///     schemaId: string - identifier of stored in ledger schema
    ///     type: string - type of the credential definition. CL is the only supported type now.
    ///     tag: string - allows to distinct between credential definitions for the same issuer and schema
    ///     value: Dictionary with Credential Definition's data: {
    ///         primary: primary credential public key,
    ///         Optional<revocation>: revocation credential public key
    ///     },
    ///     ver: Version of the CredDef json
    /// }
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_cred_def_request(
        &self,
        submitter_did: DidValue,
        cred_def: CredentialDefinition,
    ) -> IndyResult<String> {
        debug!(
            "build_cred_def_request > submitter_did {:?} cred_def {:?}",
            submitter_did, cred_def
        );

        self.crypto_service.validate_did(&submitter_did)?;

        let res = self
            .ledger_service
            .build_cred_def_request(&submitter_did, cred_def)?;

        let res = Ok(res);
        debug!("build_cred_def_request < {:?}", res);
        res
    }

    /// Builds a GET_CRED_DEF request. Request to get a Credential Definition (in particular, public key),
    /// that Issuer creates for a particular Credential Schema.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
    /// id: Credential Definition ID in ledger.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_get_cred_def_request(
        &self,
        submitter_did: Option<DidValue>,
        id: CredentialDefinitionId,
    ) -> IndyResult<String> {
        debug!(
            "build_get_cred_def_request > submitter_did {:?} id {:?}",
            submitter_did, id
        );

        self._validate_opt_did(submitter_did.as_ref())?;

        let res = self
            .ledger_service
            .build_get_cred_def_request(submitter_did.as_ref(), &id)?;

        let res = Ok(res);
        debug!("build_get_cred_def_request < {:?}", res);
        res
    }

    /// Parse a GET_CRED_DEF response to get Credential Definition in the format compatible with Anoncreds API.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// get_cred_def_response: response of GET_CRED_DEF request.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Credential Definition Id and Credential Definition json.
    /// {
    ///     id: string - identifier of credential definition
    ///     schemaId: string - identifier of stored in ledger schema
    ///     type: string - type of the credential definition. CL is the only supported type now.
    ///     tag: string - allows to distinct between credential definitions for the same issuer and schema
    ///     value: Dictionary with Credential Definition's data: {
    ///         primary: primary credential public key,
    ///         Optional<revocation>: revocation credential public key
    ///     },
    ///     ver: Version of the Credential Definition json
    /// }
    ///
    /// #Errors
    /// Common*
    pub fn parse_get_cred_def_response(
        &self,
        get_cred_def_response: String,
    ) -> IndyResult<(String, String)> {
        debug!(
            "parse_get_cred_def_response > get_cred_def_response {:?}",
            get_cred_def_response
        );

        let res = self
            .ledger_service
            .parse_get_cred_def_response(&get_cred_def_response, None)?;

        let res = Ok(res);
        debug!("parse_get_cred_def_response < {:?}", res);
        res
    }

    /// Builds a NODE request. Request to add a new node to the pool, or updates existing in the pool.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
    ///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
    /// target_did: Target Node's DID.  It differs from submitter_did field.
    /// data: Data associated with the Node: {
    ///     alias: string - Node's alias
    ///     blskey: string - (Optional) BLS multi-signature key as base58-encoded string.
    ///     blskey_pop: string - (Optional) BLS key proof of possession as base58-encoded string.
    ///     client_ip: string - (Optional) Node's client listener IP address.
    ///     client_port: string - (Optional) Node's client listener port.
    ///     node_ip: string - (Optional) The IP address other Nodes use to communicate with this Node.
    ///     node_port: string - (Optional) The port other Nodes use to communicate with this Node.
    ///     services: array<string> - (Optional) The service of the Node. VALIDATOR is the only supported one now.
    /// }
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_node_request(
        &self,
        submitter_did: DidValue,
        target_did: DidValue,
        data: NodeOperationData,
    ) -> IndyResult<String> {
        debug!(
            "build_node_request > submitter_did {:?} target_did {:?} data {:?}",
            submitter_did, target_did, data
        );

        self.crypto_service.validate_did(&submitter_did)?;

        let res = self
            .ledger_service
            .build_node_request(&submitter_did, &target_did, data)?;

        let res = Ok(res);
        debug!("build_node_request < {:?}", res);
        res
    }

    /// Builds a GET_VALIDATOR_INFO request.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: DID of the read request sender.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_get_validator_info_request(&self, submitter_did: DidValue) -> IndyResult<String> {
        info!(
            "build_get_validator_info_request > submitter_did {:?}",
            submitter_did
        );

        self.crypto_service.validate_did(&submitter_did)?;

        let res = self
            .ledger_service
            .build_get_validator_info_request(&submitter_did)?;

        let res = Ok(res);
        info!("build_get_validator_info_request < {:?}", res);
        res
    }

    /// Builds a GET_TXN request. Request to get any transaction by its seq_no.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
    /// ledger_type: (Optional) type of the ledger the requested transaction belongs to:
    ///     DOMAIN - used default,
    ///     POOL,
    ///     CONFIG
    ///     any number
    /// seq_no: requested transaction sequence number as it's stored on Ledger.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_get_txn_request(
        &self,
        submitter_did: Option<DidValue>,
        ledger_type: Option<String>,
        seq_no: i32,
    ) -> IndyResult<String> {
        debug!(
            "build_get_txn_request > submitter_did {:?} ledger_type {:?} seq_no {:?}",
            submitter_did, ledger_type, seq_no
        );

        self._validate_opt_did(submitter_did.as_ref())?;

        let res = self.ledger_service.build_get_txn_request(
            submitter_did.as_ref(),
            ledger_type.as_deref(),
            seq_no,
        )?;

        let res = Ok(res);
        debug!("build_get_txn_request < {:?}", res);
        res
    }

    /// Builds a POOL_CONFIG request. Request to change Pool's configuration.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
    ///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
    /// writes: Whether any write requests can be processed by the pool
    ///         (if false, then pool goes to read-only state). True by default.
    /// force: Whether we should apply transaction (for example, move pool to read-only state)
    ///        without waiting for consensus of this transaction.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_pool_config_request(
        &self,
        submitter_did: DidValue,
        writes: bool,
        force: bool,
    ) -> IndyResult<String> {
        debug!(
            "build_pool_config_request > submitter_did {:?} writes {:?} force {:?}",
            submitter_did, writes, force
        );

        self.crypto_service.validate_did(&submitter_did)?;

        let res = self
            .ledger_service
            .build_pool_config(&submitter_did, writes, force)?;

        let res = Ok(res);
        debug!("build_pool_config_request < {:?}", res);
        res
    }

    /// Builds a POOL_RESTART request.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
    /// action:        Action that pool has to do after received transaction. Either `start` or `cancel`.
    /// datetime:      <Optional> Restart time in datetime format. Skip to restart as early as possible.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_pool_restart_request(
        &self,
        submitter_did: DidValue,
        action: String,
        datetime: Option<String>,
    ) -> IndyResult<String> {
        debug!(
            "build_pool_restart_request > submitter_did {:?} action {:?} datetime {:?}",
            submitter_did, action, datetime
        );

        self.crypto_service.validate_did(&submitter_did)?;

        let res =
            self.ledger_service
                .build_pool_restart(&submitter_did, &action, datetime.as_deref())?;

        let res = Ok(res);
        debug!("build_pool_config_request < {:?}", res);
        res
    }

    /// Builds a POOL_UPGRADE request. Request to upgrade the Pool (sent by Trustee).
    /// It upgrades the specified Nodes (either all nodes in the Pool, or some specific ones).
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
    ///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
    /// name: Human-readable name for the upgrade.
    /// version: The version of indy-node package we perform upgrade to.
    ///          Must be greater than existing one (or equal if reinstall flag is True).
    /// action: Either start or cancel.
    /// sha256: sha256 hash of the package.
    /// timeout: (Optional) Limits upgrade time on each Node.
    /// schedule: (Optional) Schedule of when to perform upgrade on each node. Map Node DIDs to upgrade time.
    /// justification: (Optional) justification string for this particular Upgrade.
    /// reinstall: Whether it's allowed to re-install the same version. False by default.
    /// force: Whether we should apply transaction (schedule Upgrade) without waiting
    ///        for consensus of this transaction.
    /// package: (Optional) Package to be upgraded.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_pool_upgrade_request(
        &self,
        submitter_did: DidValue,
        name: String,
        version: String,
        action: String,
        sha256: String,
        timeout: Option<u32>,
        schedule: Option<Schedule>,
        justification: Option<String>,
        reinstall: bool,
        force: bool,
        package: Option<String>,
    ) -> IndyResult<String> {
        debug!(
            "build_pool_upgrade_request > submitter_did {:?} \
                name {:?} version {:?} action {:?} sha256 {:?} \
                timeout {:?} schedule {:?} justification {:?} \
                reinstall {:?} force {:?} package {:?}",
            submitter_did,
            name,
            version,
            action,
            sha256,
            timeout,
            schedule,
            justification,
            reinstall,
            force,
            package
        );

        self.crypto_service.validate_did(&submitter_did)?;

        let res = self.ledger_service.build_pool_upgrade(
            &submitter_did,
            &name,
            &version,
            &action,
            &sha256,
            timeout,
            schedule,
            justification.as_deref(),
            reinstall,
            force,
            package.as_deref(),
        )?;

        let res = Ok(res);
        debug!("build_pool_upgrade_request < {:?}", res);
        res
    }

    /// Builds a REVOC_REG_DEF request. Request to add the definition of revocation registry
    /// to an exists credential definition.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
    ///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
    /// data: Revocation Registry data:
    ///     {
    ///         "id": string - ID of the Revocation Registry,
    ///         "revocDefType": string - Revocation Registry type (only CL_ACCUM is supported for now),
    ///         "tag": string - Unique descriptive ID of the Registry,
    ///         "credDefId": string - ID of the corresponding CredentialDefinition,
    ///         "value": Registry-specific data {
    ///             "issuanceType": string - Type of Issuance(ISSUANCE_BY_DEFAULT or ISSUANCE_ON_DEMAND),
    ///             "maxCredNum": number - Maximum number of credentials the Registry can serve.
    ///             "tailsHash": string - Hash of tails.
    ///             "tailsLocation": string - Location of tails file.
    ///             "publicKeys": <public_keys> - Registry's public key.
    ///         },
    ///         "ver": string - version of revocation registry definition json.
    ///     }
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_revoc_reg_def_request(
        &self,
        submitter_did: DidValue,
        data: RevocationRegistryDefinition,
    ) -> IndyResult<String> {
        debug!(
            "build_revoc_reg_def_request > submitter_did {:?} data {:?}",
            submitter_did, data
        );

        let data = RevocationRegistryDefinitionV1::from(data);

        self.crypto_service.validate_did(&submitter_did)?;

        let res = self
            .ledger_service
            .build_revoc_reg_def_request(&submitter_did, data)?;

        let res = Ok(res);
        debug!("build_revoc_reg_def_request < {:?}", res);
        res
    }

    /// Builds a GET_REVOC_REG_DEF request. Request to get a revocation registry definition,
    /// that Issuer creates for a particular Credential Definition.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
    /// id:  ID of Revocation Registry Definition in ledger.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_get_revoc_reg_def_request(
        &self,
        submitter_did: Option<DidValue>,
        id: RevocationRegistryId,
    ) -> IndyResult<String> {
        debug!(
            "build_get_revoc_reg_def_request > submitter_did {:?} id {:?}",
            submitter_did, id
        );

        self._validate_opt_did(submitter_did.as_ref())?;

        let res = self
            .ledger_service
            .build_get_revoc_reg_def_request(submitter_did.as_ref(), &id)?;

        let res = Ok(res);
        debug!("build_get_revoc_reg_def_request < {:?}", res);
        res
    }

    /// Parse a GET_REVOC_REG_DEF response to get Revocation Registry Definition in the format
    /// compatible with Anoncreds API.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// get_revoc_reg_def_response: response of GET_REVOC_REG_DEF request.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Revocation Registry Definition Id and Revocation Registry Definition json.
    /// {
    ///     "id": string - ID of the Revocation Registry,
    ///     "revocDefType": string - Revocation Registry type (only CL_ACCUM is supported for now),
    ///     "tag": string - Unique descriptive ID of the Registry,
    ///     "credDefId": string - ID of the corresponding CredentialDefinition,
    ///     "value": Registry-specific data {
    ///         "issuanceType": string - Type of Issuance(ISSUANCE_BY_DEFAULT or ISSUANCE_ON_DEMAND),
    ///         "maxCredNum": number - Maximum number of credentials the Registry can serve.
    ///         "tailsHash": string - Hash of tails.
    ///         "tailsLocation": string - Location of tails file.
    ///         "publicKeys": <public_keys> - Registry's public key.
    ///     },
    ///     "ver": string - version of revocation registry definition json.
    /// }
    ///
    /// #Errors
    /// Common*
    pub fn parse_revoc_reg_def_response(
        &self,
        get_revoc_reg_def_response: String,
    ) -> IndyResult<(String, String)> {
        debug!(
            "parse_revoc_reg_def_response > get_revoc_reg_def_response {:?}",
            get_revoc_reg_def_response
        );

        let res = self
            .ledger_service
            .parse_get_revoc_reg_def_response(&get_revoc_reg_def_response)?;

        let res = Ok(res);
        debug!("parse_revoc_reg_def_response < {:?}", res);
        res
    }

    /// Builds a REVOC_REG_ENTRY request.  Request to add the RevocReg entry containing
    /// the new accumulator value and issued/revoked indices.
    /// This is just a delta of indices, not the whole list.
    /// So, it can be sent each time a new credential is issued/revoked.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
    ///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
    /// revoc_reg_def_id: ID of the corresponding RevocRegDef.
    /// rev_def_type: Revocation Registry type (only CL_ACCUM is supported for now).
    /// value: Registry-specific data: {
    ///     value: {
    ///         prevAccum: string - previous accumulator value.
    ///         accum: string - current accumulator value.
    ///         issued: array<number> - an array of issued indices.
    ///         revoked: array<number> an array of revoked indices.
    ///     },
    ///     ver: string - version revocation registry entry json
    /// }
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_revoc_reg_entry_request(
        &self,
        submitter_did: DidValue,
        revoc_reg_def_id: RevocationRegistryId,
        revoc_def_type: String,
        value: RevocationRegistryDelta,
    ) -> IndyResult<String> {
        debug!("build_revoc_reg_entry_request > submitter_did {:?} revoc_reg_def_id {:?} revoc_def_type {:?} value {:?}",
               submitter_did, revoc_reg_def_id, revoc_def_type, value);

        let value = RevocationRegistryDeltaV1::from(value);

        self.crypto_service.validate_did(&submitter_did)?;

        let res = self.ledger_service.build_revoc_reg_entry_request(
            &submitter_did,
            &revoc_reg_def_id,
            &revoc_def_type,
            value,
        )?;

        let res = Ok(res);
        debug!("build_revoc_reg_request < {:?}", res);
        res
    }

    /// Builds a GET_REVOC_REG request. Request to get the accumulated state of the Revocation Registry
    /// by ID. The state is defined by the given timestamp.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
    /// revoc_reg_def_id:  ID of the corresponding Revocation Registry Definition in ledger.
    /// timestamp: Requested time represented as a total number of seconds from Unix Epoch
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_get_revoc_reg_request(
        &self,
        submitter_did: Option<DidValue>,
        revoc_reg_def_id: RevocationRegistryId,
        timestamp: i64,
    ) -> IndyResult<String> {
        debug!(
            "build_get_revoc_reg_request > submitter_did {:?} revoc_reg_def_id {:?} timestamp {:?}",
            submitter_did, revoc_reg_def_id, timestamp
        );

        self._validate_opt_did(submitter_did.as_ref())?;

        let res = self.ledger_service.build_get_revoc_reg_request(
            submitter_did.as_ref(),
            &revoc_reg_def_id,
            timestamp,
        )?;

        let res = Ok(res);
        debug!("build_get_revoc_reg_request < {:?}", res);
        res
    }

    /// Parse a GET_REVOC_REG response to get Revocation Registry in the format compatible with Anoncreds API.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// get_revoc_reg_response: response of GET_REVOC_REG request.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Revocation Registry Definition Id, Revocation Registry json and Timestamp.
    /// {
    ///     "value": Registry-specific data {
    ///         "accum": string - current accumulator value.
    ///     },
    ///     "ver": string - version revocation registry json
    /// }
    ///
    /// #Errors
    /// Common*
    pub fn parse_revoc_reg_response(
        &self,
        get_revoc_reg_response: String,
    ) -> IndyResult<(String, String, u64)> {
        debug!(
            "parse_revoc_reg_response > get_revoc_reg_response {:?}",
            get_revoc_reg_response
        );

        let res = self
            .ledger_service
            .parse_get_revoc_reg_response(&get_revoc_reg_response)?;

        let res = Ok(res);
        debug!("parse_revoc_reg_response < {:?}", res);
        res
    }

    /// Builds a GET_REVOC_REG_DELTA request. Request to get the delta of the accumulated state of the Revocation Registry.
    /// The Delta is defined by from and to timestamp fields.
    /// If from is not specified, then the whole state till to will be returned.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
    /// revoc_reg_def_id:  ID of the corresponding Revocation Registry Definition in ledger.
    /// from: Requested time represented as a total number of seconds from Unix Epoch
    /// to: Requested time represented as a total number of seconds from Unix Epoch
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_get_revoc_reg_delta_request(
        &self,
        submitter_did: Option<DidValue>,
        revoc_reg_def_id: RevocationRegistryId,
        from: Option<i64>,
        to: i64,
    ) -> IndyResult<String> {
        debug!(
            "build_get_revoc_reg_delta_request > submitter_did {:?} \
                revoc_reg_def_id {:?} from {:?} to {:?}",
            submitter_did, revoc_reg_def_id, from, to
        );

        self._validate_opt_did(submitter_did.as_ref())?;

        let res = self.ledger_service.build_get_revoc_reg_delta_request(
            submitter_did.as_ref(),
            &revoc_reg_def_id,
            from,
            to,
        )?;

        let res = Ok(res);
        debug!("build_get_revoc_reg_delta_request < {:?}", res);
        res
    }

    /// Parse a GET_REVOC_REG_DELTA response to get Revocation Registry Delta in the format compatible with Anoncreds API.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// get_revoc_reg_response: response of GET_REVOC_REG_DELTA request.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Revocation Registry Definition Id, Revocation Registry Delta json and Timestamp.
    /// {
    ///     "value": Registry-specific data {
    ///         prevAccum: string - previous accumulator value.
    ///         accum: string - current accumulator value.
    ///         issued: array<number> - an array of issued indices.
    ///         revoked: array<number> an array of revoked indices.
    ///     },
    ///     "ver": string - version revocation registry delta json
    /// }
    ///
    /// #Errors
    /// Common*
    pub fn parse_revoc_reg_delta_response(
        &self,
        get_revoc_reg_delta_response: String,
    ) -> IndyResult<(String, String, u64)> {
        debug!(
            "parse_revoc_reg_delta_response > get_revoc_reg_delta_response {:?}",
            get_revoc_reg_delta_response
        );

        let res = self
            .ledger_service
            .parse_get_revoc_reg_delta_response(&get_revoc_reg_delta_response)?;

        let res = Ok(res);
        debug!("parse_revoc_reg_delta_response < {:?}", res);
        res
    }

    /// Parse transaction response to fetch metadata.
    /// The important use case for this method is validation of Node's response freshens.
    ///
    /// Distributed Ledgers can reply with outdated information for consequence read request after write.
    /// To reduce pool load libvdrtools sends read requests to one random node in the pool.
    /// Consensus validation is performed based on validation of nodes multi signature for current ledger Merkle Trie root.
    /// This multi signature contains information about the latest ldeger's transaction ordering time and sequence number that this method returns.
    ///
    /// If node that returned response for some reason is out of consensus and has outdated ledger
    /// it can be caught by analysis of the returned latest ledger's transaction ordering time and sequence number.
    ///
    /// There are two ways to filter outdated responses:
    ///     1) based on "seqNo" - sender knows the sequence number of transaction that he consider as a fresh enough.
    ///     2) based on "txnTime" - sender knows the timestamp that he consider as a fresh enough.
    ///
    /// Note: response of GET_VALIDATOR_INFO request isn't supported
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// response: response of write or get request.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// response metadata.
    /// {
    ///     "seqNo": Option<u64> - transaction sequence number,
    ///     "txnTime": Option<u64> - transaction ordering time,
    ///     "lastSeqNo": Option<u64> - the latest transaction seqNo for particular Node,
    ///     "lastTxnTime": Option<u64> - the latest transaction ordering time for particular Node
    /// }
    ///
    /// #Errors
    /// Common*
    /// Ledger*
    pub fn get_response_metadata(&self, response: String) -> IndyResult<String> {
        debug!("get_response_metadata > response {:?}", response);

        let metadata = PoolService::parse_response_metadata(&response)?;

        let res = serde_json::to_string(&metadata).to_indy(
            IndyErrorKind::InvalidState,
            "Cannot serialize ResponseMetadata",
        )?;

        let res = Ok(res);
        debug!("get_response_metadata < {:?}", res);
        res
    }

    /// Builds a AUTH_RULE request. Request to change authentication rules for a ledger transaction.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
    ///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
    /// txn_type: ledger transaction alias or associated value.
    /// action: type of an action.
    ///     Can be either "ADD" (to add a new rule) or "EDIT" (to edit an existing one).
    /// field: transaction field.
    /// old_value: (Optional) old value of a field, which can be changed to a new_value (mandatory for EDIT action).
    /// new_value: (Optional) new value that can be used to fill the field.
    /// constraint: set of constraints required for execution of an action in the following format:
    ///     {
    ///         constraint_id - <string> type of a constraint.
    ///             Can be either "ROLE" to specify final constraint or  "AND"/"OR" to combine constraints.
    ///         role - <string> (optional) role of a user which satisfy to constrain.
    ///         sig_count - <u32> the number of signatures required to execution action.
    ///         need_to_be_owner - <bool> (optional) if user must be an owner of transaction (false by default).
    ///         off_ledger_signature - <bool> (optional) allow signature of unknow for ledger did (false by default).
    ///         metadata - <object> (optional) additional parameters of the constraint.
    ///     }
    /// can be combined by
    ///     {
    ///         'constraint_id': <"AND" or "OR">
    ///         'auth_constraints': [<constraint_1>, <constraint_2>]
    ///     }
    ///
    /// Default ledger auth rules: https://github.com/hyperledger/indy-node/blob/master/docs/source/auth_rules.md
    ///
    /// More about AUTH_RULE request: https://github.com/hyperledger/indy-node/blob/master/docs/source/requests.md#auth_rule
    ///
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_auth_rule_request(
        &self,
        submitter_did: DidValue,
        txn_type: String,
        action: String,
        field: String,
        old_value: Option<String>,
        new_value: Option<String>,
        constraint: Constraint,
    ) -> IndyResult<String> {
        debug!(
            "build_auth_rule_request > submitter_did {:?} txn_type {:?} \
                action {:?} field {:?} old_value {:?} new_value {:?} \
                constraint {:?}",
            submitter_did, txn_type, action, field, old_value, new_value, constraint
        );

        self.crypto_service.validate_did(&submitter_did)?;

        let res = self.ledger_service.build_auth_rule_request(
            &submitter_did,
            &txn_type,
            &action,
            &field,
            old_value.as_deref(),
            new_value.as_deref(),
            constraint,
        )?;

        let res = Ok(res);
        debug!("build_auth_rule_request < {:?}", res);
        res
    }

    /// Builds a AUTH_RULES request. Request to change multiple authentication rules for a ledger transaction.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
    ///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
    /// rules: a list of auth rules: [
    ///     {
    ///         "auth_type": ledger transaction alias or associated value,
    ///         "auth_action": type of an action,
    ///         "field": transaction field,
    ///         "old_value": (Optional) old value of a field, which can be changed to a new_value (mandatory for EDIT action),
    ///         "new_value": (Optional) new value that can be used to fill the field,
    ///         "constraint": set of constraints required for execution of an action in the format described above for `indy_build_auth_rule_request` function.
    ///     },
    ///     ...
    /// ]
    ///
    /// Default ledger auth rules: https://github.com/hyperledger/indy-node/blob/master/docs/source/auth_rules.md
    ///
    /// More about AUTH_RULES request: https://github.com/hyperledger/indy-node/blob/master/docs/source/requests.md#auth_rules
    ///
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_auth_rules_request(
        &self,
        submitter_did: DidValue,
        rules: AuthRules,
    ) -> IndyResult<String> {
        debug!(
            "build_auth_rules_request > submitter_did {:?} rules {:?}",
            submitter_did, rules
        );

        self._validate_opt_did(Some(&submitter_did))?;

        let res = self
            .ledger_service
            .build_auth_rules_request(&submitter_did, rules)?;

        let res = Ok(res);
        debug!("build_auth_rules_request < {:?}", res);
        res
    }

    /// Builds a GET_AUTH_RULE request. Request to get authentication rules for ledger transactions.
    ///
    /// NOTE: Either none or all transaction related parameters must be specified (`old_value` can be skipped for `ADD` action).
    ///     * none - to get all authentication rules for all ledger transactions
    ///     * all - to get authentication rules for specific action (`old_value` can be skipped for `ADD` action)
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
    /// txn_type: (Optional) target ledger transaction alias or associated value.
    /// action: (Optional) target action type. Can be either "ADD" or "EDIT".
    /// field: (Optional) target transaction field.
    /// old_value: (Optional) old value of field, which can be changed to a new_value (mandatory for EDIT action).
    /// new_value: (Optional) new value that can be used to fill the field.
    ///
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_get_auth_rule_request(
        &self,
        submitter_did: Option<DidValue>,
        txn_type: Option<String>,
        action: Option<String>,
        field: Option<String>,
        old_value: Option<String>,
        new_value: Option<String>,
    ) -> IndyResult<String> {
        debug!(
            "build_get_auth_rule_request > submitter_did {:?} \
            auth_type {:?} auth_action {:?} field {:?} \
            old_value {:?} new_value {:?}",
            submitter_did, txn_type, action, field, old_value, new_value
        );

        self._validate_opt_did(submitter_did.as_ref())?;

        let res = self.ledger_service.build_get_auth_rule_request(
            submitter_did.as_ref(),
            txn_type.as_deref(),
            action.as_deref(),
            field.as_deref(),
            old_value.as_deref(),
            new_value.as_deref(),
        )?;

        let res = Ok(res);
        debug!("build_get_auth_rule_request < {:?}", res);
        res
    }

    /// Builds a TXN_AUTHR_AGRMT request. Request to add a new version of Transaction Author Agreement to the ledger.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
    ///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
    /// text: (Optional) a content of the TTA.
    ///             Mandatory in case of adding a new TAA. An existing TAA text can not be changed.
    ///             for Indy Node version <= 1.12.0:
    ///                 Use empty string to reset TAA on the ledger
    ///             for Indy Node version > 1.12.0
    ///                 Should be omitted in case of updating an existing TAA (setting `retirement_ts`)
    /// version: a version of the TTA (unique UTF-8 string).
    /// ratification_ts: (Optional) the date (timestamp) of TAA ratification by network government. (-1 to omit)
    ///              for Indy Node version <= 1.12.0:
    ///                 Must be omitted
    ///              for Indy Node version > 1.12.0:
    ///                 Must be specified in case of adding a new TAA
    ///                 Can be omitted in case of updating an existing TAA
    /// retirement_ts: (Optional) the date (timestamp) of TAA retirement. (-1 to omit)
    ///              for Indy Node version <= 1.12.0:
    ///                 Must be omitted
    ///              for Indy Node version > 1.12.0:
    ///                 Must be omitted in case of adding a new (latest) TAA.
    ///                 Should be used for updating (deactivating) non-latest TAA on the ledger.
    ///
    /// Note: Use `indy_build_disable_all_txn_author_agreements_request` to disable all TAA's on the ledger.
    ///
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_txn_author_agreement_request(
        &self,
        submitter_did: DidValue,
        text: Option<String>,
        version: String,
        ratification_ts: Option<u64>,
        retirement_ts: Option<u64>,
    ) -> IndyResult<String> {
        debug!(
            "build_txn_author_agreement_request > submitter_did {:?} \
                text {:?} version {:?} ratification_ts {:?} \
                retirement_ts {:?}",
            submitter_did, text, version, ratification_ts, retirement_ts
        );

        self.crypto_service.validate_did(&submitter_did)?;

        let res = self.ledger_service.build_txn_author_agreement_request(
            &submitter_did,
            text.as_deref(),
            &version,
            ratification_ts,
            retirement_ts,
        )?;

        let res = Ok(res);
        debug!("build_txn_author_agreement_request < {:?}", res);
        res
    }

    /// Builds a DISABLE_ALL_TXN_AUTHR_AGRMTS request. Request to disable all Transaction Author Agreement on the ledger.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
    ///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_disable_all_txn_author_agreements_request(
        &self,
        submitter_did: DidValue,
    ) -> IndyResult<String> {
        debug!(
            "build_disable_all_txn_author_agreements_request > submitter_did {:?}",
            submitter_did
        );

        self.crypto_service.validate_did(&submitter_did)?;

        let res = self
            .ledger_service
            .build_disable_all_txn_author_agreements_request(&submitter_did)?;

        let res = Ok(res);

        debug!(
            "build_disable_all_txn_author_agreements_request < {:?}",
            res
        );

        res
    }

    /// Builds a GET_TXN_AUTHR_AGRMT request. Request to get a specific Transaction Author Agreement from the ledger.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
    /// data: (Optional) specifies a condition for getting specific TAA.
    /// Contains 3 mutually exclusive optional fields:
    /// {
    ///     hash: Optional<str> - hash of requested TAA,
    ///     version: Optional<str> - version of requested TAA.
    ///     timestamp: Optional<u64> - ledger will return TAA valid at requested timestamp.
    /// }
    /// Null data or empty JSON are acceptable here. In this case, ledger will return the latest version of TAA.
    ///
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_get_txn_author_agreement_request(
        &self,
        submitter_did: Option<DidValue>,
        data: Option<GetTxnAuthorAgreementData>,
    ) -> IndyResult<String> {
        debug!(
            "build_get_txn_author_agreement_request > submitter_did {:?} data {:?}",
            submitter_did, data
        );

        self._validate_opt_did(submitter_did.as_ref())?;

        let res = self
            .ledger_service
            .build_get_txn_author_agreement_request(submitter_did.as_ref(), data.as_ref())?;

        let res = Ok(res);
        debug!("build_get_txn_author_agreement_request < {:?}", res);
        res
    }

    /// Builds a SET_TXN_AUTHR_AGRMT_AML request. Request to add a new list of acceptance mechanisms for transaction author agreement.
    /// Acceptance Mechanism is a description of the ways how the user may accept a transaction author agreement.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
    ///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
    /// aml: a set of new acceptance mechanisms:
    /// {
    ///     “<acceptance mechanism label 1>”: { acceptance mechanism description 1},
    ///     “<acceptance mechanism label 2>”: { acceptance mechanism description 2},
    ///     ...
    /// }
    /// version: a version of new acceptance mechanisms. (Note: unique on the Ledger)
    /// aml_context: (Optional) common context information about acceptance mechanisms (may be a URL to external resource).
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_acceptance_mechanisms_request(
        &self,
        submitter_did: DidValue,
        aml: AcceptanceMechanisms,
        version: String,
        aml_context: Option<String>,
    ) -> IndyResult<String> {
        debug!(
            "build_acceptance_mechanisms_request > submitter_did {:?} \
                aml {:?} version {:?} aml_context {:?}",
            submitter_did, aml, version, aml_context
        );

        self.crypto_service.validate_did(&submitter_did)?;

        let res = self.ledger_service.build_acceptance_mechanisms_request(
            &submitter_did,
            aml,
            &version,
            aml_context.as_deref(),
        )?;

        let res = Ok(res);
        debug!("build_acceptance_mechanisms_request < {:?}", res);
        res
    }

    /// Builds a GET_TXN_AUTHR_AGRMT_AML request. Request to get a list of  acceptance mechanisms from the ledger
    /// valid for specified time or the latest one.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
    /// timestamp: i64 - time to get an active acceptance mechanisms. Pass -1 to get the latest one.
    /// version: (Optional) version of acceptance mechanisms.
    /// cb: Callback that takes command result as parameter.
    ///
    /// NOTE: timestamp and version cannot be specified together.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn build_get_acceptance_mechanisms_request(
        &self,
        submitter_did: Option<DidValue>,
        timestamp: Option<u64>,
        version: Option<String>,
    ) -> IndyResult<String> {
        debug!(
            "build_get_acceptance_mechanisms_request > submitter_did {:?} \
                timestamp {:?} version {:?}",
            submitter_did, timestamp, version
        );

        self._validate_opt_did(submitter_did.as_ref())?;

        let res = self
            .ledger_service
            .build_get_acceptance_mechanisms_request(
                submitter_did.as_ref(),
                timestamp,
                version.as_deref(),
            )?;

        let res = Ok(res);
        debug!("build_get_acceptance_mechanisms_request < {:?}", res);
        res
    }

    /// Append transaction author agreement acceptance data to a request.
    /// This function should be called before signing and sending a request
    /// if there is any transaction author agreement set on the Ledger.
    ///
    /// EXPERIMENTAL
    ///
    /// This function may calculate digest by itself or consume it as a parameter.
    /// If all text, version and taa_digest parameters are specified, a check integrity of them will be done.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// request_json: original request data json.
    /// text and version - (optional) raw data about TAA from ledger.
    ///     These parameters should be passed together.
    ///     These parameters are required if taa_digest parameter is omitted.
    /// taa_digest - (optional) digest on text and version.
    ///     Digest is sha256 hash calculated on concatenated strings: version || text.
    ///     This parameter is required if text and version parameters are omitted.
    /// mechanism - mechanism how user has accepted the TAA
    /// time - UTC timestamp when user has accepted the TAA. Note that the time portion will be discarded to avoid a privacy risk.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Updated request result as json.
    ///
    /// #Errors
    /// Common*
    pub fn append_txn_author_agreement_acceptance_to_request(
        &self,
        request_json: String,
        text: Option<String>,
        version: Option<String>,
        taa_digest: Option<String>,
        acc_mech_type: String,
        time: u64,
    ) -> IndyResult<String> {
        debug!(
            "append_txn_author_agreement_acceptance_to_request > request_json {:?} \
                text {:?} version {:?} taa_digest {:?} acc_mech_type {:?} time {:?}",
            request_json, text, version, taa_digest, acc_mech_type, time
        );

        let mut request: Request<serde_json::Value> =
            serde_json::from_str(&request_json).map_err(|err| {
                err_msg(
                    IndyErrorKind::InvalidStructure,
                    format!("Unable to parse indy transaction. Err: {:?}", err),
                )
            })?;

        self.ledger_service
            .append_txn_author_agreement_acceptance_to_request(
                &mut request,
                text.as_deref(),
                version.as_deref(),
                taa_digest.as_deref(),
                &acc_mech_type,
                time,
            )?;

        let res = Ok(json!(request).to_string());

        debug!(
            "append_txn_author_agreement_acceptance_to_request < {:?}",
            res
        );

        res
    }

    /// Append Endorser to an existing request.
    ///
    /// An author of request still is a `DID` used as a `submitter_did` parameter for the building of the request.
    /// But it is expecting that the transaction will be sent by the specified Endorser.
    ///
    /// Note: Both Transaction Author and Endorser must sign output request after that.
    ///
    /// More about Transaction Endorser: https://github.com/hyperledger/indy-node/blob/master/design/transaction_endorser.md
    ///                                  https://github.com/hyperledger/indy-sdk/blob/master/docs/configuration.md
    ///
    /// #Params
    /// request_json: original request
    /// endorser_did: DID of the Endorser that will submit the transaction.
    ///               The Endorser's DID must be present on the ledger.
    /// cb: Callback that takes command result as parameter.
    ///     The command result is a request JSON with Endorser field appended.
    ///
    /// #Errors
    /// Common*
    pub fn append_request_endorser(
        &self,
        request_json: String,
        endorser_did: DidValue,
    ) -> IndyResult<String> {
        debug!(
            "append_request_endorser > request_json {:?} endorser_did {:?}",
            request_json, endorser_did
        );

        self.crypto_service.validate_did(&endorser_did)?;

        let endorser_did = endorser_did.to_short();

        let mut request: Request<serde_json::Value> =
            serde_json::from_str(&request_json).map_err(|err| {
                err_msg(
                    IndyErrorKind::InvalidStructure,
                    format!("Unable to parse indy transaction. Err: {:?}", err),
                )
            })?;

        self.ledger_service
            .append_txn_endorser(&mut request, &endorser_did)?;

        let res = Ok(json!(request).to_string());

        debug!("append_request_endorser < {:?}", res);
        res
    }

    fn _validate_opt_did(&self, did: Option<&DidValue>) -> IndyResult<()> {
        match did {
            Some(did) => Ok(self.crypto_service.validate_did(did)?),
            None => Ok(()),
        }
    }

    async fn _sign_request(
        &self,
        wallet_handle: WalletHandle,
        submitter_did: &DidValue,
        request_json: &str,
        signature_type: SignatureType,
    ) -> IndyResult<String> {
        debug!(
            "_sign_request > wallet_handle {:?} submitter_did {:?} request_json {:?}",
            wallet_handle, submitter_did, request_json
        );

        let my_did: Did = self
            .wallet_service
            .get_indy_object(wallet_handle, &submitter_did.0, &RecordOptions::id_value())
            .await?;

        let my_key: Key = self
            .wallet_service
            .get_indy_object(wallet_handle, &my_did.verkey, &RecordOptions::id_value())
            .await?;

        let (txn_bytes_to_sign, mut request) =
            self.ledger_service.get_txn_bytes_to_sign(&request_json)?;

        let signature = self
            .crypto_service
            .sign(&my_key, &txn_bytes_to_sign)
            .await?;

        let did = my_did.did.to_short();

        match signature_type {
            SignatureType::Single => {
                request["signature"] = Value::String(signature.to_base58());
            }
            SignatureType::Multi => {
                request.as_object_mut().map(|request| {
                    if !request.contains_key("signatures") {
                        request.insert(
                            "signatures".to_string(),
                            Value::Object(serde_json::Map::new()),
                        );
                    }
                    request["signatures"]
                        .as_object_mut()
                        .unwrap()
                        .insert(did.0, Value::String(signature.to_base58()));

                    if let (Some(identifier), Some(signature)) = (
                        request
                            .get("identifier")
                            .and_then(Value::as_str)
                            .map(str::to_owned),
                        request.remove("signature"),
                    ) {
                        request["signatures"]
                            .as_object_mut()
                            .unwrap()
                            .insert(identifier, signature);
                    }
                });
            }
        }

        let res: String = serde_json::to_string(&request).to_indy(
            IndyErrorKind::InvalidState,
            "Can't serialize message after signing",
        )?;

        let res = Ok(res);
        debug!("_sign_request < {:?}", res);
        res
    }

    async fn _submit_request<'a>(
        &self,
        handle: PoolHandle,
        request_json: &str,
    ) -> IndyResult<String> {
        debug!(
            "_submit_request > handle {:?} request_json {:?}",
            handle, request_json
        );

        serde_json::from_str::<Request<serde_json::Value>>(&request_json)
            .to_indy(IndyErrorKind::InvalidStructure, "Request is invalid json")?;

        let res = self.pool_service.send_tx(handle, request_json).await?;

        let res = Ok(res);
        debug!("_submit_request < {:?}", res);
        res
    }
}
