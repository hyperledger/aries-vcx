use std::sync::Arc;

use crate::{
    agency_client::AgencyClient,
    configuration::AgencyClientConfig,
    errors::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult},
    messages::{
        a2a_message::Client2AgencyMessage,
        connect::{Connect, ConnectResponse},
        create_agent::{CreateAgent, CreateAgentResponse},
        sign_up::{SignUp, SignUpResponse},
    },
    testing::{mocking::AgencyMockDecrypted, test_constants},
    wallet::base_agency_client_wallet::BaseAgencyClientWallet,
};

impl AgencyClient {
    async fn _connect(
        &self,
        my_did: &str,
        my_vk: &str,
        agency_did: &str,
        agency_vk: &str,
    ) -> AgencyClientResult<(String, String)> {
        trace!(
            "connect >>> my_did: {}, my_vk: {}, agency_did: {}",
            my_did,
            my_vk,
            agency_did
        );
        let message = Client2AgencyMessage::Connect(Connect::build(my_did, my_vk));

        let mut response = self.send_message_to_agency(&message, agency_did, agency_vk).await?;

        let ConnectResponse {
            from_vk: agency_pw_vk,
            from_did: agency_pw_did,
            ..
        } = match response.remove(0) {
            Client2AgencyMessage::ConnectResponse(resp) => resp,
            _ => {
                return Err(AgencyClientError::from_msg(
                    AgencyClientErrorKind::InvalidHttpResponse,
                    "Message does not match any variant of ConnectResponse",
                ))
            }
        };

        trace!(
            "connect <<< agency_pw_did: {}, agency_pw_vk: {}",
            agency_pw_did,
            agency_pw_vk
        );
        Ok((agency_pw_did, agency_pw_vk))
    }

    async fn _register(&self, agency_pw_did: &str, agency_pw_vk: &str) -> AgencyClientResult<()> {
        let message = Client2AgencyMessage::SignUp(SignUp::build());

        AgencyMockDecrypted::set_next_decrypted_response(test_constants::REGISTER_RESPONSE_DECRYPTED);
        let mut response = self
            .send_message_to_agency(&message, agency_pw_did, agency_pw_vk)
            .await?;

        let _response: SignUpResponse = match response.remove(0) {
            Client2AgencyMessage::SignUpResponse(resp) => resp,
            _ => {
                return Err(AgencyClientError::from_msg(
                    AgencyClientErrorKind::InvalidHttpResponse,
                    "Message does not match any variant of SignUpResponse",
                ))
            }
        };
        Ok(())
    }

    async fn _create_agent(&self, agency_pw_did: &str, agency_pw_vk: &str) -> AgencyClientResult<CreateAgentResponse> {
        let message = Client2AgencyMessage::CreateAgent(CreateAgent::build());
        AgencyMockDecrypted::set_next_decrypted_response(test_constants::AGENT_CREATED_DECRYPTED);
        let mut response = self
            .send_message_to_agency(&message, agency_pw_did, agency_pw_vk)
            .await?;

        let response: CreateAgentResponse = match response.remove(0) {
            Client2AgencyMessage::CreateAgentResponse(resp) => resp,
            _ => {
                return Err(AgencyClientError::from_msg(
                    AgencyClientErrorKind::InvalidHttpResponse,
                    "Message does not match any variant of CreateAgentResponse",
                ))
            }
        };
        Ok(response)
    }

    pub async fn provision_cloud_agent(
        &mut self,
        wallet: Arc<dyn BaseAgencyClientWallet>,
        my_did: &str,
        my_vk: &str,
        agency_did: &str,
        agency_vk: &str,
        agency_url: &str,
    ) -> AgencyClientResult<()> {
        info!(
            "provision_cloud_agent >>> my_did: {}, my_vk: {}, agency_did: {}, agency_vk: {}, agency_url: {}",
            my_did, my_vk, agency_did, agency_vk, agency_url
        );
        self.set_wallet(wallet);
        self.set_agency_url(agency_url);
        self.set_agency_vk(agency_vk);
        self.set_agency_did(agency_did);
        self.set_my_pwdid(my_did);
        self.set_my_vk(my_vk);

        AgencyMockDecrypted::set_next_decrypted_response(test_constants::CONNECTED_RESPONSE_DECRYPTED);
        let (agency_pw_did, agency_pw_vk) = self._connect(my_did, my_vk, agency_did, agency_vk).await?;
        self._register(&agency_pw_did, &agency_pw_vk).await?;
        let create_agent_response = self._create_agent(&agency_pw_did, &agency_pw_vk).await?;

        let agent_did = create_agent_response.from_did;
        let agent_vk = create_agent_response.from_vk;
        trace!(
            "provision_cloud_agent <<< agent_did: {}, agent_vk: {}",
            agent_did,
            agent_vk
        );
        self.set_agent_pwdid(&agent_did);
        self.set_agent_vk(&agent_vk);
        Ok(())
    }

    pub fn get_config(&self) -> AgencyClientResult<AgencyClientConfig> {
        Ok(AgencyClientConfig {
            agency_did: self.get_agency_did(),
            agency_verkey: self.get_agency_vk(),
            agency_endpoint: self.get_agency_url_config(),
            remote_to_sdk_did: self.agent_pwdid.clone(),
            remote_to_sdk_verkey: self.agent_vk.clone(),
            sdk_to_remote_did: self.my_pwdid.clone(),
            sdk_to_remote_verkey: self.my_vk.clone(),
        })
    }
}
