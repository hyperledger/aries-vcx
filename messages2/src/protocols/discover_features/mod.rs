mod disclose;
mod query;

use std::str::FromStr;

use derive_more::From;
use serde::{de::Error, Deserializer, Serializer};

use crate::{
    composite_message::{transit_to_aries_msg, Message},
    delayed_serde::DelayedSerde,
    message_type::message_protocol::discover_features::{
        DiscoverFeatures as DiscoverFeaturesKind, DiscoverFeaturesV1, DiscoverFeaturesV1_0, DiscoverFeaturesV1_0Kind,
    },
};

use self::{
    disclose::{DiscloseContent, DiscloseDecorators},
    query::{QueryContent, QueryDecorators},
};

pub use self::{disclose::Disclose, query::Query};

#[derive(Clone, Debug, From)]
pub enum DiscoverFeatures {
    Query(Query),
    Disclose(Disclose),
}

impl DelayedSerde for DiscoverFeatures {
    type MsgType<'a> = (DiscoverFeaturesKind, &'a str);

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType<'de>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (major, kind) = msg_type;
        let DiscoverFeaturesKind::V1(major) = major;
        let DiscoverFeaturesV1::V1_0(minor) = major;
        let kind = DiscoverFeaturesV1_0Kind::from_str(kind).map_err(D::Error::custom)?;

        match kind {
            DiscoverFeaturesV1_0Kind::Query => Query::delayed_deserialize(kind, deserializer).map(From::from),
            DiscoverFeaturesV1_0Kind::Disclose => Disclose::delayed_deserialize(kind, deserializer).map(From::from),
        }
    }

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Query(v) => v.delayed_serialize(serializer),
            Self::Disclose(v) => v.delayed_serialize(serializer),
        }
    }
}

transit_to_aries_msg!(QueryContent: QueryDecorators, DiscoverFeatures);
transit_to_aries_msg!(DiscloseContent: DiscloseDecorators, DiscoverFeatures);
