use std::marker::PhantomData;

use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use super::Protocol;
use crate::msg_types::role::Role;

#[derive(Copy, Clone, Debug, From, PartialEq, MessageType)]
#[msg_type(protocol = "out-of-band")]
pub enum OutOfBand {
    V1(OutOfBandV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(OutOfBand, Protocol))]
#[msg_type(major = 1)]
pub enum OutOfBandV1 {
    #[msg_type(minor = 0, actors = "Role::Receiver, Role::Sender")]
    V1_1(PhantomData<fn() -> OutOfBandV1_1>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum OutOfBandV1_1 {
    Invitation,
    HandshakeReuse,
    HandshakeReuseAccepted,
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::OutOfBandV1;
    use crate::misc::test_utils;

    const PROTOCOL: &str = "https://didcomm.org/out-of-band/1.1";
    const VERSION_RESOLUTION_PROTOCOL: &str = "https://didcomm.org/out-of-band/1.255";
    const UNSUPPORTED_VERSION_PROTOCOL: &str = "https://didcomm.org/out-of-band/2.0";

    const KIND_INVITATION: &str = "invitation";
    const KIND_REUSE: &str = "handshake-reuse";
    const KIND_REUSE_ACC: &str = "handshake-reuse-accepted";

    #[test]
    fn test_protocol_out_of_band() {
        test_utils::test_protocol(PROTOCOL, OutOfBandV1::V1_1(PhantomData))
    }

    #[test]
    fn test_version_resolution_out_of_band() {
        test_utils::test_protocol(VERSION_RESOLUTION_PROTOCOL, OutOfBandV1::V1_1(PhantomData))
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_out_of_band() {
        test_utils::test_protocol(UNSUPPORTED_VERSION_PROTOCOL, OutOfBandV1::V1_1(PhantomData))
    }

    #[test]
    fn test_msg_type_invitation() {
        test_utils::test_msg_type(PROTOCOL, KIND_INVITATION, OutOfBandV1::V1_1(PhantomData))
    }

    #[test]
    fn test_msg_type_reuse() {
        test_utils::test_msg_type(PROTOCOL, KIND_REUSE, OutOfBandV1::V1_1(PhantomData))
    }

    #[test]
    fn test_msg_type_reuse_acc() {
        test_utils::test_msg_type(PROTOCOL, KIND_REUSE_ACC, OutOfBandV1::V1_1(PhantomData))
    }
}
