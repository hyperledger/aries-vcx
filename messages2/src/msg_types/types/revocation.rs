use std::marker::PhantomData;

use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use super::Protocol;
use crate::msg_types::actor::Actor;

#[derive(Copy, Clone, Debug, From, PartialEq, MessageType)]
#[msg_type(protocol = "revocation_notification")]
pub enum Revocation {
    V2(RevocationV2),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(Revocation, Protocol)))]
#[msg_type(major = 2)]
pub enum RevocationV2 {
    #[msg_type(minor = 0, actors = "Actor::Holder, Actor::Issuer")]
    V2_0(PhantomData<RevocationV2_0Kind>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum RevocationV2_0Kind {
    Revoke,
    Ack,
}

// #[cfg(test)]
// mod tests {
//     use super::RevocationV2_0;
//     use crate::misc::test_utils;

//     const PROTOCOL: &str = "https://didcomm.org/revocation_notification/2.0";
//     const VERSION_RESOLUTION_PROTOCOL: &str = "https://didcomm.org/revocation_notification/2.255";
//     const UNSUPPORTED_VERSION_PROTOCOL: &str = "https://didcomm.org/revocation_notification/3.0";

//     const KIND_REVOKE: &str = "revoke";
//     const KIND_ACK: &str = "ack";

//     #[test]
//     fn test_protocol_revocation_notification() {
//         test_utils::test_protocol(PROTOCOL, RevocationV2_0)
//     }

//     #[test]
//     fn test_version_resolution_revocation_notification() {
//         test_utils::test_protocol(VERSION_RESOLUTION_PROTOCOL, RevocationV2_0)
//     }

//     #[test]
//     #[should_panic]
//     fn test_unsupported_version_revocation_notification() {
//         test_utils::test_protocol(UNSUPPORTED_VERSION_PROTOCOL, RevocationV2_0)
//     }

//     #[test]
//     fn test_msg_type_revoke() {
//         test_utils::test_msg_type(PROTOCOL, KIND_REVOKE, RevocationV2_0)
//     }

//     #[test]
//     fn test_msg_type_ack() {
//         test_utils::test_msg_type(PROTOCOL, KIND_ACK, RevocationV2_0)
//     }
// }
