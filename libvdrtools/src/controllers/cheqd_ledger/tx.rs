use crate::controllers::CheqdLedgerController;
use indy_api_types::errors::IndyResult;

impl CheqdLedgerController {
    pub(crate) fn cheqd_build_query_get_tx_by_hash(
        &self,
        hash: &str,
    ) -> IndyResult<String> {
        trace!(
            "cheqd_build_query_get_tx_by_hash > hash {:?}",
            hash,
        );
        let query = self
            .cheqd_ledger_service
            .build_query_get_tx_by_hash(hash)?;
        trace!("cheqd_build_query_get_nym < {:?}", query);
        Ok(query)
    }

    pub(crate) fn cheqd_parse_query_get_tx_by_hash_resp(&self, resp_json: &str) -> IndyResult<String> {
        trace!("cheqd_parse_query_get_tx_by_hash_resp > resp {:?}", resp_json);
        let result = self.cheqd_ledger_service.cheqd_parse_query_get_tx_by_hash_resp(resp_json)?;
        trace!("cheqd_parse_query_get_tx_by_hash_resp < {:?}", result);
        Ok(result)
    }

    pub(crate) fn tx_build_query_simulate(&self, tx: &[u8]) -> IndyResult<String> {
        trace!("tx_build_query_simulate > tx {:?}", tx);

        let query = self
            .cheqd_ledger_service
            .tx_build_query_simulate(tx)?;

        trace!("tx_build_query_simulate < {:?}", query);
        Ok(query)
    }

    pub(crate) fn tx_parse_query_simulate_resp(
        &self,
        resp_json: &str,
    ) -> IndyResult<String> {
        trace!(
            "tx_parse_query_simulate_resp > resp {:?}",
            resp_json
        );
        let result = self.cheqd_ledger_service.tx_parse_query_simulate_resp(resp_json)?;
        trace!("tx_parse_query_simulate_resp < {:?}", result);
        Ok(result)
    }
}
