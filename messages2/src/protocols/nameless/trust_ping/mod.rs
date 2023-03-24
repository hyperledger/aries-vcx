//! Module containing the `trust ping` protocol messages, as defined in the [RFC](https://github.com/hyperledger/aries-rfcs/blob/main/features/0048-trust-ping/README.md).

pub mod ping;
pub mod ping_response;

use derive_more::From;
use serde::{de::Error, Deserialize, Deserializer, Serializer, Serialize};

use self::{
    ping::{Ping, PingContent, PingDecorators},
    ping_response::{PingResponse, PingResponseContent, PingResponseDecorators},
};
use crate::{
    misc::utils::{into_msg_with_type, transit_to_aries_msg},
    msg_types::{
        traits::ProtocolVersion,
        types::trust_ping::{TrustPingProtocol as TrustPingKind, TrustPingProtocolV1, TrustPingProtocolV1_0}, MsgWithType,
    },
    protocols::traits::DelayedSerde,
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
        let (major, kind_str) = msg_type;

        let kind = match major {
            TrustPingKind::V1(TrustPingProtocolV1::V1_0(pd)) => TrustPingProtocolV1::kind(pd, kind_str),
        };

        match kind.map_err(D::Error::custom)? {
            TrustPingProtocolV1_0::Ping => Ping::deserialize(deserializer).map(From::from),
            TrustPingProtocolV1_0::PingResponse => PingResponse::deserialize(deserializer).map(From::from),
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

into_msg_with_type!(Ping, TrustPingProtocolV1_0, Ping);
into_msg_with_type!(PingResponse, TrustPingProtocolV1_0, PingResponse);
