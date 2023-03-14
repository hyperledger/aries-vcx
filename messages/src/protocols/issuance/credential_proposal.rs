use crate::{
    a2a::{A2AMessage, MessageId},
    concepts::{mime_type::MimeType, thread::Thread, timing::Timing},
    protocols::issuance::CredentialPreviewData,
    timing_optional,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct CredentialProposal {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub credential_proposal: CredentialPreviewData,
    pub schema_id: String,
    pub cred_def_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

threadlike_optional!(CredentialProposal);
a2a_message!(CredentialProposal);
timing_optional!(CredentialProposal);

impl CredentialProposal {
    pub fn create() -> Self {
        CredentialProposal::default()
    }

    pub fn set_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }

    pub fn set_schema_id(mut self, schema_id: String) -> Self {
        self.schema_id = schema_id;
        self
    }

    pub fn set_cred_def_id(mut self, cred_def_id: String) -> Self {
        self.cred_def_id = cred_def_id;
        self
    }

    pub fn add_credential_preview_data(mut self, name: &str, value: &str, mime_type: MimeType) -> Self {
        self.credential_proposal = self.credential_proposal.add_value(name, value, mime_type);
        self
    }

    pub fn set_id(mut self, id: &str) -> Self {
        self.id = MessageId(id.to_string());
        self
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct CredentialProposalData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub credential_proposal: CredentialPreviewData,
    pub schema_id: String,
    pub cred_def_id: String,
}

impl CredentialProposalData {
    pub fn create() -> Self {
        CredentialProposalData::default()
    }

    pub fn set_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }

    pub fn set_schema_id(mut self, schema_id: String) -> Self {
        self.schema_id = schema_id;
        self
    }

    pub fn set_cred_def_id(mut self, cred_def_id: String) -> Self {
        self.cred_def_id = cred_def_id;
        self
    }

    pub fn add_credential_preview_data(mut self, name: &str, value: &str, mime_type: MimeType) -> Self {
        self.credential_proposal = self.credential_proposal.add_value(name, value, mime_type);
        self
    }
}

impl From<CredentialProposalData> for CredentialProposal {
    fn from(data: CredentialProposalData) -> Self {
        Self {
            comment: data.comment,
            credential_proposal: data.credential_proposal,
            schema_id: data.schema_id,
            cred_def_id: data.cred_def_id,
            ..Self::default()
        }
    }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use super::*;
    use crate::protocols::issuance::credential_offer::test_utils::{_value, thread};

    pub fn _attachment() -> ::serde_json::Value {
        json!({"credential offer": {}})
    }

    pub fn _comment() -> String {
        String::from("comment")
    }

    pub fn _schema_id() -> String {
        String::from("schema:id")
    }

    pub fn _cred_def_id() -> String {
        String::from("cred_def_id:id")
    }

    pub fn _credential_preview_data() -> CredentialPreviewData {
        let (name, value) = _value();

        CredentialPreviewData::new().add_value(name, value, MimeType::Plain)
    }

    pub fn _credential_proposal() -> CredentialProposal {
        CredentialProposal {
            id: MessageId::id(),
            comment: Some(_comment()),
            credential_proposal: _credential_preview_data(),
            schema_id: _schema_id(),
            thread: Some(thread()),
            cred_def_id: _cred_def_id(),
            timing: None,
        }
    }

    pub fn _credential_proposal_data() -> CredentialProposalData {
        CredentialProposalData {
            comment: Some(_comment()),
            credential_proposal: _credential_preview_data(),
            schema_id: _schema_id(),
            cred_def_id: _cred_def_id(),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use super::*;
    use crate::protocols::issuance::{
        credential_offer::test_utils::{_value, thread_id},
        credential_proposal::test_utils::*,
    };

    #[test]
    fn test_credential_proposal_build_works() {
        let (name, value) = _value();

        let credential_proposal: CredentialProposal = CredentialProposal::create()
            .set_comment(_comment())
            .set_thread_id(&thread_id())
            .set_cred_def_id(_cred_def_id())
            .set_schema_id(_schema_id())
            .add_credential_preview_data(name, value, MimeType::Plain);

        assert_eq!(_credential_proposal(), credential_proposal);
    }
}
