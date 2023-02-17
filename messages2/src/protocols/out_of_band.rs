use derive_more::From;
use messages_macros::Message;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{message_type::message_family::{
    out_of_band::{OutOfBand as OutOfBandKind, OutOfBandV1, OutOfBandV1_1},
}, delayed_serde::DelayedSerde};

use super::traits::ConcreteMessage;

#[derive(Clone, Debug, From)]
pub enum OutOfBand {
    Invitation(Invitation),
    HandshakeReuse(HandshakeReuse),
    HandshakeReuseAccepted(HandshakeReuseAccepted),
}

impl DelayedSerde for OutOfBand {
    type MsgType = OutOfBandKind;

    fn delayed_deserialize<'de, D>(seg: Self::MsgType, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let OutOfBandKind::V1(major) = seg;
        let OutOfBandV1::V1_1(minor) = major;

        match minor {
            OutOfBandV1_1::Invitation => Invitation::deserialize(deserializer).map(From::from),
            OutOfBandV1_1::HandshakeReuse => HandshakeReuse::deserialize(deserializer).map(From::from),
            OutOfBandV1_1::HandshakeReuseAccepted => HandshakeReuseAccepted::deserialize(deserializer).map(From::from),
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
            Self::Invitation(v) => v.delayed_serialize(state, closure),
            Self::HandshakeReuse(v) => v.delayed_serialize(state, closure),
            Self::HandshakeReuseAccepted(v) => v.delayed_serialize(state, closure),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "OutOfBandV1_1::Invitation")]
pub struct Invitation;

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "OutOfBandV1_1::HandshakeReuse")]
pub struct HandshakeReuse;

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "OutOfBandV1_1::HandshakeReuseAccepted")]
pub struct HandshakeReuseAccepted;
