use crate::messages::did_doc::service_aries::AriesService;
use crate::messages::connection::did::Did;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ServiceResolvable {
    AriesService(AriesService),
    Did(Did),
}
