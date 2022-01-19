#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PoolMode {
    InMemory,
    Persistent,
}

impl Default for PoolMode {
    fn default() -> Self {
        PoolMode::Persistent
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddPoolConfig {
    pub rpc_address: String,
    pub chain_id: String,
    #[serde(default)]
    pub pool_mode: PoolMode,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PoolConfig {
    pub alias: String,
    pub rpc_address: String,
    pub chain_id: String,
}

impl PoolConfig {
    pub fn new(alias: String, rpc_address: String, chain_id: String) -> Self {
        PoolConfig {
            alias,
            rpc_address,
            chain_id,
        }
    } 
}