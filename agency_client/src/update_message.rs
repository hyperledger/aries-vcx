use crate::MessageStatusCode;
use crate::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::messages::update_message::{UIDsByConn, UpdateMessageStatusByConnectionsBuilder};
use crate::testing::mocking;

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
    use crate::messages::update_message::UpdateMessageStatusByConnectionsBuilder;
    use crate::testing::mocking;
    use crate::testing::test_constants::AGENCY_MSG_STATUS_UPDATED_BY_CONNS;
    use crate::testing::test_utils::SetupMocks;

    #[async_std::test]
    #[cfg(feature = "general_test")]
    async fn test_parse_update_messages_response() {
        let _setup = SetupMocks::init();
        mocking::AgencyMockDecrypted::set_next_decrypted_response(AGENCY_MSG_STATUS_UPDATED_BY_CONNS);
        UpdateMessageStatusByConnectionsBuilder::create().parse_response(&Vec::from("<something_ecrypted>")).await.unwrap();
    }
}
