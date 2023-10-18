use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::msg_parts::MsgParts;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct PickupStatusReqMsg {
    #[serde(default)]
    pub auth_pubkey: String,
    pub recipient_key: Option<String>,
}
