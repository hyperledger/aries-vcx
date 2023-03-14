use diddoc::aries::service::AriesService;

use crate::protocols::connection::did::Did;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ServiceOob {
    AriesService(AriesService),
    Did(Did),
}
