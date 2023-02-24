mod ping;
mod ping_response;

use derive_more::From;
use serde::{Deserialize, Deserializer, Serializer};

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
