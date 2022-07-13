use crate::{agency_settings, MessageStatusCode};
use crate::agency_client::AgencyClient;
use crate::api::messaging;
use crate::api::messaging::{parse_response_from_agency, prepare_message_for_agency, prepare_message_for_agent};
use crate::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::messages::a2a_message::Client2AgencyMessage;
use crate::messages::create_key::CreateKeyBuilder;
use crate::messages::get_messages::{DownloadedMessageEncrypted, GetMessagesBuilder};
use crate::messages::update_com_method::{ComMethodType, UpdateComMethod};
use crate::messages::update_connection::DeleteConnectionBuilder;
use crate::messages::update_message::{UIDsByConn, UpdateMessageStatusByConnectionsBuilder};
use crate::testing::{mocking, test_constants};
use crate::testing::mocking::{agency_mocks_enabled, AgencyMock};
use crate::utils::comm::post_to_agency;

pub async fn get_encrypted_connection_messages(_pw_did: &str, to_pw_vk: &str, agent_did: &str, agent_vk: &str, msg_uid: Option<Vec<String>>, status_codes: Option<Vec<MessageStatusCode>>) -> AgencyClientResult<Vec<DownloadedMessageEncrypted>> {
    trace!("get_connection_messages >>> pw_vk: {}, agent_vk: {}, msg_uid: {:?}",
           to_pw_vk, agent_vk, msg_uid);

    let message = Client2AgencyMessage::GetMessages(
        GetMessagesBuilder::create()
            .uid(msg_uid)?
            .status_codes(status_codes)?
            .build()
    );

    let data = prepare_message_for_agent(vec![message], &to_pw_vk, &agent_did, &agent_vk).await?;
    let response = post_to_agency(&data).await?;
    let mut response = parse_response_from_agency(&response).await?;

    match response.remove(0) {
        Client2AgencyMessage::GetMessagesResponse(res) => {
            trace!("Interpreting response as V2");
            Ok(res.msgs)
        }
        _ => Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, "Message does not match any variant of GetMessagesResponse"))
    }
}

pub async fn send_delete_connection_message(_pw_did: &str, to_pw_vk: &str, agent_did: &str, agent_vk: &str) -> AgencyClientResult<()> {
    trace!("send_delete_connection_message >>>");
    let message = DeleteConnectionBuilder::create()
        .build();

    let data = prepare_message_for_agent(vec![Client2AgencyMessage::UpdateConnection(message)], to_pw_vk, agent_did, agent_vk).await?;
    let response = post_to_agency(&data).await?;
    let mut response = parse_response_from_agency(&response).await?;

    match response.remove(0) {
        Client2AgencyMessage::UpdateConnectionResponse(_) => Ok(()),
        _ => Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, "Message does not match any variant of UpdateConnectionResponse"))
    }
}

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

impl AgencyClient {
    pub async fn update_messages(&self, status_code: MessageStatusCode, uids_by_conns: Vec<UIDsByConn>) -> AgencyClientResult<()> {
        trace!("update_messages >>> ");
        if mocking::agency_mocks_enabled() {
            trace!("update_messages >>> agency mocks enabled, returning empty response");
            return Ok(());
        };

        trace!("ideally we would use client.agency_did={} on the next line ", self.agency_did);
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
}

pub async fn create_keys(pw_did: &str, pw_verkey: &str) -> AgencyClientResult<(String, String)> {
    trace!("create_keys >>> pw_did: {}, pw_verkey: {}", pw_did, pw_verkey);

    if mocking::agency_mocks_enabled() {
        warn!("CreateKeyBuilder::send_secure >>> agency mocks enabled, setting next mocked response");
        AgencyMock::set_next_response(test_constants::CREATE_KEYS_V2_RESPONSE.to_vec());
    }

    let message = CreateKeyBuilder::create()
        .for_did(pw_did)?
        .for_verkey(pw_verkey)?
        .build();

    let agency_did = agency_settings::get_config_value(agency_settings::CONFIG_REMOTE_TO_SDK_DID)?;

    let data = prepare_message_for_agency(&Client2AgencyMessage::CreateKey(message), &agency_did).await?;
    let response = post_to_agency(&data).await?;
    let mut response = parse_response_from_agency(&response).await?;

    match response.remove(0) {
        Client2AgencyMessage::CreateKeyResponse(res) => Ok((res.for_did, res.for_verkey)),
        _ => Err(AgencyClientError::from(AgencyClientErrorKind::InvalidHttpResponse))
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ComMethod {
    id: String,
    #[serde(rename = "type")]
    e_type: ComMethodType,
    value: String,
}

pub async fn update_agent_webhook(webhook_url: &str) -> AgencyClientResult<()> {
    info!("update_agent_webhook >>> webhook_url: {:?}", webhook_url);

    let com_method: ComMethod = ComMethod {
        id: String::from("123"),
        e_type: ComMethodType::Webhook,
        value: String::from(webhook_url),
    };

    match agency_settings::get_config_value(agency_settings::CONFIG_REMOTE_TO_SDK_DID) {
        Ok(to_did) => {
            if agency_mocks_enabled() {
                warn!("update_agent_webhook_v2 ::: Indy mocks enabled, skipping updating webhook url.");
                return Ok(());
            }

            let message = Client2AgencyMessage::UpdateComMethod(UpdateComMethod::build(com_method));
            messaging::send_message_to_agency(&message, &to_did).await?;
        }
        Err(e) => warn!("Unable to update webhook (did you provide remote did in the config?): {}", e)
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::api::agent::update_agent_webhook;

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_update_agent_webhook_real() {
        let _setup = SetupLibraryAgencyV2::init().await;
        update_agent_webhook("https://example.org").unwrap();
    }
}
