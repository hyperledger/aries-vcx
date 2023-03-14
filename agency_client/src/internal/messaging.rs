use core::u8;

use serde_json::Value;

use crate::{
    agency_client::AgencyClient,
    errors::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult},
    httpclient,
    messages::{a2a_message::Client2AgencyMessage, forward::ForwardV2},
    testing::mocking::AgencyMockDecrypted,
};

impl AgencyClient {
    pub async fn post_to_agency(&self, body_content: Vec<u8>) -> AgencyClientResult<Vec<u8>> {
        let url = self.get_agency_url_full();
        httpclient::post_message(body_content, &url).await
    }

    pub async fn prepare_message_for_agency(
        &self,
        message: &Client2AgencyMessage,
        agent_did: &str,
        agent_vk: &str,
    ) -> AgencyClientResult<Vec<u8>> {
        let my_vk = self.get_my_vk();
        info!(
            "prepare_message_for_agency >>> agent_did: {}, agent_did: {}, message: {:?}",
            agent_did, agent_vk, message
        );
        let message = ::serde_json::to_string(&message).map_err(|err| {
            AgencyClientError::from_msg(
                AgencyClientErrorKind::SerializationError,
                format!("Cannot serialize A2A message: {}", err),
            )
        })?;
        let receiver_keys = ::serde_json::to_string(&vec![&agent_vk]).map_err(|err| {
            AgencyClientError::from_msg(
                AgencyClientErrorKind::SerializationError,
                format!("Cannot serialize receiver keys: {}", err),
            )
        })?;
        let message = self
            .get_wallet()
            .pack_message(Some(&my_vk), &receiver_keys, message.as_bytes())
            .await?;

        self.prepare_forward_message(message, agent_did).await
    }

    pub async fn prepare_message_for_agent(
        &self,
        message: &Client2AgencyMessage,
        agent_did: &str,
    ) -> AgencyClientResult<Vec<u8>> {
        info!("prepare_message_for_agent >>> {:?}", message);
        let agent_vk = self.get_agent_vk();
        let my_vk = self.get_my_vk();
        let message = ::serde_json::to_string(&message).map_err(|err| {
            AgencyClientError::from_msg(
                AgencyClientErrorKind::SerializationError,
                format!("Cannot serialize A2A message: {}", err),
            )
        })?;
        let receiver_keys = ::serde_json::to_string(&vec![&agent_vk]).map_err(|err| {
            AgencyClientError::from_msg(
                AgencyClientErrorKind::SerializationError,
                format!("Cannot serialize receiver keys: {}", err),
            )
        })?;
        let message = self
            .get_wallet()
            .pack_message(Some(&my_vk), &receiver_keys, message.as_bytes())
            .await?;

        self.prepare_forward_message(message, agent_did).await
    }

    pub async fn parse_message_from_response(&self, response: &[u8]) -> AgencyClientResult<String> {
        let unpacked_msg = self.get_wallet().unpack_message(response).await?;
        let message: Value = ::serde_json::from_slice(unpacked_msg.as_slice()).map_err(|err| {
            AgencyClientError::from_msg(
                AgencyClientErrorKind::InvalidJson,
                format!("Cannot deserialize response: {}", err),
            )
        })?;
        Ok(message["message"]
            .as_str()
            .ok_or(AgencyClientError::from_msg(
                AgencyClientErrorKind::InvalidJson,
                "Cannot find `message` field on response",
            ))?
            .to_string())
    }

    pub async fn parse_response_from_agency(
        &self,
        response: &Vec<u8>,
    ) -> AgencyClientResult<Vec<Client2AgencyMessage>> {
        trace!(
            "parse_response_from_agency >>> processing payload of {} bytes",
            response.len()
        );

        let message: String = if AgencyMockDecrypted::has_decrypted_mock_responses() {
            warn!("parse_response_from_agency_v2 >> retrieving decrypted mock response");
            AgencyMockDecrypted::get_next_decrypted_response()
        } else {
            self.parse_message_from_response(response).await?
        };
        trace!("parse_response_from_agency >> decrypted message: {}", message);
        let message: Client2AgencyMessage = serde_json::from_str(&message).map_err(|err| {
            AgencyClientError::from_msg(
                AgencyClientErrorKind::InvalidJson,
                format!("Cannot deserialize A2A message: {}", err),
            )
        })?;

        Ok(vec![message])
    }

    async fn prepare_forward_message(&self, message: Vec<u8>, did: &str) -> AgencyClientResult<Vec<u8>> {
        trace!("prepare_forward_message >>>");
        let agency_vk = self.get_agency_vk();

        let message = Client2AgencyMessage::Forward(ForwardV2::new(did.to_string(), message)?);

        match message {
            Client2AgencyMessage::Forward(msg) => self.prepare_forward_message_for_agency_v2(&msg, &agency_vk).await,
            _ => Err(AgencyClientError::from_msg(
                AgencyClientErrorKind::InvalidState,
                "Invalid message type",
            )),
        }
    }

    async fn prepare_forward_message_for_agency_v2(
        &self,
        message: &ForwardV2,
        agency_vk: &str,
    ) -> AgencyClientResult<Vec<u8>> {
        let message = serde_json::to_string(message).map_err(|err| {
            AgencyClientError::from_msg(
                AgencyClientErrorKind::SerializationError,
                format!("Cannot serialize Forward message: {}", err),
            )
        })?;

        let receiver_keys = serde_json::to_string(&vec![agency_vk]).map_err(|err| {
            AgencyClientError::from_msg(
                AgencyClientErrorKind::SerializationError,
                format!("Cannot serialize receiver keys: {}", err),
            )
        })?;

        self.get_wallet()
            .pack_message(None, &receiver_keys, message.as_bytes())
            .await
    }

    pub async fn prepare_message_for_connection_agent(
        &self,
        messages: Vec<Client2AgencyMessage>,
        pw_vk: &str,
        agent_did: &str,
        agent_vk: &str,
    ) -> AgencyClientResult<Vec<u8>> {
        debug!("prepare_message_for_connection_agent >> {:?}", messages);
        let message = messages.get(0).ok_or(AgencyClientError::from_msg(
            AgencyClientErrorKind::SerializationError,
            "Cannot get message",
        ))?;
        let message = serde_json::to_string(message).map_err(|err| {
            AgencyClientError::from_msg(
                AgencyClientErrorKind::SerializationError,
                format!("Cannot serialize A2A message: {}", err),
            )
        })?;
        let receiver_keys = serde_json::to_string(&vec![&agent_vk]).map_err(|err| {
            AgencyClientError::from_msg(
                AgencyClientErrorKind::SerializationError,
                format!("Cannot receiver keys: {}", err),
            )
        })?;

        let message = self
            .get_wallet()
            .pack_message(Some(pw_vk), &receiver_keys, message.as_bytes())
            .await?;

        let message = Client2AgencyMessage::Forward(ForwardV2::new(agent_did.to_owned(), message)?);

        let to_did = self.get_agent_pwdid();
        self.prepare_message_for_agent(&message, &to_did).await
    }

    pub async fn send_message_to_agency(
        &self,
        message: &Client2AgencyMessage,
        did: &str,
        verkey: &str,
    ) -> AgencyClientResult<Vec<Client2AgencyMessage>> {
        trace!("send_message_to_agency >>> message: ..., did: {}", did);
        let data = self.prepare_message_for_agency(message, did, verkey).await?;
        let response = self.post_to_agency(data).await?;
        self.parse_response_from_agency(&response).await
    }
}
