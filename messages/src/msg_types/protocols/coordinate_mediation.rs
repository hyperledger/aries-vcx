use derive_more::{From, TryInto};
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::Transitive;

use super::Protocol;
use crate::msg_types::{MsgKindType, Role};

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, MessageType)]
#[msg_type(protocol = "coordinate-mediation")]
pub enum CoordinateMediationType {
    V1(CoordinateMediationTypeV1),
}

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, Transitive, MessageType)]
#[transitive(into(CoordinateMediationType, Protocol))]
#[msg_type(major = 1)]
pub enum CoordinateMediationTypeV1 {
    #[msg_type(minor = 0, roles = "Role::Mediator")]
    V1_0(MsgKindType<CoordinateMediationTypeV1_0>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum CoordinateMediationTypeV1_0 {
    MediateRequest,
    MediateDeny,
    MediateGrant,
    KeylistUpdate,
    KeylistUpdateResponse,
    KeylistQuery,
    Keylist,
}
