use derive_more::From;
use serde::{Deserialize, Deserializer, Serialize};

use crate::message_type::message_family::{
    traits::{ConcreteMessage, DelayedSerde},
    trust_ping::{TrustPing as TrustPingKind, TrustPingV1, TrustPingV1_0},
};

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
        M: serde::ser::SerializeMap,
        F: FnMut(&'a mut M) -> S,
        S: serde::Serializer,
        S::Error: From<M::Error>,
    {
        match self {
            Self::Ping(v) => v.delayed_serialize(state, closure),
            Self::PingResponse(v) => v.delayed_serialize(state, closure),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ping;

impl ConcreteMessage for Ping {
    type Kind = TrustPingV1_0;

    fn kind() -> Self::Kind {
        Self::Kind::Ping
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PingResponse;

impl ConcreteMessage for PingResponse {
    type Kind = TrustPingV1_0;

    fn kind() -> Self::Kind {
        Self::Kind::PingResponse
    }
}
