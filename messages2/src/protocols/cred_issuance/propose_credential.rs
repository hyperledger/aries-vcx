use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{Thread, Timing},
    msg_types::types::cred_issuance::CredentialIssuanceV1_0Kind,
    Message,
};

use super::CredentialPreview;

pub type ProposeCredential = Message<ProposeCredentialContent, ProposeCredentialDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "CredentialIssuanceV1_0Kind::ProposeCredential")]
pub struct ProposeCredentialContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub credential_proposal: CredentialPreview,
    pub schema_id: String,
    pub cred_def_id: String,
}

impl ProposeCredentialContent {
    pub fn new(credential_proposal: CredentialPreview, schema_id: String, cred_def_id: String) -> Self {
        Self {
            comment: None,
            credential_proposal,
            schema_id,
            cred_def_id,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ProposeCredentialDecorators {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
