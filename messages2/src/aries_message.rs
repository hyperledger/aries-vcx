use derive_more::From;
use serde::{ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    delayed_serde::DelayedSerde,
    message_type::{MessageFamily, MessageType},
    protocols::{
        basic_message::BasicMessage, connection::Connection, cred_issuance::CredentialIssuance,
        discover_features::DiscoverFeatures, out_of_band::OutOfBand, present_proof::PresentProof,
        report_problem::ProblemReport, revocation::Revocation, routing::Forward, trust_ping::TrustPing,
    },
};

pub const MSG_TYPE: &str = "@type";

#[derive(Clone, Debug, From)]
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
}

impl DelayedSerde for AriesMessage {
    type MsgType = MessageFamily;

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match msg_type {
            Self::MsgType::Routing(msg_type) => Forward::delayed_deserialize(msg_type, deserializer).map(From::from),
            Self::MsgType::Connection(msg_type) => {
                Connection::delayed_deserialize(msg_type, deserializer).map(From::from)
            }
            Self::MsgType::Revocation(msg_type) => {
                Revocation::delayed_deserialize(msg_type, deserializer).map(From::from)
            }
            Self::MsgType::CredentialIssuance(msg_type) => {
                CredentialIssuance::delayed_deserialize(msg_type, deserializer).map(From::from)
            }
            Self::MsgType::ReportProblem(msg_type) => {
                ProblemReport::delayed_deserialize(msg_type, deserializer).map(From::from)
            }
            Self::MsgType::PresentProof(msg_type) => {
                PresentProof::delayed_deserialize(msg_type, deserializer).map(From::from)
            }
            Self::MsgType::TrustPing(msg_type) => {
                TrustPing::delayed_deserialize(msg_type, deserializer).map(From::from)
            }
            Self::MsgType::DiscoverFeatures(msg_type) => {
                DiscoverFeatures::delayed_deserialize(msg_type, deserializer).map(From::from)
            }
            Self::MsgType::BasicMessage(msg_type) => {
                BasicMessage::delayed_deserialize(msg_type, deserializer).map(From::from)
            }
            Self::MsgType::OutOfBand(msg_type) => {
                OutOfBand::delayed_deserialize(msg_type, deserializer).map(From::from)
            }
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
///
/// For readability, the [`MessageType`] matching is done in the [`DelayedSerde::delayed_deserialize`] method.
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
//
// In the event of a `serde` version bump and this breaking, the fix is a matter of
// implementing a struct such as:
// ```
// #[derive(Deserialize)]
// #[serde(tag = "@type")]
// enum MyStruct {
//     Var(u8),
//     Var2(u8)
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
        // As it visits data, it looks for a certain field (MSG_TYPE here), deserializes it and stores it separately.
        // The rest of the data is stored as [`Content`], a thin deserialization format that practically acts as a buffer
        // so the other fields besides the tag are cached.
        let tag_visitor = TaggedContentVisitor::<MessageType>::new(MSG_TYPE, "internally tagged enum A2AMessage");
        let tagged = deserializer.deserialize_any(tag_visitor)?;

        // As the TaggedContent struct has two fields, tag and content, where in our case the tag is `MessageType`,
        // the content is [`Content`], the cached remaining fields of the serialized data.
        // Serde uses this [`ContentDeserializer`] to deserialize from that format.
        let content_deser = ContentDeserializer::<D::Error>::new(tagged.content);

        // Instead of matching to oblivion and beyond on the [`MessageType`] family,
        // we make use of [`DelayedSerde`] so the matching happens incrementally.
        // This makes use of the provided deserializer and matches on the [`MessageType`]
        // to determine the type the content must be deserialized to.
        Self::delayed_deserialize(tagged.tag.family, content_deser)
    }
}

/// Custom [`Serialize`] impl for [`A2AMessage`] to use the
/// correspondent [`MessageType`] as internal tag `@type`.
///
/// For readability, we rely on [`DelayedSerde::delayed_serialize`] to do the actual serialization.
/// We need to construct the serializer after serializing [`MessageType`], hence we pass a constructor closure.
///
/// This design allows us to do a single pattern match and serialize both the correspondent message type
/// of a concrete message as well as the message itself in one go.
//
// Same rationale as with the [`Deserialize`] impl on [`A2AMessage`].
// The state gets created and ended through public API, but to flatten the concrete
// message we use serde's serializer exposed when deriving [`Serialize`].
//
// Using the closure to create the serializer has the benefit of keeping the
// "private" import only here, not throughout the crate.
//
// In the event of a `serde` version bump and this breaking, the fix is a matter of
// implementing a struct such as:
// ```
// #[derive(Serialize)]
// struct A {
//    field: u8
// }
//
// #[derive(Serialize)]
// struct MyStruct {
//     #[serde(rename = "@type")]
//     msg_type: MessageType,
//     #[serde(flatten)]
//     stuff: A
// }
// ```
//
// Then analyze the expanded [`Serialize`] impl and adapt the actual implementation below.
impl Serialize for AriesMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::__private::ser::FlatMapSerializer;

        // Serializing a struct to serde's internal data model happens in three steps,
        // as described (here)[https://serde.rs/impl-serialize.html#serializing-a-sequence-or-map].
        //
        // We initialize the serialization state.
        let mut state = serializer.serialize_map(None)?;
        // We populate the state with the serialized fields.
        // The [`FlatMapSerializer`] is what serde_derive uses to flatten a structure during serialization.
        // We need to accomplish the following:
        // 1) Normally serialize the '@type' field which will contain a [`MessageType`] that's determined by
        // the [`DelayedSerde`] impl on generic [`ConcreteMessage`] impls.
        // 2) Flatten `self` (the actual message) so that it's placed adjacently to the '@type' field.
        self.delayed_serialize(&mut state, &mut FlatMapSerializer)?;
        // We end the serialization state, returning the data in it's serialized format.
        state.end()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // #[test]
    // fn test_ser() {
    //     let msg = AriesMessage::BasicMessage(BasicMessage {
    //         field: "stuff".to_owned(),
    //     });
    //     println!("{}", serde_json::to_string(&msg).unwrap());
    // }

    #[test]
    fn test_de() {
        let json_str = r#"{"@type":"https://didcomm.org/basicmessage/1.0/message","field":"stuff"}"#;
        let msg: AriesMessage = serde_json::from_str(json_str).unwrap();
        println!("{msg:?}");

        let json_str = r#"{"@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/basicmessage/1.0/message","field":"stuff"}"#;
        let msg: AriesMessage = serde_json::from_str(json_str).unwrap();
        println!("{msg:?}");
    }
}
