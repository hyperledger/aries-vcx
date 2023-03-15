use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use super::{
    traits::{MajorVersion, MinorVersion, ProtocolName},
    Protocol,
};
use crate::msg_types::{actor::Actor, registry::get_supported_version};

#[derive(Copy, Clone, Debug, From, PartialEq, MessageType)]
#[semver(protocol = "out-of-band")]
pub enum OutOfBand {
    V1(OutOfBandV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(OutOfBand, Protocol)))]
#[semver(major = 1, parent = "OutOfBand", actors(Actor::Receiver, Actor::Sender))]
pub enum OutOfBandV1 {
    V1_1(OutOfBandV1_1),
}

#[derive(Copy, Clone, Debug, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(OutOfBandV1, OutOfBand, Protocol)))]
#[semver(minor = 1, parent = "OutOfBandV1")]
pub struct OutOfBandV1_1;

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, MessageType)]
#[strum(serialize_all = "kebab-case")]
#[semver(parent = "OutOfBandV1_1")]
pub enum OutOfBandV1_1Kind {
    Invitation,
    HandshakeReuse,
    HandshakeReuseAccepted,
}

#[cfg(test)]
mod tests {
    use crate::misc::test_utils;

    use super::OutOfBandV1_1;

    const PROTOCOL: &str = "https://didcomm.org/out-of-band/1.1";
    const VERSION_RESOLUTION_PROTOCOL: &str = "https://didcomm.org/out-of-band/1.255";
    const UNSUPPORTED_VERSION_PROTOCOL: &str = "https://didcomm.org/out-of-band/2.0";

    const KIND_INVITATION: &str = "invitation";
    const KIND_REUSE: &str = "handshake-reuse";
    const KIND_REUSE_ACC: &str = "handshake-reuse-accepted";

    #[test]
    fn test_protocol_out_of_band() {
        test_utils::test_protocol(PROTOCOL, OutOfBandV1_1)
    }

    #[test]
    fn test_version_resolution_out_of_band() {
        test_utils::test_protocol(VERSION_RESOLUTION_PROTOCOL, OutOfBandV1_1)
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_out_of_band() {
        test_utils::test_protocol(UNSUPPORTED_VERSION_PROTOCOL, OutOfBandV1_1)
    }

    #[test]
    fn test_msg_type_invitation() {
        test_utils::test_msg_type(PROTOCOL, KIND_INVITATION, OutOfBandV1_1)
    }

    #[test]
    fn test_msg_type_reuse() {
        test_utils::test_msg_type(PROTOCOL, KIND_REUSE, OutOfBandV1_1)
    }

    #[test]
    fn test_msg_type_reuse_acc() {
        test_utils::test_msg_type(PROTOCOL, KIND_REUSE_ACC, OutOfBandV1_1)
    }
}