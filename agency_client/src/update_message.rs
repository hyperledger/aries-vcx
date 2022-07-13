use crate::{agency_settings, Client2AgencyMessage, MessageStatusCode, parse_response_from_agency, prepare_message_for_agency};
use crate::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::messages::update_message::{UIDsByConn, UpdateMessageStatusByConnectionsBuilder};
use crate::testing::{mocking, test_constants};
use crate::testing::mocking::AgencyMock;
use crate::utils::comm::post_to_agency;

pub async fn update_messages(status_code: MessageStatusCode, uids_by_conns: Vec<UIDsByConn>) -> AgencyClientResult<()> {
    trace!("update_messages >>> ");
    if mocking::agency_mocks_enabled() {
        trace!("update_messages >>> agency mocks enabled, returning empty response");
        return Ok(());
    };

    let agency_did = agency_settings::get_config_value(agency_settings::CONFIG_REMOTE_TO_SDK_DID)?;
    AgencyMock::set_next_response(test_constants::UPDATE_MESSAGES_RESPONSE.to_vec());

    let message = UpdateMessageStatusByConnectionsBuilder::create()
        .uids_by_conns(uids_by_conns)?
        .status_code(status_code)?
        .build();

    let data = prepare_message_for_agency(&Client2AgencyMessage::UpdateMessageStatusByConnections(message), &agency_did).await?;
    let response = post_to_agency(&data).await?;
    let mut response = parse_response_from_agency(&response).await?;

    match response.remove(0) {
        Client2AgencyMessage::UpdateMessageStatusByConnectionsResponse(_) => Ok(()),
        _ => Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, "Message does not match any variant of UpdateMessageStatusByConnectionsResponse"))
    }
}