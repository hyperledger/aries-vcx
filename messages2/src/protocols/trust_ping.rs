use derive_more::From;
use messages_macros::Message;
use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeMap};

use crate::{message_type::message_family::{
    trust_ping::{TrustPing as TrustPingKind, TrustPingV1, TrustPingV1_0},
}, delayed_serde::DelayedSerde};

use super::traits::ConcreteMessage;

#[derive(Clone, Debug, From)]
pub enum TrustPing {
    Ping(Ping),
    PingResponse(PingResponse),
}

impl DelayedSerde for TrustPing {
    type MsgType = TrustPingKind;

    fn delayed_deserialize<'de, D>(seg: Self::MsgType, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let TrustPingKind::V1(major) = seg;
        let TrustPingV1::V1_0(minor) = major;

        match minor {
            TrustPingV1_0::Ping => Ping::deserialize(deserializer).map(From::from),
            TrustPingV1_0::PingResponse => PingResponse::deserialize(deserializer).map(From::from),
        }
    }

    fn delayed_serialize<'a, M, F, S>(&self, state: &'a mut M, closure: &mut F) -> Result<S::Ok, S::Error>
    where
        M: SerializeMap,
        F: FnMut(&'a mut M) -> S,
        S: Serializer,
        S::Error: From<M::Error>,
    {
        match self {
            Self::Ping(v) => v.delayed_serialize(state, closure),
            Self::PingResponse(v) => v.delayed_serialize(state, closure),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "TrustPingV1_0::PingResponse")]
pub struct Ping;

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "TrustPingV1_0::PingResponse")]
pub struct PingResponse;
