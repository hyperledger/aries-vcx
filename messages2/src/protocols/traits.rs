use std::fmt::Debug;

use crate::message_type::MessageType;

pub trait MessageKind {
    type Kind: Into<MessageType> + PartialEq + Debug;

    fn kind() -> Self::Kind;
}
