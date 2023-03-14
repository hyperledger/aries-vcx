use crate::{
    errors::error::AgencyClientResult,
    messages::{a2a_message::A2AMessageKinds, message_type::MessageType},
    MessageStatusCode,
};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMessageStatusByConnections {
    #[serde(rename = "@type")]
    msg_type: MessageType,
    status_code: Option<MessageStatusCode>,
    uids_by_conns: Vec<UIDsByConn>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMessageStatusByConnectionsResponse {
    #[serde(rename = "@type")]
    msg_type: MessageType,
    status_code: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UIDsByConn {
    #[serde(rename = "pairwiseDID")]
    pub pairwise_did: String,
    pub uids: Vec<String>,
}

pub struct UpdateMessageStatusByConnectionsBuilder {
    status_code: Option<MessageStatusCode>,
    uids_by_conns: Vec<UIDsByConn>,
}

impl UpdateMessageStatusByConnectionsBuilder {
    pub fn create() -> UpdateMessageStatusByConnectionsBuilder {
        trace!("UpdateMessageStatusByConnectionsBuilder::create >>>");

        UpdateMessageStatusByConnectionsBuilder {
            status_code: None,
            uids_by_conns: Vec::new(),
        }
    }

    pub fn uids_by_conns(&mut self, uids_by_conns: Vec<UIDsByConn>) -> AgencyClientResult<&mut Self> {
        self.uids_by_conns = uids_by_conns;
        Ok(self)
    }

    pub fn status_code(&mut self, code: MessageStatusCode) -> AgencyClientResult<&mut Self> {
        self.status_code = Some(code);
        Ok(self)
    }

    pub fn build(&self) -> UpdateMessageStatusByConnections {
        UpdateMessageStatusByConnections {
            msg_type: MessageType::build_v2(A2AMessageKinds::UpdateMessageStatusByConnections),
            uids_by_conns: self.uids_by_conns.clone(),
            status_code: self.status_code.clone(),
        }
    }
}
