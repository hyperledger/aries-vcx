use crate::{A2AMessage, A2AMessageKinds, A2AMessageV2, agency_settings, MessageStatusCode, mocking, parse_response_from_agency, prepare_message_for_agency};
use crate::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::message_type::MessageTypes;
use crate::mocking::AgencyMock;
use crate::utils::comm::post_to_agency;
use crate::utils::constants;

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
    updated_uids_by_conns: Vec<UIDsByConn>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UIDsByConn {
    #[serde(rename = "pairwiseDID")]
    pub pairwise_did: String,
    pub uids: Vec<String>,
}

struct UpdateMessageStatusByConnectionsBuilder {
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

        AgencyMock::set_next_response(constants::UPDATE_MESSAGES_RESPONSE.to_vec());

        let data = self.prepare_request()?;

        let response = post_to_agency(&data).await?;

        self.parse_response(&response)
    }

    fn prepare_request(&mut self) -> AgencyClientResult<Vec<u8>> {
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
        prepare_message_for_agency(&message, &agency_did)
    }

    fn parse_response(&self, response: &Vec<u8>) -> AgencyClientResult<()> {
        trace!("UpdateMessageStatusByConnectionsBuilder::parse_response >>>");

        let mut response = parse_response_from_agency(response)?;

        match response.remove(0) {
            A2AMessage::Version2(A2AMessageV2::UpdateMessageStatusByConnectionsResponse(_)) => Ok(()),
            _ => Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, "Message does not match any variant of UpdateMessageStatusByConnectionsResponse"))
        }
    }
}

pub async fn update_agency_messages(status_code: &str, msg_json: &str) -> AgencyClientResult<()> {
    trace!("update_agency_messages >>> status_code: {:?}, msg_json: {:?}", status_code, msg_json);

    let status_code: MessageStatusCode = ::serde_json::from_str(&format!("\"{}\"", status_code))
        .map_err(|err| AgencyClientError::from_msg(AgencyClientErrorKind::InvalidJson, format!("Cannot deserialize MessageStatusCode: {}", err)))?;

    debug!("updating agency messages {} to status code: {:?}", msg_json, status_code);

    let uids_by_conns: Vec<UIDsByConn> = serde_json::from_str(msg_json)
        .map_err(|err| AgencyClientError::from_msg(AgencyClientErrorKind::InvalidJson, format!("Cannot deserialize UIDsByConn: {}", err)))?;

    update_messages(status_code, uids_by_conns).await
}

pub async fn update_messages(status_code: MessageStatusCode, uids_by_conns: Vec<UIDsByConn>) -> AgencyClientResult<()> {
    trace!("update_messages >>> ");

    if mocking::agency_mocks_enabled() {
        trace!("update_messages >>> agency mocks enabled, returning empty response");
        return Ok(());
    };

    UpdateMessageStatusByConnectionsBuilder::create()
        .uids_by_conns(uids_by_conns)?
        .status_code(status_code)?
        .send_secure()
        .await
}

#[cfg(test)]
mod tests {
    use crate::mocking;
    use crate::update_message::UpdateMessageStatusByConnectionsBuilder;
    use crate::utils::test_constants::AGENCY_MSG_STATUS_UPDATED_BY_CONNS;
    use crate::utils::test_utils::SetupMocks;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_parse_update_messages_response() {
        let _setup = SetupMocks::init();
        mocking::AgencyMockDecrypted::set_next_decrypted_response(AGENCY_MSG_STATUS_UPDATED_BY_CONNS);
        UpdateMessageStatusByConnectionsBuilder::create().parse_response(&Vec::from("<something_ecrypted>")).unwrap();
    }
}
