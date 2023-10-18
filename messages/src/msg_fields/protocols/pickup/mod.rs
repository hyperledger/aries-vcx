mod decorators;
mod delivery;
mod delivery_request;
mod live_delivery_change;
mod messages_received;
mod status;
mod status_request;
use derive_more::From;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use self::{
    decorators::PickupDecoratorsCommon,
    delivery::{Delivery, DeliveryContent},
    delivery_request::{DeliveryRequest, DeliveryRequestContent},
    live_delivery_change::{LiveDeliveryChange, LiveDeliveryChangeContent},
    messages_received::{MessagesReceived, MessagesReceivedContent},
    status::{Status, StatusContent},
    status_request::{StatusRequest, StatusRequestContent},
};
use crate::{
    misc::utils::{into_msg_with_type, transit_to_aries_msg},
    msg_fields::traits::DelayedSerde,
    msg_types::{
        protocols::pickup::{PickupType, PickupTypeV2, PickupTypeV2_0},
        MsgWithType,
    },
};

#[derive(Clone, Debug, From, PartialEq)]
pub enum Pickup {
    Status(Status),
    StatusRequest(StatusRequest),
    DeliveryRequest(DeliveryRequest),
    Delivery(Delivery),
    MessagesReceived(MessagesReceived),
    LiveDeliveryChange(LiveDeliveryChange),
}

impl DelayedSerde for Pickup {
    type MsgType<'a> = (PickupType, &'a str);

    fn delayed_deserialize<'de, D>(
        msg_type: Self::MsgType<'de>,
        deserializer: D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (protocol, kind_str) = msg_type;

        let kind = match protocol {
            PickupType::V2(PickupTypeV2::V2_0(kind)) => kind.kind_from_str(kind_str),
        };

        match kind.map_err(D::Error::custom)? {
            PickupTypeV2_0::Status => Status::deserialize(deserializer).map(From::from),
            _ => todo!(),
        }
    }

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Status(v) => MsgWithType::from(v).serialize(serializer),
            Self::StatusRequest(v) => MsgWithType::from(v).serialize(serializer),
            Self::Delivery(v) => MsgWithType::from(v).serialize(serializer),
            Self::DeliveryRequest(v) => MsgWithType::from(v).serialize(serializer),
            Self::MessagesReceived(v) => MsgWithType::from(v).serialize(serializer),
            Self::LiveDeliveryChange(v) => MsgWithType::from(v).serialize(serializer),
        }
    }
}

transit_to_aries_msg!(StatusContent: PickupDecoratorsCommon, Pickup);
transit_to_aries_msg!(StatusRequestContent: PickupDecoratorsCommon, Pickup);
transit_to_aries_msg!(DeliveryContent: PickupDecoratorsCommon, Pickup);
transit_to_aries_msg!(DeliveryRequestContent: PickupDecoratorsCommon, Pickup);
transit_to_aries_msg!(MessagesReceivedContent: PickupDecoratorsCommon, Pickup);
transit_to_aries_msg!(LiveDeliveryChangeContent: PickupDecoratorsCommon, Pickup);

into_msg_with_type!(Status, PickupTypeV2_0, Status);
into_msg_with_type!(StatusRequest, PickupTypeV2_0, StatusRequest);
into_msg_with_type!(Delivery, PickupTypeV2_0, Delivery);
into_msg_with_type!(DeliveryRequest, PickupTypeV2_0, DeliveryRequest);
into_msg_with_type!(MessagesReceived, PickupTypeV2_0, MessagesReceived);
into_msg_with_type!(LiveDeliveryChange, PickupTypeV2_0, LiveDeliveryChange);
