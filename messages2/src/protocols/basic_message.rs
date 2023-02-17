use messages_macros::Message;
use serde::{Deserialize, Serialize};

use crate::message_type::message_family::{
    basic_message::{BasicMessage as BasicMessageKind, BasicMessageV1, BasicMessageV1_0},
};

use super::traits::ConcreteMessage;

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "BasicMessageKind::V1(BasicMessageV1::V1_0(BasicMessageV1_0::Message))")]
pub struct BasicMessage {
    pub field: String
}