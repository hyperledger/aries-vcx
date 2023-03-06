mod disclose;
mod query;

use derive_more::From;
use serde::{Deserializer, Serializer};

use crate::{
    composite_message::{transit_to_aries_msg, Message},
    delayed_serde::DelayedSerde,
    message_type::message_family::discover_features::{
        DiscoverFeatures as DiscoverFeaturesKind, DiscoverFeaturesV1, DiscoverFeaturesV1_0,
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
    type MsgType = DiscoverFeaturesKind;

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let DiscoverFeaturesKind::V1(major) = msg_type;
        let DiscoverFeaturesV1::V1_0(minor) = major;

        match minor {
            DiscoverFeaturesV1_0::Query => Query::delayed_deserialize(minor, deserializer).map(From::from),
            DiscoverFeaturesV1_0::Disclose => Disclose::delayed_deserialize(minor, deserializer).map(From::from),
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
