use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct UnpackMessageOutput {
    pub message: String,
    pub recipient_verkey: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_verkey: Option<String>,
}
