use serde::{Deserialize, Serialize};

use crate::message_type::message_family::{
    basic_message::{BasicMessage as BasicMessageKind, BasicMessageV1, BasicMessageV1_0},
};

use super::traits::ConcreteMessage;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BasicMessage {
    pub field: String
}

impl ConcreteMessage for BasicMessage {
    type Kind = BasicMessageKind;

    fn kind() -> Self::Kind {
        Self::Kind::V1(BasicMessageV1::V1_0(BasicMessageV1_0::Message))
    }
}
