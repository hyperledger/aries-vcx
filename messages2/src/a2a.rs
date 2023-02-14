use derive_more::From;
use serde::{ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    message_type::{message_family::traits::DelayedSerde, MessageFamily, MessageType},
    protocols::{
        basic_message::BasicMessage, connection::Connection, cred_issuance::CredentialIssuance,
        discover_features::DiscoverFeatures, out_of_band::OutOfBand, present_proof::PresentProof,
        report_problem::ProblemReport, revocation::Revocation, routing::Forward, trust_ping::TrustPing,
    },
};

#[derive(From)]
pub enum A2AMessage {
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
}

impl DelayedSerde for A2AMessage {
    type Seg = MessageFamily;

    fn delayed_deserialize<'de, D>(seg: Self::Seg, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match seg {
            Self::Seg::Routing(seg) => Forward::delayed_deserialize(seg, deserializer).map(From::from),
            Self::Seg::Connection(seg) => Connection::delayed_deserialize(seg, deserializer).map(From::from),
            Self::Seg::Revocation(seg) => Revocation::delayed_deserialize(seg, deserializer).map(From::from),
            Self::Seg::CredentialIssuance(seg) => {
                CredentialIssuance::delayed_deserialize(seg, deserializer).map(From::from)
            }
            Self::Seg::ReportProblem(seg) => ProblemReport::delayed_deserialize(seg, deserializer).map(From::from),
            Self::Seg::PresentProof(seg) => PresentProof::delayed_deserialize(seg, deserializer).map(From::from),
            Self::Seg::TrustPing(seg) => TrustPing::delayed_deserialize(seg, deserializer).map(From::from),
            Self::Seg::DiscoverFeatures(seg) => {
                DiscoverFeatures::delayed_deserialize(seg, deserializer).map(From::from)
            }
            Self::Seg::BasicMessage(seg) => BasicMessage::delayed_deserialize(seg, deserializer).map(From::from),
            Self::Seg::OutOfBand(seg) => OutOfBand::delayed_deserialize(seg, deserializer).map(From::from),
        }
    }

    fn delayed_serialize<'a, M, F, S>(&self, state: &'a mut M, closure: &mut F) -> Result<S::Ok, S::Error>
    where
        M: SerializeMap,
        F: FnMut(&'a mut M) -> S,
        S: Serializer,
        S::Error: From<M::Error>,
    {
        match self {
            Self::Routing(v) => v.delayed_serialize(state, closure),
            Self::Connection(v) => v.delayed_serialize(state, closure),
            Self::Revocation(v) => v.delayed_serialize(state, closure),
            Self::CredentialIssuance(v) => v.delayed_serialize(state, closure),
            Self::ReportProblem(v) => v.delayed_serialize(state, closure),
            Self::PresentProof(v) => v.delayed_serialize(state, closure),
            Self::TrustPing(v) => v.delayed_serialize(state, closure),
            Self::DiscoverFeatures(v) => v.delayed_serialize(state, closure),
            Self::BasicMessage(v) => v.delayed_serialize(state, closure),
            Self::OutOfBand(v) => v.delayed_serialize(state, closure),
        }
    }
}

/// Custom [`Deserialize`] impl for [`A2AMessage`] to use the `@type` as internal tag,
/// but deserialize it to a [`MessageType`].
//
// Yes, we're using some private serde constructs. Here's why I think this is okay:
//  1) This emulates the derived implementation with the #[serde(tag = "@type")] attribute,
//     but uses [`MessageType`] instead of some [`Field`] struct that serde generates.
//
//  2) Without this, the implementation would either rely on something inefficient such as [`Value`] as an intermediary,
//     use some custom map which fails on duplicate entries as intermediary or basically use [`serde_value`]
//     which seems to be an old replica of [`Content`] and [`ContentDeserializer`] and require a pretty much
//     copy paste of [`TaggedContentVisitor`].
//
//  3) Exposing these parts as public is in progress from serde. When that will happen is still unknown.
//     See: https://github.com/serde-rs/serde/issues/741
//     With [`serde_value`] lacking activity and not seeming to get integrated into [`serde`], this will most likely resurface.
//
//  4) Reimplementing this on breaking semver changes is as easy as expanding the derived [`Deserialize`] impl and altering it a bit.
//     And if that fails, the 2nd argument will still be viable.
//
impl<'de> Deserialize<'de> for A2AMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::__private::de::{ContentDeserializer, TaggedContentVisitor};

        let tag_visitor = TaggedContentVisitor::<MessageType>::new("@type", "internally tagged enum A2AMessage");
        let tagged = deserializer.deserialize_any(tag_visitor)?;

        let content_deser = ContentDeserializer::<D::Error>::new(tagged.content);

        Self::delayed_deserialize(tagged.tag.family, content_deser)
    }
}

// Same rationale as with the [`Deserialize`] impl on [`A2AMessage`].
// We add the message type and flatten the actual message while serializing.
impl Serialize for A2AMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::__private::ser::FlatMapSerializer;

        let mut state = serializer.serialize_map(None)?;
        self.delayed_serialize(&mut state, &mut FlatMapSerializer)?;
        state.end()
    }
}
