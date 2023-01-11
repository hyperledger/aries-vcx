use async_std::sync::RwLockReadGuard;

use indy_api_types::{errors::*};
use crate::domain::id::FullyQualifiedId;
use crate::controllers::vdr::{VDRController, ledger::Ledger, VDR};

impl VDRController {
    pub(crate) async fn prepare_did_txn(&self,
                                        vdr: &VDR,
                                        txn_params: String,
                                        submitter_did: String,
                                        endorser: Option<String>,
    ) -> IndyResult<(String, Vec<u8>, String, Vec<u8>, Option<String>)> {
        trace!(
            "prepare_did_txn > txn_params {:?} submitter_did {:?} endorser {:?}",
            txn_params, submitter_did, endorser,
        );
        let (ledger, parsed_id) = vdr.resolve_ledger_for_id(&submitter_did).await?;

        let (txn_bytes, bytes_to_sign) = ledger.build_did_request(&txn_params, &submitter_did, endorser.as_deref()).await?;
        self.build_prepare_txn_result(ledger, parsed_id, txn_bytes, bytes_to_sign, endorser.as_deref())
    }

    pub(crate) async fn prepare_schema_txn(&self,
                                           vdr: &VDR,
                                           txn_params: String,
                                           submitter_did: String,
                                           endorser: Option<String>,
    ) -> IndyResult<(String, Vec<u8>, String, Vec<u8>, Option<String>)> {
        trace!(
            "prepare_schema_txn > txn_params {:?} submitter_did {:?} endorser {:?}",
            txn_params, submitter_did, endorser,
        );
        let (ledger, parsed_id) = vdr.resolve_ledger_for_id(&submitter_did).await?;

        let (txn_bytes, bytes_to_sign) = ledger.build_schema_request(&txn_params, &submitter_did, endorser.as_deref()).await?;
        self.build_prepare_txn_result(ledger, parsed_id, txn_bytes, bytes_to_sign, endorser.as_deref())
    }

    pub(crate) async fn prepare_creddef_txn(&self,
                                            vdr: &VDR,
                                            txn_params: String,
                                            submitter_did: String,
                                            endorser: Option<String>,
    ) -> IndyResult<(String, Vec<u8>, String, Vec<u8>, Option<String>)> {
        trace!(
            "prepare_creddef_txn > txn_params {:?} submitter_did {:?} endorser {:?}",
            txn_params, submitter_did, endorser,
        );
        let (ledger, parsed_id) = vdr.resolve_ledger_for_id(&submitter_did).await?;

        let (txn_bytes, bytes_to_sign) = ledger.build_cred_def_request(&txn_params, &submitter_did, endorser.as_deref()).await?;
        self.build_prepare_txn_result(ledger, parsed_id, txn_bytes, bytes_to_sign, endorser.as_deref())
    }

    fn build_prepare_txn_result(&self,
                                ledger: RwLockReadGuard<(dyn Ledger + 'static)>,
                                id: FullyQualifiedId,
                                txn_bytes: Vec<u8>,
                                bytes_to_sign: Vec<u8>,
                                endorser: Option<&str>) -> IndyResult<(String, Vec<u8>, String, Vec<u8>, Option<String>)> {
        trace!(
            "build_prepare_txn_result > id {:?} txn_bytes {:?} bytes_to_sign {:?} endorser {:?}",
            id, txn_bytes, bytes_to_sign, endorser,
        );
        let namespace = id.namespace();
        let signature_spec = id.did_method.signature_type().to_string();
        let endorsement_spec = ledger.prepare_endorsement_spec(endorser)?;
        let endorsement_spec = endorsement_spec.map(|endorsement_spec| json!(endorsement_spec).to_string());
        Ok((namespace, txn_bytes, signature_spec, bytes_to_sign, endorsement_spec))
    }
}
