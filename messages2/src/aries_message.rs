use derive_more::From;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    composite_message::Message,
    delayed_serde::DelayedSerde,
    message_type::{MessageFamily, MessageType},
    protocols::{
        basic_message::{BasicMessage, BasicMessageDecorators},
        connection::{invitation::Invitation, Connection},
        cred_issuance::CredentialIssuance,
        discover_features::DiscoverFeatures,
        notification::{Ack, AckDecorators},
        out_of_band::OutOfBand,
        present_proof::PresentProof,
        report_problem::{ProblemReport, ProblemReportDecorators},
        revocation::Revocation,
        routing::Forward,
        traits::MessageKind,
        trust_ping::TrustPing,
    },
};

pub const MSG_TYPE: &str = "@type";

#[derive(Clone, Debug, From)]
pub enum AriesMessage {
    Routing(Message<Forward>),
    Connection(Connection),
    Revocation(Revocation),
    CredentialIssuance(CredentialIssuance),
    ReportProblem(Message<ProblemReport, ProblemReportDecorators>),
    PresentProof(PresentProof),
    TrustPing(TrustPing),
    DiscoverFeatures(DiscoverFeatures),
    BasicMessage(Message<BasicMessage, BasicMessageDecorators>),
    OutOfBand(OutOfBand),
    Notification(Message<Ack, AckDecorators>),
}

impl DelayedSerde for AriesMessage {
    type MsgType = MessageFamily;

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match msg_type {
            Self::MsgType::Routing(msg_type) => {
                Message::<Forward>::delayed_deserialize(msg_type, deserializer).map(From::from)
            }
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
                Message::<ProblemReport, ProblemReportDecorators>::delayed_deserialize(msg_type, deserializer)
                    .map(From::from)
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
                Message::<BasicMessage, BasicMessageDecorators>::delayed_deserialize(msg_type, deserializer)
                    .map(From::from)
            }
            Self::MsgType::OutOfBand(msg_type) => {
                OutOfBand::delayed_deserialize(msg_type, deserializer).map(From::from)
            }
            Self::MsgType::Notification(msg_type) => {
                Message::<Ack, AckDecorators>::delayed_deserialize(msg_type, deserializer).map(From::from)
            }
        }
    }

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Routing(v) => v.delayed_serialize(serializer),
            Self::Connection(v) => v.delayed_serialize(serializer),
            Self::Revocation(v) => v.delayed_serialize(serializer),
            Self::CredentialIssuance(v) => v.delayed_serialize(serializer),
            Self::ReportProblem(v) => v.delayed_serialize(serializer),
            Self::PresentProof(v) => v.delayed_serialize(serializer),
            Self::TrustPing(v) => v.delayed_serialize(serializer),
            Self::DiscoverFeatures(v) => v.delayed_serialize(serializer),
            Self::BasicMessage(v) => v.delayed_serialize(serializer),
            Self::OutOfBand(v) => v.delayed_serialize(serializer),
            Self::Notification(v) => v.delayed_serialize(serializer),
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
//     copy paste of [`TaggedContentVisitor`]. Also, [`serde_value::Value`] seems to always alocate.
//     Using something like `HashMap::<&str, &RawValue>` wouldn't work either, as there are issues flattening
//     `serde_json::RawValue`. It would also require some custom deserialization afterwards.
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

        // The TaggedContent struct has two fields, tag and content, where in our case the tag is `MessageType`
        // and the content is [`Content`], the cached remaining fields of the serialized data.
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
// We rely on [`MsgWithType`] to attach the [`MessageType`]
// to the final output.
impl Serialize for AriesMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.delayed_serialize(serializer)
    }
}

/// Struct used for serializing an [`AriesMessage`]
/// by also attaching the [`MessageType`] to the output.
#[derive(Serialize)]
pub(crate) struct MsgWithType<'a, T> {
    #[serde(rename = "@type")]
    msg_type: MessageType,
    #[serde(flatten)]
    message: &'a T,
}

impl<'a, C, MD> From<&'a Message<C, MD>> for MsgWithType<'a, Message<C, MD>>
where
    C: MessageKind,
    MessageType: From<<C as MessageKind>::Kind>,
{
    fn from(content: &'a Message<C, MD>) -> Self {
        let msg_type = C::kind().into();
        Self {
            msg_type,
            message: content,
        }
    }
}

impl<'a> From<&'a Invitation> for MsgWithType<'a, Invitation> {
    fn from(content: &'a Invitation) -> Self {
        let msg_type = Invitation::kind().into();
        Self {
            msg_type,
            message: content,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // #[test]
    // fn test_ser() {
    //     let msg = AriesMessage::BasicMessage(BasicMessage {
    //         id: "test".to_owned(),
    //         sent_time: "test".to_owned(),
    //         content: "test".to_owned(),
    //         l10n: None,
    //         thread: None,
    //         timing: None,
    //     });

    //     println!("{}", serde_json::to_string(&msg).unwrap());
    // }

    #[test]
    fn test_de() {
        let json_str = r#"{"@type":"https://didcomm.org/basicmessage/1.0/message","@id":"test","sent_time":"test","content":"test"}"#;
        let msg: AriesMessage = serde_json::from_str(json_str).unwrap();
        println!("{msg:?}");

        let json_str = r#"{"@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/basicmessage/1.0/message","@id":"test","sent_time":"test","content":"test"}"#;
        let msg: AriesMessage = serde_json::from_str(json_str).unwrap();
        println!("{msg:?}");
    }
}
