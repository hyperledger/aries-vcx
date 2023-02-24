use messages_macros::Message;
use serde::{Deserialize, Serialize};
use transitive::TransitiveFrom;

use crate::{
    aries_message::AriesMessage,
    macros::threadlike_ack,
    message_type::message_family::cred_issuance::CredentialIssuanceV1_0,
    protocols::{common::ack::Ack, traits::ConcreteMessage},
};

use super::CredentialIssuance;

#[derive(Clone, Debug, Deserialize, Serialize, Message, TransitiveFrom)]
#[message(kind = "CredentialIssuanceV1_0::Ack")]
#[transitive(into(CredentialIssuance, AriesMessage))]
#[serde(transparent)]
pub struct AckCredential(pub Ack);

threadlike_ack!(AckCredential);
