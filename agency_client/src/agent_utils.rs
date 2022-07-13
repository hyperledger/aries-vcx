use crate::{agency_settings, parse_response_from_agency, prepare_message_for_agency};
use crate::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::messages::a2a_message::Client2AgencyMessage;
use crate::messages::connect::{Connect, ConnectResponse};
use crate::messages::create_agent::{CreateAgent, CreateAgentResponse};
use crate::messages::sign_up::{SignUp, SignUpResponse};
use crate::messages::update_com_method::{ComMethodType, UpdateComMethod};
use crate::testing::mocking::{agency_mocks_enabled, AgencyMockDecrypted};
use crate::testing::test_constants;
use crate::utils::comm::post_to_agency;
use crate::utils::error_utils;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ComMethod {
    id: String,
    #[serde(rename = "type")]
    e_type: ComMethodType,
    value: String,
}

async fn _connect(my_did: &str, my_vk: &str, agency_did: &str) -> AgencyClientResult<(String, String)> {
    trace!("connect >>> my_did: {}, my_vk: {}, agency_did: {}", my_did, my_vk, agency_did);
    /* STEP 1 - CONNECT */
    let message = Client2AgencyMessage::Connect(Connect::build(my_did, my_vk)
    );

    let mut response = _send_message_to_agency(&message, agency_did).await?;

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
    let mut response = _send_message_to_agency(&message, &agency_pw_did).await?;

    let _response: SignUpResponse =
        match response.remove(0) {
            Client2AgencyMessage::SignUpResponse(resp) => resp,
            _ => return Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, "Message does not match any variant of SignUpResponse"))
        };

    /* STEP 3 - CREATE AGENT */
    let message = Client2AgencyMessage::CreateAgent(CreateAgent::build());
    AgencyMockDecrypted::set_next_decrypted_response(test_constants::AGENT_CREATED_DECRYPTED);
    let mut response = _send_message_to_agency(&message, &agency_pw_did).await?;

    let response: CreateAgentResponse =
        match response.remove(0) {
            Client2AgencyMessage::CreateAgentResponse(resp) => resp,
            _ => return Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, "Message does not match any variant of CreateAgentResponse"))
        };

    trace!("onboarding <<< from_did: {}, from_vk: {}", response.from_did, response.from_vk);
    Ok((response.from_did, response.from_vk))
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
            _send_message_to_agency(&message, &to_did).await?;
        }
        Err(e) => warn!("Unable to update webhook (did you provide remote did in the config?): {}", e)
    }
    Ok(())
}

async fn _send_message_to_agency(message: &Client2AgencyMessage, did: &str) -> AgencyClientResult<Vec<Client2AgencyMessage>> {
    trace!("send_message_to_agency >>> message: ..., did: {}", did);
    let data = prepare_message_for_agency(message, &did).await?;

    let response = post_to_agency(&data)
        .await
        .map_err(|err| err.map(AgencyClientErrorKind::InvalidHttpResponse, error_utils::INVALID_HTTP_RESPONSE.message))?;

    parse_response_from_agency(&response).await
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::agent_utils::update_agent_webhook;

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_update_agent_webhook_real() {
        let _setup = SetupLibraryAgencyV2::init().await;
        update_agent_webhook("https://example.org").unwrap();
    }
}
