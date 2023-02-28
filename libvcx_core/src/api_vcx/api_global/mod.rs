pub mod agency_client;
pub mod ledger;
pub mod pool;
pub mod profile;
pub mod settings;
pub mod state;
pub mod wallet;

use crate::api_vcx::utils::version_constants;

lazy_static! {
    pub static ref VERSION_STRING: String = format!("{}{}", version_constants::VERSION, version_constants::REVISION);
}
