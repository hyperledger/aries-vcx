use error::{VcxError, VcxErrorKind, VcxResult};
use messages::{A2AMessage, A2AMessageKinds, A2AMessageV2, MessageStatusCode, parse_response_from_agency, prepare_message_for_agency};
use messages::message_type::MessageTypes;
use settings;
use utils::{constants, httpclient};
use utils::httpclient::AgencyMock;

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
    version: settings::ProtocolTypes,
}

impl UpdateMessageStatusByConnectionsBuilder {
    pub fn create() -> UpdateMessageStatusByConnectionsBuilder {
        trace!("UpdateMessageStatusByConnectionsBuilder::create >>>");

        UpdateMessageStatusByConnectionsBuilder {
            status_code: None,
            uids_by_conns: Vec::new(),
            version: settings::get_protocol_type(),
        }
    }

    pub fn uids_by_conns(&mut self, uids_by_conns: Vec<UIDsByConn>) -> VcxResult<&mut Self> {
        //Todo: validate msg_uid??
        self.uids_by_conns = uids_by_conns;
        Ok(self)
    }

    pub fn status_code(&mut self, code: MessageStatusCode) -> VcxResult<&mut Self> {
        //Todo: validate that it can be parsed to number??
        self.status_code = Some(code.clone());
        Ok(self)
    }

    #[allow(dead_code)]
    pub fn version(&mut self, version: &Option<settings::ProtocolTypes>) -> VcxResult<&mut Self> {
        self.version = match version {
            Some(version) => version.clone(),
            None => settings::get_protocol_type()
        };
        Ok(self)
    }

    pub fn send_secure(&mut self) -> VcxResult<()> {
        trace!("UpdateMessages::send >>>");

        AgencyMock::set_next_response(constants::UPDATE_MESSAGES_RESPONSE.to_vec());

        let data = self.prepare_request()?;

        let response = httpclient::post_u8(&data)?;

        self.parse_response(&response)
    }

    fn prepare_request(&mut self) -> VcxResult<Vec<u8>> {
        let message = match self.version {
            settings::ProtocolTypes::V1 |
            settings::ProtocolTypes::V2 |
            settings::ProtocolTypes::V3 |
            settings::ProtocolTypes::V4 =>
                A2AMessage::Version2(
                    A2AMessageV2::UpdateMessageStatusByConnections(
                        UpdateMessageStatusByConnections {
                            msg_type: MessageTypes::build(A2AMessageKinds::UpdateMessageStatusByConnections),
                            uids_by_conns: self.uids_by_conns.clone(),
                            status_code: self.status_code.clone(),
                        }
                    )
                ),
        };

        let agency_did = settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_DID)?;
        prepare_message_for_agency(&message, &agency_did, &self.version)
    }

    fn parse_response(&self, response: &Vec<u8>) -> VcxResult<()> {
        trace!("UpdateMessageStatusByConnectionsBuilder::parse_response >>>");

        let mut response = parse_response_from_agency(response, &self.version)?;

        match response.remove(0) {
            A2AMessage::Version2(A2AMessageV2::UpdateMessageStatusByConnectionsResponse(_)) => Ok(()),
            _ => Err(VcxError::from_msg(VcxErrorKind::InvalidHttpResponse, "Message does not match any variant of UpdateMessageStatusByConnectionsResponse"))
        }
    }
}

pub fn update_agency_messages(status_code: &str, msg_json: &str) -> VcxResult<()> {
    trace!("update_agency_messages >>> status_code: {:?}, msg_json: {:?}", status_code, msg_json);

    let status_code: MessageStatusCode = ::serde_json::from_str(&format!("\"{}\"", status_code))
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize MessageStatusCode: {}", err)))?;

    debug!("updating agency messages {} to status code: {:?}", msg_json, status_code);

    let uids_by_conns: Vec<UIDsByConn> = serde_json::from_str(msg_json)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize UIDsByConn: {}", err)))?;

    update_messages(status_code, uids_by_conns)
}

pub fn update_messages(status_code: MessageStatusCode, uids_by_conns: Vec<UIDsByConn>) -> VcxResult<()> {
    trace!("update_messages >>> ");

    if settings::agency_mocks_enabled() {
        trace!("update_messages >>> agency mocks enabled, returning empty response");
        return Ok(());
    };

    UpdateMessageStatusByConnectionsBuilder::create()
        .uids_by_conns(uids_by_conns)?
        .status_code(status_code)?
        .send_secure()
}

#[cfg(test)]
mod tests {
    #[cfg(any(feature = "agency_pool_tests"))]
    use std::thread;
    #[cfg(any(feature = "agency_pool_tests"))]
    use std::time::Duration;

    use connection::send_generic_message;
    use messages::get_message::download_messages_noauth;
    use messages::MessageStatusCode;
    use messages::update_message::{UIDsByConn, update_agency_messages, UpdateMessageStatusByConnectionsBuilder};
    use utils::devsetup::{SetupAriesMocks, SetupLibraryAgencyV2};
    use utils::httpclient::AgencyMockDecrypted;
    use utils::mockdata::mockdata_agency::AGENCY_MSG_STATUS_UPDATED_BY_CONNS;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_parse_parse_update_messages_response() {
        let _setup = SetupAriesMocks::init();
        AgencyMockDecrypted::set_next_decrypted_response(AGENCY_MSG_STATUS_UPDATED_BY_CONNS);
        UpdateMessageStatusByConnectionsBuilder::create().parse_response(&Vec::from("<something_ecrypted>")).unwrap();
    }

    #[cfg(feature = "agency_pool_tests")]
    #[test]
    fn test_update_agency_messages() {
        let _setup = SetupLibraryAgencyV2::init();
        let (_alice_to_faber, faber_to_alice) = ::connection::tests::create_connected_connections(None, None);

        send_generic_message(faber_to_alice, "Hello 1").unwrap();
        send_generic_message(faber_to_alice, "Hello 2").unwrap();
        send_generic_message(faber_to_alice, "Hello 3").unwrap();

        thread::sleep(Duration::from_millis(1000));
        ::utils::devsetup::set_consumer(None);

        let received = download_messages_noauth(None, Some(vec![MessageStatusCode::Received.to_string()]), None).unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].msgs.len(), 3);
        let pairwise_did = received[0].pairwise_did.clone();
        let uid = received[0].msgs[0].uid.clone();

        let reviewed = download_messages_noauth(Some(vec![pairwise_did.clone()]), Some(vec![MessageStatusCode::Reviewed.to_string()]), None).unwrap();
        let reviewed_count_before = reviewed[0].msgs.len();

        // update status
        let message = serde_json::to_string(&vec![UIDsByConn { pairwise_did: pairwise_did.clone(), uids: vec![uid.clone()] }]).unwrap();
        update_agency_messages("MS-106", &message).unwrap();

        let received = download_messages_noauth(None, Some(vec![MessageStatusCode::Received.to_string()]), None).unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].msgs.len(), 2);

        let reviewed = download_messages_noauth(Some(vec![pairwise_did.clone()]), Some(vec![MessageStatusCode::Reviewed.to_string()]), None).unwrap();
        let reviewed_count_after = reviewed[0].msgs.len();
        assert_eq!(reviewed_count_after, reviewed_count_before + 1);

        let specific_review = download_messages_noauth(Some(vec![pairwise_did.clone()]), Some(vec![MessageStatusCode::Reviewed.to_string()]), Some(vec![uid.clone()])).unwrap();
        assert_eq!(specific_review[0].msgs[0].uid, uid);
    }
}
