use derive_more::From;
use serde::{Deserialize, Serialize};

use crate::message_type::message_family::{
    revocation::{Revocation as RevocationKind, RevocationV1, RevocationV1_0},
    traits::{ConcreteMessage, DelayedSerde},
};

#[derive(From)]
pub enum Revocation {
    Revoke(Revoke),
    Ack(Ack),
}

impl DelayedSerde for Revocation {
    type Seg = RevocationKind;

    fn delayed_deserialize<'de, D>(seg: Self::Seg, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let RevocationKind::V1(major) = seg;
        let RevocationV1::V1_0(minor) = major;

        match minor {
            RevocationV1_0::Revoke => Revoke::deserialize(deserializer).map(From::from),
            RevocationV1_0::Ack => Ack::deserialize(deserializer).map(From::from),
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
            Self::Revoke(v) => v.delayed_serialize(state, closure),
            Self::Ack(v) => v.delayed_serialize(state, closure),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Revoke;

impl ConcreteMessage for Revoke {
    type Kind = RevocationV1_0;

    fn kind() -> Self::Kind {
        Self::Kind::Revoke
    }
}

#[derive(Deserialize, Serialize)]
pub struct Ack;

impl ConcreteMessage for Ack {
    type Kind = RevocationV1_0;

    fn kind() -> Self::Kind {
        Self::Kind::Ack
    }
}
