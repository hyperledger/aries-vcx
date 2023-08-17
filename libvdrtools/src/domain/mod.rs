pub mod anoncreds;
pub mod cache;
pub mod crypto;
pub mod ledger;
pub mod pairwise;

use indy_api_types::validation::Validatable;

#[derive(Debug, Serialize, Deserialize)]
pub struct IndyConfig {
    pub crypto_thread_pool_size: Option<usize>,
    pub collect_backtrace: Option<bool>,
}

impl Validatable for IndyConfig {}
