#![allow(clippy::or_fun_call)]
#![allow(clippy::module_inception)]
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::new_without_default)]
#![allow(clippy::inherent_to_string)]
#![allow(clippy::large_enum_variant)]
#![deny(clippy::unwrap_used)]

pub mod decorators;
mod error;
pub mod maybe_known;
pub mod misc;
pub mod msg_fields;
pub mod msg_parts;
pub mod msg_types;

use derive_more::From;
use misc::utils;
use msg_types::{report_problem::ReportProblemTypeV1_0, routing::RoutingTypeV1_0, MsgWithType};
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use shared_vcx::misc::utils::CowStr;

use crate::{
    misc::utils::MSG_TYPE,
    msg_fields::{
        protocols::{
            basic_message::BasicMessage, connection::Connection, cred_issuance::CredentialIssuance,
            discover_features::DiscoverFeatures, notification::Notification, out_of_band::OutOfBand,
            present_proof::PresentProof, report_problem::ProblemReport, revocation::Revocation, routing::Forward,
            trust_ping::TrustPing,
        },
        traits::DelayedSerde,
    },
    msg_types::{
        basic_message::BasicMessageTypeV1_0,
        protocols::{
            basic_message::{BasicMessageType, BasicMessageTypeV1},
            report_problem::{ReportProblemType, ReportProblemTypeV1},
            routing::{RoutingType, RoutingTypeV1},
        },
        MessageType, Protocol,
    },
};

/// Enum that can represent any message of the implemented protocols.
///
/// It abstracts away the `@type` field and uses it to determine how
/// to deserialize the input into the correct message type.
///
/// It also automatically appends the correct `@type` field when serializing
/// a message.
#[derive(Clone, Debug, From, PartialEq)]
pub enum AriesMessage {
    Routing(Forward),
    Connection(Connection),
    Revocation(Revocation),
    CredentialIssuance(CredentialIssuance),
    ReportProblem(ProblemReport),
    PresentProof(PresentProof),
    TrustPing(TrustPing),
    DiscoverFeatures(DiscoverFeatures),
    BasicMessage(BasicMessage),
    OutOfBand(OutOfBand),
    Notification(Notification),
}

impl DelayedSerde for AriesMessage {
    type MsgType<'a> = MessageType<'a>;

    /// Match on every protocol variant and either:
    /// - call the equivalent type's [`DelayedSerde::delayed_deserialize`]
    /// - handle the message kind and directly deserialize to the proper type
    ///
    /// The second option is employed simply because some protocols only have one message
    /// and one version (at least right now) and there's no point in crowding the codebase
    /// with one variant enums or the like.
    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType<'de>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let MessageType {
            protocol,
            kind: kind_str,
        } = msg_type;

        match protocol {
            Protocol::RoutingType(msg_type) => {
                let kind = match msg_type {
                    RoutingType::V1(RoutingTypeV1::V1_0(kind)) => kind.kind_from_str(kind_str),
                };

                match kind.map_err(D::Error::custom)? {
                    RoutingTypeV1_0::Forward => Forward::deserialize(deserializer).map(From::from),
                }
            }
            Protocol::ConnectionType(msg_type) => {
                Connection::delayed_deserialize((msg_type, kind_str), deserializer).map(From::from)
            }
            Protocol::SignatureType(_) => Err(utils::not_standalone_msg::<D>(kind_str)),
            Protocol::RevocationType(msg_type) => {
                Revocation::delayed_deserialize((msg_type, kind_str), deserializer).map(From::from)
            }
            Protocol::CredentialIssuanceType(msg_type) => {
                CredentialIssuance::delayed_deserialize((msg_type, kind_str), deserializer).map(From::from)
            }
            Protocol::ReportProblemType(msg_type) => {
                let kind = match msg_type {
                    ReportProblemType::V1(ReportProblemTypeV1::V1_0(kind)) => kind.kind_from_str(kind_str),
                };

                match kind.map_err(D::Error::custom)? {
                    ReportProblemTypeV1_0::ProblemReport => ProblemReport::deserialize(deserializer).map(From::from),
                }
            }
            Protocol::PresentProofType(msg_type) => {
                PresentProof::delayed_deserialize((msg_type, kind_str), deserializer).map(From::from)
            }
            Protocol::TrustPingType(msg_type) => {
                TrustPing::delayed_deserialize((msg_type, kind_str), deserializer).map(From::from)
            }
            Protocol::DiscoverFeaturesType(msg_type) => {
                DiscoverFeatures::delayed_deserialize((msg_type, kind_str), deserializer).map(From::from)
            }
            Protocol::BasicMessageType(msg_type) => {
                let kind = match msg_type {
                    BasicMessageType::V1(BasicMessageTypeV1::V1_0(kind)) => kind.kind_from_str(kind_str),
                };

                match kind.map_err(D::Error::custom)? {
                    BasicMessageTypeV1_0::Message => BasicMessage::deserialize(deserializer).map(From::from),
                }
            }
            Protocol::OutOfBandType(msg_type) => {
                OutOfBand::delayed_deserialize((msg_type, kind_str), deserializer).map(From::from)
            }
            Protocol::NotificationType(msg_type) => {
                Notification::delayed_deserialize((msg_type, kind_str), deserializer).map(From::from)
            }
        }
    }

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Routing(v) => MsgWithType::from(v).serialize(serializer),
            Self::Connection(v) => v.delayed_serialize(serializer),
            Self::Revocation(v) => v.delayed_serialize(serializer),
            Self::CredentialIssuance(v) => v.delayed_serialize(serializer),
            Self::ReportProblem(v) => MsgWithType::from(v).serialize(serializer),
            Self::PresentProof(v) => v.delayed_serialize(serializer),
            Self::TrustPing(v) => v.delayed_serialize(serializer),
            Self::DiscoverFeatures(v) => v.delayed_serialize(serializer),
            Self::BasicMessage(v) => MsgWithType::from(v).serialize(serializer),
            Self::OutOfBand(v) => v.delayed_serialize(serializer),
            Self::Notification(v) => v.delayed_serialize(serializer),
        }
    }
}

/// Custom [`Deserialize`] impl for [`AriesMessage`] to use the `@type` as internal tag,
/// but deserialize it to a [`MessageType`].
///
/// For readability, the [`MessageType`] matching is done in the
/// [`DelayedSerde::delayed_deserialize`] method.
//
// Yes, we're using some private serde constructs. Here's why I think this is okay:
//  1) This emulates the derived implementation with the #[serde(tag = "@type")] attribute,
// but uses [`MessageType`] instead of some [`Field`] struct that serde generates.
//
//  2) Without this, the implementation would either rely on something inefficient such as [`Value`]
// as an intermediary, use some custom map which fails on duplicate entries as intermediary or
// basically use [`serde_value`] which seems to be an old replica of [`Content`] and
// [`ContentDeserializer`] and requires a pretty much copy paste of [`TaggedContentVisitor`].
// Also, [`serde_value::Value`] seems to always allocate. Using something like `HashMap::<&str,
// &RawValue>` wouldn't work either, as there are issues flattening `serde_json::RawValue`. It
// would also require some custom deserialization afterwards.
//
//  3) Exposing these parts as public is in progress from serde. When that will happen is still
// unknown. See: <https://github.com/serde-rs/serde/issues/741>. With [`serde_value`] lacking
// activity and not seeming to get integrated into [`serde`], this will most likely resurface.
//
//  4) Reimplementing this on breaking semver changes is as easy as expanding the derived
// [`Deserialize`] impl and altering it a bit. And if that fails, the 2nd argument will still be
// viable.
//
//
// In the event of a `serde` version bump and this breaking, the fix is a matter of
// implementing a struct such as:
// ```
// #[derive(Deserialize)]
// #[serde(tag = "@type")]
// enum MyStruct {
//     Var(u8),
//     Var2(u8),
// }
// ```
//
// Then analyze the expanded [`Deserialize`] impl and adapt the actual implementation below.
impl<'de> Deserialize<'de> for AriesMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::__private::de::{ContentDeserializer, TaggedContentVisitor};

        // TaggedContentVisitor is a visitor used in serde_derive for internally tagged enums.
        // As it visits data, it looks for a certain field (MSG_TYPE here), deserializes it and stores it
        // separately. The rest of the data is stored as [`Content`], a thin deserialization format
        // that practically acts as a buffer so the other fields besides the tag are cached.
        let tag_visitor =
            TaggedContentVisitor::<CowStr>::new(MSG_TYPE, concat!("internally tagged enum ", stringify!(AriesMessage)));
        let tagged = deserializer.deserialize_any(tag_visitor)?;

        // The TaggedContent struct has two fields, tag and content, where in our case the tag is
        // `CowStr` and the content is [`Content`], the cached remaining fields of the
        // serialized data. Serde uses this [`ContentDeserializer`] to deserialize from that format.
        let deserializer = ContentDeserializer::<D::Error>::new(tagged.content);

        // CowStr will try to borrow the data if possible, and we'll further
        // borrow from CowStr here until we know the message kind to try to parse.
        let msg_type = tagged.tag.0.as_ref().try_into().map_err(D::Error::custom)?;

        // Instead of matching to oblivion and beyond on the [`MessageType`] protocol,
        // we make use of [`DelayedSerde`] so the matching happens incrementally.
        // This makes use of the provided deserializer and matches on the [`MessageType`]
        // to determine the type the content must be deserialized to.
        Self::delayed_deserialize(msg_type, deserializer)
    }
}

impl Serialize for AriesMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.delayed_serialize(serializer)
    }
}
