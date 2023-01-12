use async_std::sync::RwLockReadGuard;

use crate::controllers::vdr::{ledger::Ledger, VDRController, VDR};
use crate::domain::id::FullyQualifiedId;
use indy_api_types::errors::*;

impl VDRController {
    /// Prepare transaction to submit DID on the Ledger.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params

    /// vdr: pointer to VDR object
    /// txn_specific_params: DID transaction specific data.
    ///                      Depends on the Ledger type:
    ///     Indy:
    ///         {
    ///             dest: string - Target DID as base58-encoded string.
    ///             verkey: Optional<string> - Target identity verification key as base58-encoded string.
    ///             alias: Optional<string> DID's alias.
    ///             role: Optional<string> Role of a user DID record:
    ///                             null (common USER)
    ///                             TRUSTEE
    ///                             STEWARD
    ///                             TRUST_ANCHOR
    ///                             ENDORSER - equal to TRUST_ANCHOR that will be removed soon
    ///                             NETWORK_MONITOR
    ///                             empty string to reset role
    ///         }
    ///     Cheqd: TBD
    /// submitter_did: Fully-qualified DID of the transaction author as base58-encoded string.
    /// endorser: DID of the Endorser that will endorse the transaction.
    ///           The Endorser's DID must be present on the ledger with 'ENDORSER' role.
    ///
    /// #Returns
    /// namespace: Ledger namespace to submit transaction (captured from submitter DID)
    /// txn_bytes: prepared transaction as bytes
    /// signature_spec: type of the signature transaction must be signed with (one of: `Ed25519` or `Secp256k1`)
    /// bytes_to_sign: bytes must be signed
    /// endorsement_spec: endorsement process specification
    pub(crate) async fn prepare_did_txn(
        &self,
        vdr: &VDR,
        txn_params: String,
        submitter_did: String,
        endorser: Option<String>,
    ) -> IndyResult<(String, Vec<u8>, String, Vec<u8>, Option<String>)> {
        trace!(
            "prepare_did_txn > txn_params {:?} submitter_did {:?} endorser {:?}",
            txn_params,
            submitter_did,
            endorser,
        );
        let (ledger, parsed_id) = vdr.resolve_ledger_for_id(&submitter_did).await?;

        let (txn_bytes, bytes_to_sign) = ledger
            .build_did_request(&txn_params, &submitter_did, endorser.as_deref())
            .await?;
        self.build_prepare_txn_result(
            ledger,
            parsed_id,
            txn_bytes,
            bytes_to_sign,
            endorser.as_deref(),
        )
    }

    /// Prepare transaction to submit Schema on the Ledger.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params

    /// handle: handle pointing to created VDR object (returned by vdr_create)
    /// txn_specific_params: Schema transaction specific data
    ///                      Depends on the Ledger type:
    ///     Indy:
    ///         {
    ///             id: identifier of schema
    ///             attrNames: array of attribute name strings (the number of attributes should be less or equal than 125)
    ///             name: Schema's name string
    ///             version: Schema's version string,
    ///             ver: Version of the Schema json
    ///         }
    /// submitter_did: Fully-qualified DID of the transaction author as base58-encoded string.
    /// endorser: DID of the Endorser that will endorse the transaction.
    ///           The Endorser's DID must be present on the ledger with 'ENDORSER' role.
    ///
    /// #Returns
    /// namespace: Ledger namespace to submit transaction (captured from submitter DID)
    /// txn_bytes: prepared transaction as bytes
    /// signature_spec: type of the signature transaction must be signed with (one of: `Ed25519` or `Secp256k1`)
    /// bytes_to_sign: bytes must be signed
    /// endorsement_spec: endorsement process specification
    pub(crate) async fn prepare_schema_txn(
        &self,
        vdr: &VDR,
        txn_params: String,
        submitter_did: String,
        endorser: Option<String>,
    ) -> IndyResult<(String, Vec<u8>, String, Vec<u8>, Option<String>)> {
        trace!(
            "prepare_schema_txn > txn_params {:?} submitter_did {:?} endorser {:?}",
            txn_params,
            submitter_did,
            endorser,
        );
        let (ledger, parsed_id) = vdr.resolve_ledger_for_id(&submitter_did).await?;

        let (txn_bytes, bytes_to_sign) = ledger
            .build_schema_request(&txn_params, &submitter_did, endorser.as_deref())
            .await?;
        self.build_prepare_txn_result(
            ledger,
            parsed_id,
            txn_bytes,
            bytes_to_sign,
            endorser.as_deref(),
        )
    }

    /// Prepare transaction to submit Credential Definition on the Ledger.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params

    /// vdr: pointer to VDR object
    /// txn_specific_params: CredDef transaction specific data
    ///                      Depends on the Ledger type:
    ///     Indy:
    ///         {
    ///             id: string - identifier of credential definition
    ///             schemaId: string - identifier of stored in ledger schema
    ///             type: string - type of the credential definition. CL is the only supported type now.
    ///             tag: string - allows to distinct between credential definitions for the same issuer and schema
    ///             value: Dictionary with Credential Definition's data: {
    ///                 primary: primary credential public key,
    ///                 Optional<revocation>: revocation credential public key
    ///             },
    ///             ver: Version of the CredDef json
    ///         }
    /// submitter_did: Fully-qualified DID of the transaction author as base58-encoded string.
    /// endorser: DID of the Endorser that will endorse the transaction.
    ///           The Endorser's DID must be present on the ledger with 'ENDORSER' role.
    ///
    /// #Returns
    /// namespace: Ledger namespace to submit transaction (captured from submitter DID)
    /// txn_bytes: prepared transaction as bytes
    /// signature_spec: type of the signature transaction must be signed with (one of: `Ed25519` or `Secp256k1`)
    /// bytes_to_sign: bytes must be signed
    /// endorsement_spec: endorsement process specification
    pub(crate) async fn prepare_creddef_txn(
        &self,
        vdr: &VDR,
        txn_params: String,
        submitter_did: String,
        endorser: Option<String>,
    ) -> IndyResult<(String, Vec<u8>, String, Vec<u8>, Option<String>)> {
        trace!(
            "prepare_creddef_txn > txn_params {:?} submitter_did {:?} endorser {:?}",
            txn_params,
            submitter_did,
            endorser,
        );
        let (ledger, parsed_id) = vdr.resolve_ledger_for_id(&submitter_did).await?;

        let (txn_bytes, bytes_to_sign) = ledger
            .build_cred_def_request(&txn_params, &submitter_did, endorser.as_deref())
            .await?;
        self.build_prepare_txn_result(
            ledger,
            parsed_id,
            txn_bytes,
            bytes_to_sign,
            endorser.as_deref(),
        )
    }

    fn build_prepare_txn_result(
        &self,
        ledger: RwLockReadGuard<(dyn Ledger + 'static)>,
        id: FullyQualifiedId,
        txn_bytes: Vec<u8>,
        bytes_to_sign: Vec<u8>,
        endorser: Option<&str>,
    ) -> IndyResult<(String, Vec<u8>, String, Vec<u8>, Option<String>)> {
        trace!(
            "build_prepare_txn_result > id {:?} txn_bytes {:?} bytes_to_sign {:?} endorser {:?}",
            id,
            txn_bytes,
            bytes_to_sign,
            endorser,
        );
        let namespace = id.namespace();
        let signature_spec = id.did_method.signature_type().to_string();
        let endorsement_spec = ledger.prepare_endorsement_spec(endorser)?;
        let endorsement_spec =
            endorsement_spec.map(|endorsement_spec| json!(endorsement_spec).to_string());
        Ok((
            namespace,
            txn_bytes,
            signature_spec,
            bytes_to_sign,
            endorsement_spec,
        ))
    }
}
