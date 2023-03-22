use std::marker::PhantomData;

use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use super::Protocol;
use crate::msg_types::role::Role;

#[derive(Copy, Clone, Debug, From, PartialEq, MessageType)]
#[msg_type(protocol = "basicmessage")]
pub enum BasicMessage {
    V1(BasicMessageV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(BasicMessage, Protocol))]
#[msg_type(major = 1)]
pub enum BasicMessageV1 {
    #[msg_type(minor = 0, roles = "Role::Receiver, Role::Sender")]
    V1_0(PhantomData<fn() -> BasicMessageV1_0>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum BasicMessageV1_0 {
    Message,
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::BasicMessageV1;
    use crate::misc::test_utils;

    const PROTOCOL: &str = "https://didcomm.org/basicmessage/1.0";
    const VERSION_RESOLUTION_PROTOCOL: &str = "https://didcomm.org/basicmessage/1.255";
    const UNSUPPORTED_VERSION_PROTOCOL: &str = "https://didcomm.org/basicmessage/2.0";

    const KIND_MESSAGE: &str = "message";

    #[test]
    fn test_protocol_basic_message() {
        test_utils::test_protocol(PROTOCOL, BasicMessageV1::V1_0(PhantomData))
    }

    #[test]
    fn test_version_resolution_basic_message() {
        test_utils::test_protocol(VERSION_RESOLUTION_PROTOCOL, BasicMessageV1::V1_0(PhantomData))
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_basic_message() {
        test_utils::test_protocol(UNSUPPORTED_VERSION_PROTOCOL, BasicMessageV1::V1_0(PhantomData))
    }

    #[test]
    fn test_msg_type_message() {
        test_utils::test_msg_type(PROTOCOL, KIND_MESSAGE, BasicMessageV1::V1_0(PhantomData))
    }
}
