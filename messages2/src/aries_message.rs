use derive_more::From;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    composite_message::Message,
    delayed_serde::DelayedSerde,
    message_type::{
        message_protocol::{
            basic_message::{BasicMessage as BasicMessageKind, BasicMessageV1, BasicMessageV1_0Kind},
            notification::{Notification, NotificationV1, NotificationV1_0Kind},
            report_problem::{ReportProblem, ReportProblemV1, ReportProblemV1_0Kind},
            routing::{Routing, RoutingV1, RoutingV1_0Kind},
        },
        serde::MessageType,
        Protocol,
    },
    protocols::{
        basic_message::BasicMessage, connection::Connection, cred_issuance::CredentialIssuance,
        discover_features::DiscoverFeatures, notification::Ack, out_of_band::OutOfBand, present_proof::PresentProof,
        report_problem::ProblemReport, revocation::Revocation, routing::Forward, trust_ping::TrustPing,
    },
    utils::MSG_TYPE,
};

#[derive(Clone, Debug, From)]
pub enum AriesMessage {
    Routing(Message<Forward>),
    Connection(Connection),
    Revocation(Revocation),
    CredentialIssuance(CredentialIssuance),
    ReportProblem(ProblemReport),
    PresentProof(PresentProof),
    TrustPing(TrustPing),
    DiscoverFeatures(DiscoverFeatures),
    BasicMessage(BasicMessage),
    OutOfBand(OutOfBand),
    Notification(Ack),
}

impl DelayedSerde for AriesMessage {
    type MsgType<'a> = (Protocol, &'a str);

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType<'de>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (msg_type, kind) = msg_type;

        match msg_type {
            Protocol::Routing(msg_type) => {
                let Routing::V1(RoutingV1::V1_0(_msg_type)) = msg_type;
                let msg_type = RoutingV1_0Kind::Forward;

                Message::<Forward>::delayed_deserialize(msg_type, deserializer).map(From::from)
            }
            Protocol::Connection(msg_type) => {
                Connection::delayed_deserialize((msg_type, kind), deserializer).map(From::from)
            }
            Protocol::Revocation(msg_type) => {
                Revocation::delayed_deserialize((msg_type, kind), deserializer).map(From::from)
            }
            Protocol::CredentialIssuance(msg_type) => {
                CredentialIssuance::delayed_deserialize((msg_type, kind), deserializer).map(From::from)
            }
            Protocol::ReportProblem(msg_type) => {
                let ReportProblem::V1(ReportProblemV1::V1_0(_msg_type)) = msg_type;
                let msg_type = ReportProblemV1_0Kind::ProblemReport;

                ProblemReport::delayed_deserialize(msg_type, deserializer).map(From::from)
            }
            Protocol::PresentProof(msg_type) => {
                PresentProof::delayed_deserialize((msg_type, kind), deserializer).map(From::from)
            }
            Protocol::TrustPing(msg_type) => {
                TrustPing::delayed_deserialize((msg_type, kind), deserializer).map(From::from)
            }
            Protocol::DiscoverFeatures(msg_type) => {
                DiscoverFeatures::delayed_deserialize((msg_type, kind), deserializer).map(From::from)
            }
            Protocol::BasicMessage(msg_type) => {
                let BasicMessageKind::V1(BasicMessageV1::V1_0(_msg_type)) = msg_type;
                let msg_type = BasicMessageV1_0Kind::Message;

                BasicMessage::delayed_deserialize(msg_type, deserializer).map(From::from)
            }
            Protocol::OutOfBand(msg_type) => {
                OutOfBand::delayed_deserialize((msg_type, kind), deserializer).map(From::from)
            }
            Protocol::Notification(msg_type) => {
                let Notification::V1(NotificationV1::V1_0(_msg_type)) = msg_type;
                let msg_type = NotificationV1_0Kind::Ack;

                Ack::delayed_deserialize(msg_type, deserializer).map(From::from)
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
        let MessageType { protocol, kind } = tagged.tag;

        // Instead of matching to oblivion and beyond on the [`MessageType`] family,
        // we make use of [`DelayedSerde`] so the matching happens incrementally.
        // This makes use of the provided deserializer and matches on the [`MessageType`]
        // to determine the type the content must be deserialized to.
        Self::delayed_deserialize((protocol, kind), content_deser)
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
