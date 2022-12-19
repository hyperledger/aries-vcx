use crate::did_doc::service_aries::AriesService;
use crate::protocols::connection::did::Did;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ServiceResolvable {
    AriesService(AriesService),
    Did(Did),
}
