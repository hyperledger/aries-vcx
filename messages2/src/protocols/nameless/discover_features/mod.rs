//! Module containing the `discover features` protocol messages, as defined in the [RFC](https://github.com/hyperledger/aries-rfcs/blob/main/features/0031-discover-features/README.md).

mod disclose;
mod query;

use std::str::FromStr;

use derive_more::From;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

pub use self::{disclose::Disclose, query::Query};
use self::{
    disclose::{DiscloseContent, DiscloseDecorators},
    query::{QueryContent, QueryDecorators},
};
use crate::{
    maybe_known::MaybeKnown,
    misc::utils::transit_to_aries_msg,
    msg_types::{
        types::discover_features::{
            DiscoverFeatures as DiscoverFeaturesKind, DiscoverFeaturesV1, DiscoverFeaturesV1_0,
        },
        Protocol, Role,
    },
    protocols::traits::DelayedSerde,
};

#[derive(Clone, Debug, From, PartialEq)]
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
        let DiscoverFeaturesV1::V1_0(_minor) = major;
        let kind = DiscoverFeaturesV1_0::from_str(kind).map_err(D::Error::custom)?;

        match kind {
            DiscoverFeaturesV1_0::Query => Query::delayed_deserialize(kind, deserializer).map(From::from),
            DiscoverFeaturesV1_0::Disclose => Disclose::delayed_deserialize(kind, deserializer).map(From::from),
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ProtocolDescriptor {
    pub pid: MaybeKnown<Protocol>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<MaybeKnown<Role>>>,
}

impl ProtocolDescriptor {
    pub fn new(pid: MaybeKnown<Protocol>) -> Self {
        Self { pid, roles: None }
    }
}

transit_to_aries_msg!(QueryContent: QueryDecorators, DiscoverFeatures);
transit_to_aries_msg!(DiscloseContent: DiscloseDecorators, DiscoverFeatures);
