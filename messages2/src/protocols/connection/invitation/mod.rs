pub mod pairwise;
pub mod public;

use derive_more::From;
use messages_macros::MessageContent;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use url::Url;

use crate::composite_message::Message;
use crate::decorators::Timing;
use crate::delayed_serde::DelayedSerde;
use crate::message_type::message_family::connection::ConnectionV1_0;
use crate::protocols::traits::MessageKind;

use self::pairwise::PairwiseInvitation;
use self::public::PublicInvitation;

/// Type used to encapsulate a fully resolved invitation, which
/// contains all the information necessary for generating a [`crate::protocols::connection::request::Request`].
///
/// Other invitation types would get resolved to this.
pub type CompleteInvitation = InvitationImpl<Url>;

// While technically true that this type is also just a message content,
// we derive the macro and set a message kind simply so we can reuse
// the derive [`Deserialize`] impl while still doing the safety check
// on the message kind in [`DelayedSerde::delayed_deserialize`].
#[derive(Debug, Clone, From, Deserialize, Serialize, MessageContent)]
#[message(kind = "ConnectionV1_0::Invitation")]
#[serde(untagged)]
pub enum Invitation {
    Public(Message<PublicInvitation>),
    Pairwise(Message<PairwiseInvitation<Url>, PwInvitationDecorators>),
    PairwiseDID(Message<PairwiseInvitation<String>, PwInvitationDecorators>),
}

impl DelayedSerde for Invitation {
    type MsgType = ConnectionV1_0;

    // We have to rely on the untagged deserialization of the enum from here instead
    // of the deserializing variants themselves.
    // Note that the variants still need the message kind and [`DelayedSerde`] impl themselves,
    // but for serialization.
    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expected = Self::kind();
        if msg_type == expected {
            Self::deserialize(deserializer)
        } else {
            let const_msg = concat!("Failed deserializing ", stringify!(Invitation));
            let msg = format!("{const_msg}; Expected kind: {expected:?}, found: {msg_type:?}");
            Err(D::Error::custom(msg))
        }
    }

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Public(v) => v.delayed_serialize(serializer),
            Self::Pairwise(v) => v.delayed_serialize(serializer),
            Self::PairwiseDID(v) => v.delayed_serialize(serializer),
        }
    }
}

/// Represents an invitation with T as the service endpoint.
/// Essentially, T can only be a DID or a URL.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitationImpl<T> {
    pub label: String,
    pub recipient_keys: Vec<String>,
    #[serde(default)]
    pub routing_keys: Vec<String>,
    pub service_endpoint: T,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PwInvitationDecorators {
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
