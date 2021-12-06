use indyrs::{future::Future, cheqd_ledger, IndyError};

pub fn build_query_get_tx_by_hash(hash: &str) -> Result<String, IndyError> {
    cheqd_ledger::tx::build_query_get_tx_by_hash(hash).wait()
}

pub fn parse_query_get_tx_by_hash_resp(query_resp: &str) -> Result<String, IndyError> {
    cheqd_ledger::tx::parse_query_get_tx_by_hash_resp(query_resp).wait()
}

pub fn build_query_simulate(tx_raw: &Vec<u8>) -> Result<String, IndyError> {
    cheqd_ledger::tx::build_query_simulate(tx_raw).wait()
}

pub fn parse_query_simulate_resp(query_resp: &str) -> Result<String, IndyError> {
    cheqd_ledger::tx::parse_query_simulate_resp(query_resp).wait()
}
