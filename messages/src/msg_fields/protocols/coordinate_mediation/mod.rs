mod keylist;
mod keylist_query;
mod keylist_update;
mod keylist_update_response;
mod mediate_deny;
mod mediate_grant;
mod mediate_request;
use derive_more::From;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use self::{
    keylist::{Keylist, KeylistContent, KeylistDecorators},
    keylist_query::{KeylistQuery, KeylistQueryContent, KeylistQueryDecorators},
    keylist_update::{KeylistUpdate, KeylistUpdateContent, KeylistUpdateDecorators},
    keylist_update_response::{
        KeylistUpdateResponse, KeylistUpdateResponseContent, KeylistUpdateResponseDecorators,
    },
    mediate_deny::{MediateDeny, MediateDenyContent, MediateDenyDecorators},
    mediate_grant::{MediateGrant, MediateGrantContent, MediateGrantDecorators},
    mediate_request::{MediateRequest, MediateRequestContent, MediateRequestDecorators},
};
use crate::{
    misc::utils::{into_msg_with_type, transit_to_aries_msg},
    msg_fields::traits::DelayedSerde,
    msg_types::{
        protocols::coordinate_mediation::{
            CoordinateMediationType, CoordinateMediationTypeV1, CoordinateMediationTypeV1_0,
        },
        MsgWithType,
    },
};

#[derive(Clone, Debug, From, PartialEq)]
pub enum CoordinateMediation {
    MediateRequest(MediateRequest),
    MediateDeny(MediateDeny),
    MediateGrant(MediateGrant),
    KeylistUpdate(KeylistUpdate),
    KeylistUpdateResponse(KeylistUpdateResponse),
    KeylistQuery(KeylistQuery),
    Keylist(Keylist),
}

impl DelayedSerde for CoordinateMediation {
    type MsgType<'a> = (CoordinateMediationType, &'a str);

    fn delayed_deserialize<'de, D>(
        msg_type: Self::MsgType<'de>,
        deserializer: D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (protocol, kind_str) = msg_type;

        let kind = match protocol {
            CoordinateMediationType::V1(CoordinateMediationTypeV1::V1_0(kind)) => {
                kind.kind_from_str(kind_str)
            }
        };

        match kind.map_err(D::Error::custom)? {
            CoordinateMediationTypeV1_0::MediateRequest => {
                MediateRequest::deserialize(deserializer).map(From::from)
            }
            CoordinateMediationTypeV1_0::MediateDeny => {
                MediateDeny::deserialize(deserializer).map(From::from)
            }
            CoordinateMediationTypeV1_0::MediateGrant => {
                MediateGrant::deserialize(deserializer).map(From::from)
            }
            CoordinateMediationTypeV1_0::KeylistUpdate => {
                KeylistUpdate::deserialize(deserializer).map(From::from)
            }
            CoordinateMediationTypeV1_0::KeylistUpdateResponse => {
                KeylistUpdateResponse::deserialize(deserializer).map(From::from)
            }
            CoordinateMediationTypeV1_0::KeylistQuery => {
                KeylistQuery::deserialize(deserializer).map(From::from)
            }
            CoordinateMediationTypeV1_0::Keylist => {
                Keylist::deserialize(deserializer).map(From::from)
            }
        }
    }

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::MediateRequest(v) => MsgWithType::from(v).serialize(serializer),
            Self::MediateDeny(v) => MsgWithType::from(v).serialize(serializer),
            Self::MediateGrant(v) => MsgWithType::from(v).serialize(serializer),
            Self::KeylistUpdate(v) => MsgWithType::from(v).serialize(serializer),
            Self::KeylistUpdateResponse(v) => MsgWithType::from(v).serialize(serializer),
            Self::KeylistQuery(v) => MsgWithType::from(v).serialize(serializer),
            Self::Keylist(v) => MsgWithType::from(v).serialize(serializer),
        }
    }
}

transit_to_aries_msg!(MediateRequestContent: MediateRequestDecorators, CoordinateMediation);
transit_to_aries_msg!(MediateDenyContent: MediateDenyDecorators, CoordinateMediation);
transit_to_aries_msg!(MediateGrantContent: MediateGrantDecorators, CoordinateMediation);
transit_to_aries_msg!(KeylistUpdateContent: KeylistUpdateDecorators, CoordinateMediation);
transit_to_aries_msg!(KeylistUpdateResponseContent: KeylistUpdateResponseDecorators, CoordinateMediation);
transit_to_aries_msg!(KeylistQueryContent: KeylistQueryDecorators, CoordinateMediation);
transit_to_aries_msg!(KeylistContent: KeylistDecorators, CoordinateMediation);

into_msg_with_type!(MediateRequest, CoordinateMediationTypeV1_0, MediateRequest);
into_msg_with_type!(MediateDeny, CoordinateMediationTypeV1_0, MediateDeny);
into_msg_with_type!(MediateGrant, CoordinateMediationTypeV1_0, MediateGrant);
into_msg_with_type!(KeylistUpdate, CoordinateMediationTypeV1_0, KeylistUpdate);
into_msg_with_type!(
    KeylistUpdateResponse,
    CoordinateMediationTypeV1_0,
    KeylistUpdateResponse
);
into_msg_with_type!(KeylistQuery, CoordinateMediationTypeV1_0, KeylistQuery);
into_msg_with_type!(Keylist, CoordinateMediationTypeV1_0, Keylist);
