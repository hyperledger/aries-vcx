//! Module containing the `discover features` protocol messages, as defined in the [RFC](<https://github.com/hyperledger/aries-rfcs/blob/main/features/0031-discover-features/README.md>).

pub mod disclose;
pub mod query;

use derive_more::From;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use shared::maybe_known::MaybeKnown;
use typed_builder::TypedBuilder;

use self::{
    disclose::{Disclose, DiscloseContent, DiscloseDecorators},
    query::{Query, QueryContent, QueryDecorators},
};
use crate::{
    misc::utils::{into_msg_with_type, transit_to_aries_msg},
    msg_fields::traits::DelayedSerde,
    msg_types::{
        protocols::discover_features::{
            DiscoverFeaturesType as DiscoverFeaturesKind, DiscoverFeaturesTypeV1,
            DiscoverFeaturesTypeV1_0,
        },
        MsgWithType, Protocol, Role,
    },
};

#[derive(Clone, Debug, From, PartialEq)]
pub enum DiscoverFeatures {
    Query(Query),
    Disclose(Disclose),
}

impl DelayedSerde for DiscoverFeatures {
    type MsgType<'a> = (DiscoverFeaturesKind, &'a str);

    fn delayed_deserialize<'de, D>(
        msg_type: Self::MsgType<'de>,
        deserializer: D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (protocol, kind_str) = msg_type;

        let kind = match protocol {
            DiscoverFeaturesKind::V1(DiscoverFeaturesTypeV1::V1_0(kind)) => {
                kind.kind_from_str(kind_str)
            }
        };

        match kind.map_err(D::Error::custom)? {
            DiscoverFeaturesTypeV1_0::Query => Query::deserialize(deserializer).map(From::from),
            DiscoverFeaturesTypeV1_0::Disclose => {
                Disclose::deserialize(deserializer).map(From::from)
            }
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct ProtocolDescriptor {
    pub pid: MaybeKnown<Protocol>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<MaybeKnown<Role>>>,
}

transit_to_aries_msg!(QueryContent: QueryDecorators, DiscoverFeatures);
transit_to_aries_msg!(DiscloseContent: DiscloseDecorators, DiscoverFeatures);

into_msg_with_type!(Query, DiscoverFeaturesTypeV1_0, Query);
into_msg_with_type!(Disclose, DiscoverFeaturesTypeV1_0, Disclose);
