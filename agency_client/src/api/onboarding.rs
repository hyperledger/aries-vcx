use crate::agency_settings;
use crate::agency_client::AgencyClient;
use crate::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::messages::a2a_message::Client2AgencyMessage;
use crate::messages::connect::{Connect, ConnectResponse};
use crate::messages::create_agent::{CreateAgent, CreateAgentResponse};
use crate::messages::sign_up::{SignUp, SignUpResponse};
use crate::provision::AgencyClientConfig;
use crate::testing::mocking::AgencyMockDecrypted;
use crate::testing::test_constants;

impl AgencyClient {
    async fn _connect(&self, my_did: &str, my_vk: &str, agency_did: &str) -> AgencyClientResult<(String, String)> {
        trace!("connect >>> my_did: {}, my_vk: {}, agency_did: {}", my_did, my_vk, agency_did);
        /* STEP 1 - CONNECT */
        let message = Client2AgencyMessage::Connect(Connect::build(my_did, my_vk)
        );

        let mut response = self.send_message_to_agency(&message, agency_did).await?;

        let ConnectResponse { from_vk: agency_pw_vk, from_did: agency_pw_did, .. } =
            match response.remove(0) {
                Client2AgencyMessage::ConnectResponse(resp) => resp,
                _ => return
                    Err(AgencyClientError::from_msg(
                        AgencyClientErrorKind::InvalidHttpResponse,
                        "Message does not match any variant of ConnectResponse")
                    )
            };

        agency_settings::set_config_value(agency_settings::CONFIG_REMOTE_TO_SDK_VERKEY, &agency_pw_vk);
        agency_settings::set_config_value(agency_settings::CONFIG_REMOTE_TO_SDK_DID, &agency_pw_did);

        trace!("connect <<< agency_pw_did: {}, agency_pw_vk: {}", agency_pw_did, agency_pw_vk);
        Ok((agency_pw_did, agency_pw_vk))
    }

    pub async fn provision_cloud_agent(&self, my_did: &str, my_vk: &str, agency_did: &str, agency_vk: &str, agency_endpoint: &str) -> AgencyClientResult<AgencyClientConfig> {
        info!("provision_cloud_agent >>> my_did: {}, my_vk: {}, agency_did: {}", my_did, my_vk, agency_did);
        AgencyMockDecrypted::set_next_decrypted_response(test_constants::CONNECTED_RESPONSE_DECRYPTED);
        let (agency_pw_did, _) = self._connect(my_did, my_vk, agency_did).await?;

        /* STEP 2 - REGISTER */
        let message = Client2AgencyMessage::SignUp(SignUp::build());

        AgencyMockDecrypted::set_next_decrypted_response(test_constants::REGISTER_RESPONSE_DECRYPTED);
        let mut response = self.send_message_to_agency(&message, &agency_pw_did).await?;

        let _response: SignUpResponse =
            match response.remove(0) {
                Client2AgencyMessage::SignUpResponse(resp) => resp,
                _ => return Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, "Message does not match any variant of SignUpResponse"))
            };

        /* STEP 3 - CREATE AGENT */
        let message = Client2AgencyMessage::CreateAgent(CreateAgent::build());
        AgencyMockDecrypted::set_next_decrypted_response(test_constants::AGENT_CREATED_DECRYPTED);
        let mut response = self.send_message_to_agency(&message, &agency_pw_did).await?;

        let response: CreateAgentResponse =
            match response.remove(0) {
                Client2AgencyMessage::CreateAgentResponse(resp) => resp,
                _ => return Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, "Message does not match any variant of CreateAgentResponse"))
            };

        trace!("provision_cloud_agent <<< from_did: {}, from_vk: {}", response.from_did, response.from_vk);
        let agent_did = response.from_did;
        let agent_vk = response.from_vk;

        Ok(AgencyClientConfig {
            agency_did: agency_did.into(),
            agency_verkey: agency_vk.into(),
            agency_endpoint: agency_endpoint.into(),
            remote_to_sdk_did: agent_did,
            remote_to_sdk_verkey: agent_vk,
            sdk_to_remote_did: my_did.into(),
            sdk_to_remote_verkey: my_vk.into(),
        })
    }
}