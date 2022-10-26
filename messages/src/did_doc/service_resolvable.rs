use crate::connection::did::Did;
use crate::did_doc::service_aries::AriesService;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ServiceResolvable {
    AriesService(AriesService),
    Did(Did),
}
