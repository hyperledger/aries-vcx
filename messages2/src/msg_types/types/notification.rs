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
#[semver(protocol = "notification")]
pub enum Notification {
    V1(NotificationV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(Notification, Protocol)))]
#[semver(major = 1, parent = "Notification", actors(Actor::Notified, Actor::Notifier))]
pub enum NotificationV1 {
    V1_0(NotificationV1_0),
}

#[derive(Copy, Clone, Debug, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(NotificationV1, Notification, Protocol)))]
#[semver(minor = 0, parent = "NotificationV1")]
pub struct NotificationV1_0;

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, MessageType)]
#[strum(serialize_all = "kebab-case")]
#[semver(parent = "NotificationV1_0")]
pub enum NotificationV1_0Kind {
    Ack,
}

#[cfg(test)]
mod tests {
    use crate::misc::test_utils;

    use super::NotificationV1_0;

    const PROTOCOL: &str = "https://didcomm.org/notification/1.0";
    const VERSION_RESOLUTION_PROTOCOL: &str = "https://didcomm.org/notification/1.255";
    const UNSUPPORTED_VERSION_PROTOCOL: &str = "https://didcomm.org/notification/2.0";

    const KIND_ACK: &str = "ack";

    #[test]
    fn test_protocol_notification() {
        test_utils::test_protocol(PROTOCOL, NotificationV1_0)
    }

    #[test]
    fn test_version_resolution_discover_features() {
        test_utils::test_protocol(VERSION_RESOLUTION_PROTOCOL, NotificationV1_0)
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_discover_features() {
        test_utils::test_protocol(UNSUPPORTED_VERSION_PROTOCOL, NotificationV1_0)
    }

    #[test]
    fn test_msg_type_ack() {
        test_utils::test_msg_type(PROTOCOL, KIND_ACK, NotificationV1_0)
    }
}
