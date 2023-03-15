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
#[semver(protocol = "revocation_notification")]
pub enum Revocation {
    V2(RevocationV2),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(Revocation, Protocol)))]
#[semver(major = 2, parent = "Revocation", actors(Actor::Holder, Actor::Issuer))]
pub enum RevocationV2 {
    V2_0(RevocationV2_0),
}

#[derive(Copy, Clone, Debug, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(RevocationV2, Revocation, Protocol)))]
#[semver(minor = 0, parent = "RevocationV2")]
pub struct RevocationV2_0;

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, MessageType)]
#[strum(serialize_all = "kebab-case")]
#[semver(parent = "RevocationV2_0")]
pub enum RevocationV2_0Kind {
    Revoke,
    Ack,
}

#[cfg(test)]
mod tests {
    use crate::misc::test_utils;

    use super::RevocationV2_0;

    const PROTOCOL: &str = "https://didcomm.org/revocation_notification/2.0";
    const VERSION_RESOLUTION_PROTOCOL: &str = "https://didcomm.org/revocation_notification/2.255";
    const UNSUPPORTED_VERSION_PROTOCOL: &str = "https://didcomm.org/revocation_notification/3.0";

    const KIND_REVOKE: &str = "revoke";
    const KIND_ACK: &str = "ack";

    #[test]
    fn test_protocol_revocation_notification() {
        test_utils::test_protocol(PROTOCOL, RevocationV2_0)
    }

    #[test]
    fn test_version_resolution_revocation_notification() {
        test_utils::test_protocol(VERSION_RESOLUTION_PROTOCOL, RevocationV2_0)
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_revocation_notification() {
        test_utils::test_protocol(UNSUPPORTED_VERSION_PROTOCOL, RevocationV2_0)
    }

    #[test]
    fn test_msg_type_revoke() {
        test_utils::test_msg_type(PROTOCOL, KIND_REVOKE, RevocationV2_0)
    }

    #[test]
    fn test_msg_type_ack() {
        test_utils::test_msg_type(PROTOCOL, KIND_ACK, RevocationV2_0)
    }
}