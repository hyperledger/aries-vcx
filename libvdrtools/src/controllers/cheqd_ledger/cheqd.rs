use crate::controllers::CheqdLedgerController;
use indy_api_types::errors::prelude::*;
use indy_api_types::WalletHandle;
use crate::domain::crypto::did::Did;
use crate::domain::crypto::key::Key;
use indy_wallet::RecordOptions;

impl CheqdLedgerController {
    pub(crate) async fn sign_cheqd_request(
        &self,
        wallet_handle: WalletHandle,
        request_bytes: &[u8],
        did: &str,
    ) -> IndyResult<Vec<u8>> {
        trace!(
            "sign_cheqd_request > request_bytes {:?} did {:?}",
            request_bytes,
            did
        );

        let my_did: Did = self
            .wallet_service
            .get_indy_object(wallet_handle, &did, &RecordOptions::id_value())
            .await?;
        let my_key: Key = self
            .wallet_service
            .get_indy_object(wallet_handle, &my_did.verkey, &RecordOptions::id_value())
            .await?;

        let signature = self.crypto_service.sign(&my_key, request_bytes).await?;

        self.cheqd_ledger_service.build_signed_message(request_bytes, did, &signature)
    }

    pub(crate) async fn cheqd_build_msg_create_did(
        &self,
        did: &str,
        verkey: &str,
    ) -> IndyResult<Vec<u8>> {
        trace!(
            "cheqd_build_msg_create_did > did {:?} verkey {:?} ",
            did,
            verkey,
        );
        let msg = self
            .cheqd_ledger_service
            .cheqd_build_msg_create_did(did, verkey)?;
        trace!("cheqd_build_msg_create_did < {:?}", msg);

        Ok(msg)
    }

    pub(crate) fn cheqd_parse_msg_create_did_resp(&self, resp: &str) -> IndyResult<String> {
        trace!("cheqd_parse_msg_create_did_resp > resp {:?}", resp);
        let res = self.cheqd_ledger_service.cheqd_parse_msg_create_did_resp(&resp)?;
        trace!("cheqd_parse_msg_create_did_resp < {:?}", res);
        Ok(res)
    }

    pub(crate) async fn cheqd_build_msg_update_did(
        &self,
        did: &str,
        verkey: &str,
        version_id: &str,
    ) -> IndyResult<Vec<u8>> {
        trace!(
            "cheqd_build_msg_update_did > creator {:?} verkey {:?} version_id {:?}",
            did,
            verkey,
            version_id
        );
        let msg = self
            .cheqd_ledger_service
            .cheqd_build_msg_update_did(did, verkey, version_id)?;
        trace!("cheqd_build_msg_update_did < {:?}", msg);

        Ok(msg)
    }

    pub(crate) fn cheqd_parse_msg_update_did_resp(&self, resp: &str) -> IndyResult<String> {
        trace!("cheqd_parse_msg_update_did_resp > resp {:?}", resp);
        let res = self.cheqd_ledger_service.cheqd_parse_msg_update_did_resp(&resp)?;
        trace!("cheqd_parse_msg_update_did_resp < {:?}", res);
        Ok(res)
    }

    pub(crate) fn cheqd_build_query_get_did(&self, did: &str) -> IndyResult<String> {
        trace!("cheqd_build_query_get_did > id {:?}", did);
        let query = self.cheqd_ledger_service.cheqd_build_query_get_did(did)?;
        trace!("cheqd_build_query_get_did < {:?}", query);
        Ok(query)
    }

    pub(crate) fn cheqd_parse_query_get_did_resp(&self, resp_json: &str) -> IndyResult<String> {
        trace!("cheqd_parse_query_get_did_resp > resp {:?}", resp_json);
        let json_result = self.cheqd_ledger_service.cheqd_parse_query_get_did_resp(&resp_json)?;
        trace!("cheqd_parse_query_get_did_resp < {:?}", json_result);
        Ok(json_result)
    }
}
