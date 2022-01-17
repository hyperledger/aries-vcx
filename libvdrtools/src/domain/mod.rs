pub mod anoncreds;
pub mod crypto;
pub mod ledger;
pub mod pairwise;
pub mod pool;
pub mod cache;
#[cfg(feature = "cheqd")]
pub mod cheqd_keys;
#[cfg(feature = "cheqd")]
pub mod cheqd_pool;
#[cfg(feature = "cheqd")]
pub mod cheqd_ledger;
pub mod id;
pub mod vdr;

use indy_api_types::validation::Validatable;

#[derive(Debug, Serialize, Deserialize)]
pub struct IndyConfig {
    pub crypto_thread_pool_size: Option<usize>,
    pub collect_backtrace: Option<bool>,
    pub freshness_threshold: Option<u64>
}

impl Validatable for IndyConfig {}