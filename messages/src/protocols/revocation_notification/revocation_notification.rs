use crate::{
    a2a::{A2AMessage, MessageId},
    concepts::{ack::please_ack::PleaseAck, thread::Thread, timing::Timing},
    timing_optional,
};

type CredentialId = String;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct RevocationNotification {
    #[serde(rename = "@id")]
    id: MessageId,
    credential_id: CredentialId,
    revocation_format: RevocationFormat,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    please_ack: Option<PleaseAck>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    timing: Option<Timing>,
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    thread: Option<Thread>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RevocationFormat {
    #[default]
    IndyAnoncreds,
}

please_ack!(RevocationNotification);
a2a_message!(RevocationNotification);
timing_optional!(RevocationNotification);
threadlike_optional!(RevocationNotification);

impl RevocationNotification {
    pub fn create() -> Self {
        Self::default()
    }

    pub fn get_id(&self) -> MessageId {
        self.id.clone()
    }

    pub fn get_credential_id(&self) -> CredentialId {
        self.credential_id.clone()
    }

    pub fn get_revocation_format(&self) -> RevocationFormat {
        self.revocation_format.clone()
    }

    pub fn set_credential_id(mut self, rev_reg_id: String, cred_rev_id: String) -> Self {
        self.credential_id = format!("{}::{}", rev_reg_id, cred_rev_id);
        self
    }

    pub fn set_revocation_format(mut self, revocation_format: RevocationFormat) -> Self {
        self.revocation_format = revocation_format;
        self
    }

    pub fn set_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }
}
