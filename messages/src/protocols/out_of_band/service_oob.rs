use crate::protocols::connection::did::Did;
use diddoc::aries::service::AriesService;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ServiceOob {
    AriesService(AriesService),
    Did(Did),
}
