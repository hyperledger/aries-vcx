use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{Thread, Timing},
    message_type::message_family::cred_issuance::CredentialIssuanceV1_0,
    protocols::traits::MessageKind,
};

use super::CredentialPreview;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "CredentialIssuanceV1_0::ProposeCredential")]
pub struct ProposeCredential {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub credential_proposal: CredentialPreview,
    pub schema_id: String,
    pub cred_def_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProposeCredentialDecorators {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
