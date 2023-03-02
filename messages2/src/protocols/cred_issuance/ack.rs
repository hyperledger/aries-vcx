use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    message_type::message_family::cred_issuance::CredentialIssuanceV1_0,
    protocols::{notification::Ack, traits::MessageKind},
};

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "CredentialIssuanceV1_0::Ack")]
#[serde(transparent)]
pub struct AckCredential(pub Ack);
