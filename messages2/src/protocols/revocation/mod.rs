mod ack;
mod notification;

use derive_more::From;
use serde::{Deserialize, Deserializer, Serializer};

use crate::{
    delayed_serde::DelayedSerde,
    message_type::message_family::revocation::{Revocation as RevocationKind, RevocationV2, RevocationV2_0},
};

use self::{ack::AckRevoke, notification::Revoke};

#[derive(Clone, Debug, From)]
pub enum Revocation {
    Revoke(Revoke),
    Ack(AckRevoke),
}

impl DelayedSerde for Revocation {
    type MsgType = RevocationKind;

    fn delayed_deserialize<'de, D>(seg: Self::MsgType, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let RevocationKind::V2(major) = seg;
        let RevocationV2::V2_0(minor) = major;

        match minor {
            RevocationV2_0::Revoke => Revoke::deserialize(deserializer).map(From::from),
            RevocationV2_0::Ack => AckRevoke::deserialize(deserializer).map(From::from),
        }
    }

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Revoke(v) => v.delayed_serialize(serializer),
            Self::Ack(v) => v.delayed_serialize(serializer),
        }
    }
}
