mod disclose;
mod query;

use derive_more::From;
use serde::{Deserializer, Serializer};

use crate::{
    composite_message::Message,
    delayed_serde::DelayedSerde,
    message_type::message_family::discover_features::{
        DiscoverFeatures as DiscoverFeaturesKind, DiscoverFeaturesV1, DiscoverFeaturesV1_0,
    },
};

use self::{
    disclose::{Disclose, DiscloseDecorators},
    query::{Query, QueryDecorators},
};

#[derive(Clone, Debug, From)]
pub enum DiscoverFeatures {
    Query(Message<Query, QueryDecorators>),
    Disclose(Message<Disclose, DiscloseDecorators>),
}

impl DelayedSerde for DiscoverFeatures {
    type MsgType = DiscoverFeaturesKind;

    fn delayed_deserialize<'de, D>(seg: Self::MsgType, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let DiscoverFeaturesKind::V1(major) = seg;
        let DiscoverFeaturesV1::V1_0(minor) = major;

        match minor {
            DiscoverFeaturesV1_0::Query => {
                Message::<Query, QueryDecorators>::delayed_deserialize(minor, deserializer).map(From::from)
            }
            DiscoverFeaturesV1_0::Disclose => {
                Message::<Disclose, DiscloseDecorators>::delayed_deserialize(minor, deserializer).map(From::from)
            }
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
