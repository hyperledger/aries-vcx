use aries_vcx::indy::ledger::pool::test_utils::create_tmp_genesis_txn_file;
use aries_vcx_agent::{Agent, InitConfig, PoolInitConfig, WalletInitConfig};

fn _agent_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn _enterprise_seed() -> String {
    "000000000000000000000000Trustee1".to_string()
}

fn _service_endpoint() -> String {
    format!("http://localhost:8081/didcomm")
}

fn _pool_config(agent_id: &str) -> PoolInitConfig {
    PoolInitConfig {
        genesis_path: create_tmp_genesis_txn_file(),
        pool_name: format!("pool_{}", agent_id),
    }
}

fn _wallet_config(agent_id: &str) -> WalletInitConfig {
    WalletInitConfig {
        wallet_name: format!("rust_agent_{}", agent_id),
        wallet_key: "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY".to_string(),
        wallet_kdf: "RAW".to_string(),
    }
}

fn _init_config() -> InitConfig {
    let agent_id = _agent_id();
    InitConfig {
        enterprise_seed: _enterprise_seed(),
        pool_config: _pool_config(&agent_id),
        wallet_config: _wallet_config(&agent_id),
        agency_config: None,
        service_endpoint: _service_endpoint(),
    }
}

pub async fn initialize_agent() -> Agent {
    Agent::initialize(_init_config()).await.unwrap()
}
