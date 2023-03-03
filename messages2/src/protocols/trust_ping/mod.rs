mod ping;
mod ping_response;

use derive_more::From;
use serde::{Deserializer, Serializer};

use crate::{
    composite_message::Message,
    delayed_serde::DelayedSerde,
    message_type::message_family::trust_ping::{TrustPing as TrustPingKind, TrustPingV1, TrustPingV1_0},
};

use self::{
    ping::{Ping, PingDecorators},
    ping_response::{PingResponse, PingResponseDecorators},
};

#[derive(Clone, Debug, From)]
pub enum TrustPing {
    Ping(Message<Ping, PingDecorators>),
    PingResponse(Message<PingResponse, PingResponseDecorators>),
}

impl DelayedSerde for TrustPing {
    type MsgType = TrustPingKind;

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let TrustPingKind::V1(major) = msg_type;
        let TrustPingV1::V1_0(minor) = major;

        match minor {
            TrustPingV1_0::Ping => {
                Message::<Ping, PingDecorators>::delayed_deserialize(minor, deserializer).map(From::from)
            }
            TrustPingV1_0::PingResponse => {
                Message::<PingResponse, PingResponseDecorators>::delayed_deserialize(minor, deserializer)
                    .map(From::from)
            }
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
