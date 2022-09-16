use indy_sys::PoolHandle;

use crate::did_doc::service_aries::AriesService;
use crate::error::prelude::*;
use crate::libindy::utils::ledger;
use crate::messages::connection::did::Did;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ServiceResolvable {
    AriesService(AriesService),
    Did(Did),
}

impl ServiceResolvable {
    pub async fn resolve(&self, pool_handle: PoolHandle) -> VcxResult<AriesService> {
        match self {
            ServiceResolvable::AriesService(service) => Ok(service.clone()),
            ServiceResolvable::Did(did) => ledger::get_service(pool_handle, did).await,
        }
    }
}
