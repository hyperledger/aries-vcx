use indyrs::{cheqd_pool, future::Future, IndyError};

pub fn add(alias: &str, rpc_address: &str, chain_id: &str, mode: Option<&str>) -> Result<String, IndyError> {
    let config = json!({
        "rpc_address": rpc_address,
        "chain_id": chain_id,
        "pool_mode": mode.unwrap_or("Persistent"),
    }).to_string();
    cheqd_pool::add(alias, &config).wait()
}

pub fn get_config(alias: &str) -> Result<String, IndyError> {
    cheqd_pool::get_config(alias).wait()
}

pub fn get_all_config() -> Result<String, IndyError> {
    cheqd_pool::get_all_config().wait()
}

pub fn broadcast_tx_commit(pool_alias: &str, signed_tx: &[u8]) -> Result<String, IndyError> {
    cheqd_pool::broadcast_tx_commit(pool_alias, signed_tx).wait()
}

pub fn abci_query(pool_alias: &str, req_json: &str) -> Result<String, IndyError> {
    cheqd_pool::abci_query(pool_alias, req_json).wait()
}

pub fn abci_info(pool_alias: &str) -> Result<String, IndyError> {
    cheqd_pool::abci_info(pool_alias).wait()
}
