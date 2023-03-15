mod ack;
mod notification;

use std::str::FromStr;

use derive_more::From;
use serde::{de::Error, Deserializer, Serializer};

pub use self::{ack::AckRevoke, notification::Revoke};
use self::{
    ack::AckRevokeContent,
    notification::{RevokeContent, RevokeDecorators},
};
use super::notification::AckDecorators;
use crate::{
    misc::utils::transit_to_aries_msg,
    msg_types::types::revocation::{Revocation as RevocationKind, RevocationV2, RevocationV2_0Kind},
    protocols::traits::DelayedSerde,
};

#[derive(Clone, Debug, From, PartialEq)]
pub enum Revocation {
    Revoke(Revoke),
    Ack(AckRevoke),
}

impl DelayedSerde for Revocation {
    type MsgType<'a> = (RevocationKind, &'a str);

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType<'de>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (major, kind) = msg_type;
        let RevocationKind::V2(major) = major;
        let RevocationV2::V2_0(_minor) = major;
        let kind = RevocationV2_0Kind::from_str(kind).map_err(D::Error::custom)?;

        match kind {
            RevocationV2_0Kind::Revoke => Revoke::delayed_deserialize(kind, deserializer).map(From::from),
            RevocationV2_0Kind::Ack => AckRevoke::delayed_deserialize(kind, deserializer).map(From::from),
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

transit_to_aries_msg!(RevokeContent: RevokeDecorators, Revocation);
transit_to_aries_msg!(AckRevokeContent: AckDecorators, Revocation);
