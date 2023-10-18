mod status;
use derive_more::From;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use status::Status;

use self::status::StatusContent;
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
        }
    }
}

transit_to_aries_msg!(StatusContent, Pickup);

into_msg_with_type!(Status, PickupTypeV2_0, Status);
