use crate::agency_client::AgencyClient;
use crate::{agency_settings, AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::messages::a2a_message::Client2AgencyMessage;
use crate::messages::create_key::CreateKeyBuilder;
use crate::messages::update_com_method::{ComMethodType, UpdateComMethod};
use crate::messages::update_connection::DeleteConnectionBuilder;
use crate::testing::{mocking, test_constants};
use crate::testing::mocking::{agency_mocks_enabled, AgencyMock};

impl AgencyClient {
    pub async fn delete_connection_agent(&self, _pw_did: &str, to_pw_vk: &str, agent_did: &str, agent_vk: &str) -> AgencyClientResult<()> {
        trace!("send_delete_connection_message >>>");
        let message = DeleteConnectionBuilder::create()
            .build();

        let data = self.prepare_message_for_connection_agent(vec![Client2AgencyMessage::UpdateConnection(message)], to_pw_vk, agent_did, agent_vk).await?;
        let response = self.post_to_agency(&data).await?;
        let mut response = self.parse_response_from_agency(&response).await?;

        match response.remove(0) {
            Client2AgencyMessage::UpdateConnectionResponse(_) => Ok(()),
            _ => Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, "Message does not match any variant of UpdateConnectionResponse"))
        }
    }

    pub async fn create_connection_agent(&self, pw_did: &str, pw_verkey: &str) -> AgencyClientResult<(String, String)> {
        trace!("create_keys >>> pw_did: {}, pw_verkey: {}", pw_did, pw_verkey);

        if mocking::agency_mocks_enabled() {
            warn!("CreateKeyBuilder::send_secure >>> agency mocks enabled, setting next mocked response");
            AgencyMock::set_next_response(test_constants::CREATE_KEYS_V2_RESPONSE.to_vec());
        }

        let message = CreateKeyBuilder::create()
            .for_did(pw_did)?
            .for_verkey(pw_verkey)?
            .build();

        let agent_pwdid = self.get_agent_pwdid();

        let data = self.prepare_message_for_agent(&Client2AgencyMessage::CreateKey(message), &agent_pwdid).await?;
        let response = self.post_to_agency(&data).await?;
        let mut response = self.parse_response_from_agency(&response).await?;

        match response.remove(0) {
            Client2AgencyMessage::CreateKeyResponse(res) => Ok((res.for_did, res.for_verkey)),
            _ => Err(AgencyClientError::from(AgencyClientErrorKind::InvalidHttpResponse))
        }
    }

    pub async fn update_agent_webhook(&self, webhook_url: &str) -> AgencyClientResult<()> {
        info!("update_agent_webhook >>> webhook_url: {:?}", webhook_url);

        if agency_mocks_enabled() {
            warn!("update_agent_webhook ::: Indy mocks enabled, skipping updating webhook url.");
            return Ok(());
        }

        let com_method: ComMethod = ComMethod {
            id: String::from("123"),
            e_type: ComMethodType::Webhook,
            value: String::from(webhook_url),
        };
        let agent_did = self.get_agent_pwdid();
        let message = Client2AgencyMessage::UpdateComMethod(UpdateComMethod::build(com_method));
        let data = self.prepare_message_for_agent(&message, &agent_did).await?;
        let response = self.post_to_agency(&data).await?;
        self.parse_response_from_agency(&response).await?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ComMethod {
    id: String,
    #[serde(rename = "type")]
    e_type: ComMethodType,
    value: String,
}
