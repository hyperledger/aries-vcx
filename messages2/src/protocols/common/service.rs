use diddoc::aries::service::AriesService;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Service {
    AriesService(AriesService),
    Did(String),
}
