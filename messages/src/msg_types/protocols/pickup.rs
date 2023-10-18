use derive_more::{From, TryInto};
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::Transitive;

use super::Protocol;
use crate::msg_types::MsgKindType;

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, MessageType)]
#[msg_type(protocol = "messagepickup")]
pub enum PickupType {
    V2(PickupTypeV2),
}

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, Transitive, MessageType)]
#[transitive(into(PickupType, Protocol))]
#[msg_type(major = 2)]
pub enum PickupTypeV2 {
    #[msg_type(minor = 0, roles = "")]
    V2_0(MsgKindType<PickupTypeV2_0>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum PickupTypeV2_0 {
    Status,
    StatusRequest,
    DeliveryRequest,
    Delivery,
    MessageReceived,
    LiveDeliveryChange,
}
