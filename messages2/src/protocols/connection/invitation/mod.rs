pub mod pairwise;
pub mod public;

use derive_more::From;
use messages_macros::MessageContent;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use url::Url;

use crate::aries_message::MsgWithType;
use crate::composite_message::Message;
use crate::delayed_serde::DelayedSerde;
use crate::message_type::message_family::connection::ConnectionV1_0;
use crate::protocols::traits::MessageKind;

use self::pairwise::{PairwiseInvitation, PwInvitationDecorators};
use self::public::PublicInvitation;

/// Type used to encapsulate a fully resolved invitation, which
/// contains all the information necessary for generating a [`crate::protocols::connection::request::Request`].
///
/// Other invitation types would get resolved to this.
pub type CompleteInvitation = InvitationImpl<Url>;

// We implement the message kind on this type as we have to rely on
// untagged deserialization, since we cannot know the invitation format
// ahead of time.
//
// However, to have the capability of setting different decorators
// based on the invitation format, we don't wrap the [`Invitation`]
// in a [`Message`], but rather its variants.
#[derive(Debug, Clone, From, Deserialize, Serialize, MessageContent)]
#[message(kind = "ConnectionV1_0::Invitation")]
#[serde(untagged)]
pub enum Invitation {
    Public(Message<PublicInvitation>),
    Pairwise(Message<PairwiseInvitation<Url>, PwInvitationDecorators>),
    PairwiseDID(Message<PairwiseInvitation<String>, PwInvitationDecorators>),
}

/// We need a custom [`DelayedSerde`] impl to take advantage of 
/// serde's untagged deserialization.
impl DelayedSerde for Invitation {
    type MsgType = ConnectionV1_0;

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
        MsgWithType::from(self).serialize(serializer)
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
