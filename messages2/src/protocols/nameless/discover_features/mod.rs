//! Module containing the `discover features` protocol messages, as defined in the [RFC](https://github.com/hyperledger/aries-rfcs/blob/main/features/0031-discover-features/README.md).

pub mod disclose;
pub mod query;

use derive_more::From;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use self::{
    disclose::{Disclose, DiscloseContent, DiscloseDecorators},
    query::{Query, QueryContent, QueryDecorators},
};
use crate::{
    maybe_known::MaybeKnown,
    misc::utils::{transit_to_aries_msg, into_msg_with_type},
    msg_types::{
        types::discover_features::{
            DiscoverFeaturesProtocol as DiscoverFeaturesKind, DiscoverFeaturesProtocolV1, DiscoverFeaturesProtocolV1_0,
        },
        Protocol, Role, MsgWithType, traits::ProtocolVersion,
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
        let (major, kind_str) = msg_type;

        let kind = match major {
            DiscoverFeaturesKind::V1(DiscoverFeaturesProtocolV1::V1_0(pd)) => DiscoverFeaturesProtocolV1::kind(pd, kind_str)
        };

        match kind.map_err(D::Error::custom)? {
            DiscoverFeaturesProtocolV1_0::Query => Query::deserialize(deserializer).map(From::from),
            DiscoverFeaturesProtocolV1_0::Disclose => Disclose::deserialize(deserializer).map(From::from),
        }
    }

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Query(v) => MsgWithType::from(v).serialize(serializer),
            Self::Disclose(v) => MsgWithType::from(v).serialize(serializer),
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

into_msg_with_type!(Query, DiscoverFeaturesProtocolV1_0, Query);
into_msg_with_type!(Disclose, DiscoverFeaturesProtocolV1_0, Disclose);