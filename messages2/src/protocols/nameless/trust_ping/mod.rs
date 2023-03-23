//! Module containing the `trust ping` protocol messages, as defined in the [RFC](https://github.com/hyperledger/aries-rfcs/blob/main/features/0048-trust-ping/README.md).

mod ping;
mod ping_response;

use std::str::FromStr;

use derive_more::From;
use serde::{de::Error, Deserializer, Serializer};

pub use self::{
    ping::{Ping, PingContent, PingDecorators},
    ping_response::{PingResponse, PingResponseContent, PingResponseDecorators},
};
use crate::{
    misc::utils::transit_to_aries_msg,
    msg_types::types::trust_ping::{TrustPing as TrustPingKind, TrustPingV1, TrustPingV1_0},
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
        let (major, kind) = msg_type;
        let TrustPingKind::V1(major) = major;
        let TrustPingV1::V1_0(_minor) = major;
        let kind = TrustPingV1_0::from_str(kind).map_err(D::Error::custom)?;

        match kind {
            TrustPingV1_0::Ping => Ping::delayed_deserialize(kind, deserializer).map(From::from),
            TrustPingV1_0::PingResponse => PingResponse::delayed_deserialize(kind, deserializer).map(From::from),
        }
    }

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Ping(v) => v.delayed_serialize(serializer),
            Self::PingResponse(v) => v.delayed_serialize(serializer),
        }
    }
}

transit_to_aries_msg!(PingContent: PingDecorators, TrustPing);
transit_to_aries_msg!(PingResponseContent: PingResponseDecorators, TrustPing);
