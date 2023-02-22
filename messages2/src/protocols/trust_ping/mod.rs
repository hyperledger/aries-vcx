mod ping;
mod ping_response;

use derive_more::From;
use serde::{ser::SerializeMap, Deserialize, Deserializer, Serializer};

use crate::{
    delayed_serde::DelayedSerde,
    message_type::message_family::trust_ping::{TrustPing as TrustPingKind, TrustPingV1, TrustPingV1_0},
};

use self::{ping::Ping, ping_response::PingResponse};

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
