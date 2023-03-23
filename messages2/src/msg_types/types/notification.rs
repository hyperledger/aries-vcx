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
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    #[test]
    fn test_protocol_notification() {
        test_utils::test_serde(
            Protocol::from(NotificationV1::new_v1_0()),
            json!("https://didcomm.org/notification/1.0"),
        )
    }

    #[test]
    fn test_version_resolution_discover_features() {
        test_utils::test_msg_type_resolution(
            "https://didcomm.org/notification/1.255",
            NotificationV1::new_v1_0(),
        )
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_discover_features() {
        test_utils::test_serde(
            Protocol::from(NotificationV1::new_v1_0()),
            json!("https://didcomm.org/notification/2.0"),
        )
    }

    #[test]
    fn test_msg_type_ack() {
        test_utils::test_msg_type("https://didcomm.org/notification/1.0", "ack", NotificationV1::new_v1_0())
    }
}
