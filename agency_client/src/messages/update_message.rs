use crate::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::message_type::MessageTypes;
use crate::messages::a2a_message::{A2AMessage, A2AMessageKinds, A2AMessageV2};
use crate::{agency_settings, MessageStatusCode, parse_response_from_agency, prepare_message_for_agency};
use crate::testing::mocking::AgencyMock;
use crate::testing::test_constants;
use crate::utils::comm::post_to_agency;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMessageStatusByConnections {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    status_code: Option<MessageStatusCode>,
    uids_by_conns: Vec<UIDsByConn>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMessageStatusByConnectionsResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
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
        //Todo: validate msg_uid??
        self.uids_by_conns = uids_by_conns;
        Ok(self)
    }

    pub fn status_code(&mut self, code: MessageStatusCode) -> AgencyClientResult<&mut Self> {
        //Todo: validate that it can be parsed to number??
        self.status_code = Some(code.clone());
        Ok(self)
    }

    pub async fn send_secure(&mut self) -> AgencyClientResult<()> {
        trace!("UpdateMessages::send >>>");

        AgencyMock::set_next_response(test_constants::UPDATE_MESSAGES_RESPONSE.to_vec());

        let data = self.prepare_request().await?;

        let response = post_to_agency(&data).await?;

        self.parse_response(&response).await
    }

    async fn prepare_request(&mut self) -> AgencyClientResult<Vec<u8>> {
        let message = A2AMessage::Version2(
            A2AMessageV2::UpdateMessageStatusByConnections(
                UpdateMessageStatusByConnections {
                    msg_type: MessageTypes::build(A2AMessageKinds::UpdateMessageStatusByConnections),
                    uids_by_conns: self.uids_by_conns.clone(),
                    status_code: self.status_code.clone(),
                }
            )
        );

        let agency_did = agency_settings::get_config_value(agency_settings::CONFIG_REMOTE_TO_SDK_DID)?;
        prepare_message_for_agency(&message, &agency_did).await
    }

    pub async fn parse_response(&self, response: &Vec<u8>) -> AgencyClientResult<()> {
        trace!("UpdateMessageStatusByConnectionsBuilder::parse_response >>>");

        let mut response = parse_response_from_agency(response).await?;

        match response.remove(0) {
            A2AMessage::Version2(A2AMessageV2::UpdateMessageStatusByConnectionsResponse(_)) => Ok(()),
            _ => Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, "Message does not match any variant of UpdateMessageStatusByConnectionsResponse"))
        }
    }
}
