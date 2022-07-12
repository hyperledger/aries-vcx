use crate::{agency_settings, parse_response_from_agency, prepare_message_for_agency};
use crate::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::messages::a2a_message::{A2AMessage, A2AMessageV2};
use crate::messages::onboarding::{ComMethodType, Connect, ConnectResponse, CreateAgent, CreateAgentResponse, SignUp, SignUpResponse, UpdateComMethod};
use crate::testing::mocking::{agency_mocks_enabled, AgencyMockDecrypted};
use crate::testing::test_constants;
use crate::utils::comm::post_to_agency;
use crate::utils::error_utils;

#[derive(Serialize, Deserialize, Debug)]
pub struct ComMethod {
    id: String,
    #[serde(rename = "type")]
    e_type: ComMethodType,
    value: String,
}

pub async fn connect(my_did: &str, my_vk: &str, agency_did: &str) -> AgencyClientResult<(String, String)> {
    trace!("connect >>> my_did: {}, my_vk: {}, agency_did: {}", my_did, my_vk, agency_did);
    /* STEP 1 - CONNECT */
    let message = A2AMessage::Version2(
        A2AMessageV2::Connect(Connect::build(my_did, my_vk))
    );

    let mut response = send_message_to_agency(&message, agency_did).await?;

    let ConnectResponse { from_vk: agency_pw_vk, from_did: agency_pw_did, .. } =
        match response.remove(0) {
            A2AMessage::Version2(A2AMessageV2::ConnectResponse(resp)) =>
                resp,
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
    let (agency_pw_did, _) = connect(my_did, my_vk, agency_did).await?;

    /* STEP 2 - REGISTER */
    let message = A2AMessage::Version2(
        A2AMessageV2::SignUp(SignUp::build())
    );

    AgencyMockDecrypted::set_next_decrypted_response(test_constants::REGISTER_RESPONSE_DECRYPTED);
    let mut response = send_message_to_agency(&message, &agency_pw_did).await?;

    let _response: SignUpResponse =
        match response.remove(0) {
            A2AMessage::Version2(A2AMessageV2::SignUpResponse(resp)) => resp,
            _ => return Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, "Message does not match any variant of SignUpResponse"))
        };

    /* STEP 3 - CREATE AGENT */
    let message = A2AMessage::Version2(
        A2AMessageV2::CreateAgent(CreateAgent::build())
    );
    AgencyMockDecrypted::set_next_decrypted_response(test_constants::AGENT_CREATED_DECRYPTED);
    let mut response = send_message_to_agency(&message, &agency_pw_did).await?;

    let response: CreateAgentResponse =
        match response.remove(0) {
            A2AMessage::Version2(A2AMessageV2::CreateAgentResponse(resp)) => resp,
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
            update_agent_webhook_v2(&to_did, com_method).await?;
        }
        Err(e) => warn!("Unable to update webhook (did you provide remote did in the config?): {}", e)
    }
    Ok(())
}

async fn update_agent_webhook_v2(to_did: &str, com_method: ComMethod) -> AgencyClientResult<()> {
    info!("> update_agent_webhook_v2");
    if agency_mocks_enabled() {
        warn!("update_agent_webhook_v2 ::: Indy mocks enabled, skipping updating webhook url.");
        return Ok(());
    }

    let message = A2AMessage::Version2(
        A2AMessageV2::UpdateComMethod(UpdateComMethod::build(com_method))
    );
    send_message_to_agency(&message, &to_did).await?;
    Ok(())
}

pub async fn send_message_to_agency(message: &A2AMessage, did: &str) -> AgencyClientResult<Vec<A2AMessage>> {
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
    use crate::messages::onboarding::ComMethodType;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_method_type_serialization() {
        assert_eq!("\"1\"", serde_json::to_string::<ComMethodType>(&ComMethodType::A2A).unwrap());
        assert_eq!("\"2\"", serde_json::to_string::<ComMethodType>(&ComMethodType::Webhook).unwrap());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_method_type_deserialization() {
        assert_eq!(ComMethodType::A2A, serde_json::from_str::<ComMethodType>("\"1\"").unwrap());
        assert_eq!(ComMethodType::Webhook, serde_json::from_str::<ComMethodType>("\"2\"").unwrap());
    }


    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_update_agent_webhook_real() {
        let _setup = SetupLibraryAgencyV2::init().await;
        update_agent_webhook("https://example.org").unwrap();
    }
}
