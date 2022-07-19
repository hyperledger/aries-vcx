use crate::agency_settings;
use crate::api::messaging;
use crate::api::messaging::send_message_to_agency;
use crate::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::messages::a2a_message::Client2AgencyMessage;
use crate::messages::connect::{Connect, ConnectResponse};
use crate::messages::create_agent::{CreateAgent, CreateAgentResponse};
use crate::messages::sign_up::{SignUp, SignUpResponse};
use crate::testing::mocking::AgencyMockDecrypted;
use crate::testing::test_constants;

async fn _connect(my_did: &str, my_vk: &str, agency_did: &str) -> AgencyClientResult<(String, String)> {
    trace!("connect >>> my_did: {}, my_vk: {}, agency_did: {}", my_did, my_vk, agency_did);
    /* STEP 1 - CONNECT */
    let message = Client2AgencyMessage::Connect(Connect::build(my_did, my_vk)
    );

    let mut response = send_message_to_agency(&message, agency_did).await?;

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

pub async fn onboarding(my_did: &str, my_vk: &str, agency_did: &str) -> AgencyClientResult<(String, String)> {
    info!("onboarding >>> my_did: {}, my_vk: {}, agency_did: {}", my_did, my_vk, agency_did);
    AgencyMockDecrypted::set_next_decrypted_response(test_constants::CONNECTED_RESPONSE_DECRYPTED);
    let (agency_pw_did, _) = _connect(my_did, my_vk, agency_did).await?;

    /* STEP 2 - REGISTER */
    let message = Client2AgencyMessage::SignUp(SignUp::build());

    AgencyMockDecrypted::set_next_decrypted_response(test_constants::REGISTER_RESPONSE_DECRYPTED);
    let mut response = messaging::send_message_to_agency(&message, &agency_pw_did).await?;

    let _response: SignUpResponse =
        match response.remove(0) {
            Client2AgencyMessage::SignUpResponse(resp) => resp,
            _ => return Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, "Message does not match any variant of SignUpResponse"))
        };

    /* STEP 3 - CREATE AGENT */
    let message = Client2AgencyMessage::CreateAgent(CreateAgent::build());
    AgencyMockDecrypted::set_next_decrypted_response(test_constants::AGENT_CREATED_DECRYPTED);
    let mut response = messaging::send_message_to_agency(&message, &agency_pw_did).await?;

    let response: CreateAgentResponse =
        match response.remove(0) {
            Client2AgencyMessage::CreateAgentResponse(resp) => resp,
            _ => return Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, "Message does not match any variant of CreateAgentResponse"))
        };

    trace!("onboarding <<< from_did: {}, from_vk: {}", response.from_did, response.from_vk);
    Ok((response.from_did, response.from_vk))
}
