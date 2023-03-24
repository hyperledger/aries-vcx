//! Module containing the `trust ping` protocol messages, as defined in the [RFC](https://github.com/hyperledger/aries-rfcs/blob/main/features/0048-trust-ping/README.md).

pub mod ping;
pub mod ping_response;

use derive_more::From;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use self::{
    ping::{Ping, PingContent, PingDecorators},
    ping_response::{PingResponse, PingResponseContent, PingResponseDecorators},
};
use crate::{
    misc::utils::{into_msg_with_type, transit_to_aries_msg},
    msg_fields::traits::DelayedSerde,
    msg_types::{
        protocols::trust_ping::{TrustPingType as TrustPingKind, TrustPingTypeV1, TrustPingTypeV1_0},
        MsgWithType,
    },
};

#[derive(Clone, Debug, From, PartialEq)]
pub enum TrustPing {
    Ping(Ping),
    PingResponse(PingResponse),
}

impl DelayedSerde for TrustPing {
    type MsgType<'a> = (TrustPingKind, &'a str);

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType<'de>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (protocol, kind_str) = msg_type;

        let kind = match protocol {
            TrustPingKind::V1(TrustPingTypeV1::V1_0(kind)) => kind.kind_from_str(kind_str),
        };

        match kind.map_err(D::Error::custom)? {
            TrustPingTypeV1_0::Ping => Ping::deserialize(deserializer).map(From::from),
            TrustPingTypeV1_0::PingResponse => PingResponse::deserialize(deserializer).map(From::from),
        }
    }

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Ping(v) => MsgWithType::from(v).serialize(serializer),
            Self::PingResponse(v) => MsgWithType::from(v).serialize(serializer),
        }
    }
}

transit_to_aries_msg!(PingContent: PingDecorators, TrustPing);
transit_to_aries_msg!(PingResponseContent: PingResponseDecorators, TrustPing);

into_msg_with_type!(Ping, TrustPingTypeV1_0, Ping);
into_msg_with_type!(PingResponse, TrustPingTypeV1_0, PingResponse);
