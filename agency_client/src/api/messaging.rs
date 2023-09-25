use crate::{
    agency_client::AgencyClient,
    api::downloaded_message::DownloadedMessageEncrypted,
    errors::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult},
    messages::{
        a2a_message::Client2AgencyMessage,
        get_messages::GetMessagesBuilder,
        update_message::{UIDsByConn, UpdateMessageStatusByConnectionsBuilder},
    },
    testing::{mocking, mocking::AgencyMock, test_constants},
    MessageStatusCode,
};

impl AgencyClient {
    pub async fn update_messages(
        &self,
        status_code: MessageStatusCode,
        uids_by_conns: Vec<UIDsByConn>,
    ) -> AgencyClientResult<()> {
        trace!("update_messages >>> ");
        if mocking::agency_mocks_enabled() {
            trace!("update_messages >>> agency mocks enabled, returning empty response");
            return Ok(());
        };
        AgencyMock::set_next_response(test_constants::UPDATE_MESSAGES_RESPONSE.to_vec());

        let message = UpdateMessageStatusByConnectionsBuilder::create()
            .uids_by_conns(uids_by_conns)?
            .status_code(status_code)?
            .build();
        let agent_did = self.get_agent_pwdid();
        let data = self
            .prepare_message_for_agent(
                &Client2AgencyMessage::UpdateMessageStatusByConnections(message),
                &agent_did,
            )
            .await?;
        let response = self.post_to_agency(data).await?;
        let mut response = self.parse_response_from_agency(&response).await?;

        match response.remove(0) {
            Client2AgencyMessage::UpdateMessageStatusByConnectionsResponse(_) => Ok(()),
            _ => Err(AgencyClientError::from_msg(
                AgencyClientErrorKind::InvalidHttpResponse,
                "Message does not match any variant of UpdateMessageStatusByConnectionsResponse",
            )),
        }
    }

    pub async fn get_encrypted_connection_messages(
        &self,
        _pw_did: &str,
        to_pw_vk: &str,
        agent_did: &str,
        agent_vk: &str,
        msg_uid: Option<Vec<String>>,
        status_codes: Option<Vec<MessageStatusCode>>,
    ) -> AgencyClientResult<Vec<DownloadedMessageEncrypted>> {
        trace!(
            "get_connection_messages >>> pw_vk: {}, agent_vk: {}, msg_uid: {:?}",
            to_pw_vk,
            agent_vk,
            msg_uid
        );

        let message = Client2AgencyMessage::GetMessages(
            GetMessagesBuilder::create()
                .uid(msg_uid)?
                .status_codes(status_codes)?
                .build(),
        );

        let data = self
            .prepare_message_for_connection_agent(vec![message], to_pw_vk, agent_did, agent_vk)
            .await?;
        let response = self.post_to_agency(data).await?;
        let mut response = self.parse_response_from_agency(&response).await?;

        match response.remove(0) {
            Client2AgencyMessage::GetMessagesResponse(res) => {
                trace!("Interpreting response as V2");
                Ok(res.msgs)
            }
            _ => Err(AgencyClientError::from_msg(
                AgencyClientErrorKind::InvalidHttpResponse,
                "Message does not match any variant of GetMessagesResponse",
            )),
        }
    }
}
