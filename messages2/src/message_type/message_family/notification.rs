use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use crate::{
    error::{MsgTypeError, MsgTypeResult},
    message_type::actor::Actor,
    message_type::{registry::get_supported_version, MessageType},
};

use super::{
    traits::{ResolveMajorVersion, ResolveMinorVersion, ResolveMsgKind},
    MessageFamily,
};

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(MessageFamily, MessageType))]
#[semver(family = "notification")]
pub enum Notification {
    V1(NotificationV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(Notification, MessageFamily, MessageType)))]
#[semver(major = 1, parent = "Notification", actors(Actor::Notified, Actor::Notifier))]
pub enum NotificationV1 {
    V1_0(NotificationV1_0),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(NotificationV1, Notification, MessageFamily, MessageType)))]
#[semver(minor = 0, parent = "NotificationV1")]
#[strum(serialize_all = "kebab-case")]
pub enum NotificationV1_0 {
    Ack,
}
