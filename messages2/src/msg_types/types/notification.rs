use std::marker::PhantomData;

use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use super::Protocol;
use crate::msg_types::role::Role;

#[derive(Copy, Clone, Debug, From, PartialEq, MessageType)]
#[msg_type(protocol = "notification")]
pub enum Notification {
    V1(NotificationV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(Notification, Protocol))]
#[msg_type(major = 1)]
pub enum NotificationV1 {
    #[msg_type(minor = 0, roles = "Role::Notified, Role::Notifier")]
    V1_0(PhantomData<fn() -> NotificationV1_0>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum NotificationV1_0 {
    Ack,
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::NotificationV1;
    use crate::misc::test_utils;

    const PROTOCOL: &str = "https://didcomm.org/notification/1.0";
    const VERSION_RESOLUTION_PROTOCOL: &str = "https://didcomm.org/notification/1.255";
    const UNSUPPORTED_VERSION_PROTOCOL: &str = "https://didcomm.org/notification/2.0";

    const KIND_ACK: &str = "ack";

    #[test]
    fn test_protocol_notification() {
        test_utils::test_protocol(PROTOCOL, NotificationV1::V1_0(PhantomData))
    }

    #[test]
    fn test_version_resolution_discover_features() {
        test_utils::test_protocol(VERSION_RESOLUTION_PROTOCOL, NotificationV1::V1_0(PhantomData))
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_discover_features() {
        test_utils::test_protocol(UNSUPPORTED_VERSION_PROTOCOL, NotificationV1::V1_0(PhantomData))
    }

    #[test]
    fn test_msg_type_ack() {
        test_utils::test_msg_type(PROTOCOL, KIND_ACK, NotificationV1::V1_0(PhantomData))
    }
}
