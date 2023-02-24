use messages_macros::Message;
use serde::{Deserialize, Serialize};
use transitive::TransitiveFrom;

use crate::{
    aries_message::AriesMessage,
    decorators::{Thread, Timing},
    macros::threadlike_opt_impl,
    message_type::message_family::cred_issuance::CredentialIssuanceV1_0,
    protocols::traits::ConcreteMessage,
};

use super::{CredentialIssuance, CredentialPreview};

#[derive(Clone, Debug, Deserialize, Serialize, Message, TransitiveFrom)]
#[message(kind = "CredentialIssuanceV1_0::ProposeCredential")]
#[transitive(into(CredentialIssuance, AriesMessage))]
pub struct ProposeCredential {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub credential_proposal: CredentialPreview,
    pub schema_id: String,
    pub cred_def_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

threadlike_opt_impl!(ProposeCredential);
