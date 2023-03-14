use crate::{
    api::downloaded_message::DownloadedMessageEncrypted,
    errors::error::AgencyClientResult,
    messages::{a2a_message::A2AMessageKinds, message_type::MessageType},
    MessageStatusCode,
};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetMessages {
    #[serde(rename = "@type")]
    msg_type: MessageType,
    #[serde(rename = "excludePayload")]
    #[serde(skip_serializing_if = "Option::is_none")]
    exclude_payload: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uids: Option<Vec<String>>,
    #[serde(rename = "statusCodes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    status_codes: Option<Vec<MessageStatusCode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "pairwiseDIDs")]
    pairwise_dids: Option<Vec<String>>,
}

impl GetMessages {
    pub(crate) fn build(
        kind: A2AMessageKinds,
        exclude_payload: Option<String>,
        uids: Option<Vec<String>>,
        status_codes: Option<Vec<MessageStatusCode>>,
        pairwise_dids: Option<Vec<String>>,
    ) -> GetMessages {
        GetMessages {
            msg_type: MessageType::build_v2(kind),
            exclude_payload,
            uids,
            status_codes,
            pairwise_dids,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetMessagesResponse {
    #[serde(rename = "@type")]
    pub msg_type: MessageType,
    pub msgs: Vec<DownloadedMessageEncrypted>,
}

#[derive(Debug)]
pub struct GetMessagesBuilder {
    exclude_payload: Option<String>,
    uids: Option<Vec<String>>,
    status_codes: Option<Vec<MessageStatusCode>>,
    pairwise_dids: Option<Vec<String>>,
}

impl GetMessagesBuilder {
    pub fn create() -> GetMessagesBuilder {
        trace!("GetMessages::create_message >>>");

        GetMessagesBuilder {
            uids: None,
            exclude_payload: None,
            status_codes: None,
            pairwise_dids: None,
        }
    }

    pub fn uid(&mut self, uids: Option<Vec<String>>) -> AgencyClientResult<&mut Self> {
        self.uids = uids;
        Ok(self)
    }

    pub fn status_codes(&mut self, status_codes: Option<Vec<MessageStatusCode>>) -> AgencyClientResult<&mut Self> {
        self.status_codes = status_codes;
        Ok(self)
    }

    pub fn build(&self) -> GetMessages {
        GetMessages::build(
            A2AMessageKinds::GetMessages,
            self.exclude_payload.clone(),
            self.uids.clone(),
            self.status_codes.clone(),
            self.pairwise_dids.clone(),
        )
    }
}
