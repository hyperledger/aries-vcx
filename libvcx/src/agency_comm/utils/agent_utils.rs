use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use agency_comm::{A2AMessage, A2AMessageKinds, A2AMessageV2, agency_settings, parse_response_from_agency, prepare_message_for_agency};
use agency_comm::mocking::agency_mocks_enabled;
use agency_comm::message_type::MessageTypes;
use agency_comm::mocking::AgencyMockDecrypted;
use agency_comm::utils::constants;
use agency_comm::utils::error_utils;
use agency_comm::utils::error::prelude::*;
use agency_comm::utils::comm::post_to_agency;

#[derive(Serialize, Deserialize, Debug)]
pub struct Connect {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "fromDID")]
    from_did: String,
    #[serde(rename = "fromDIDVerKey")]
    from_vk: String,
}

impl Connect {
    fn build(from_did: &str, from_vk: &str) -> Connect {
        Connect {
            msg_type: MessageTypes::build(A2AMessageKinds::Connect),
            from_did: from_did.to_string(),
            from_vk: from_vk.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "withPairwiseDID")]
    from_did: String,
    #[serde(rename = "withPairwiseDIDVerKey")]
    from_vk: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignUp {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
}

impl SignUp {
    fn build() -> SignUp {
        SignUp {
            msg_type: MessageTypes::build(A2AMessageKinds::SignUp),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignUpResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateAgent {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
}

impl CreateAgent {
    fn build() -> CreateAgent {
        CreateAgent {
            msg_type: MessageTypes::build(A2AMessageKinds::CreateAgent),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateAgentResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "withPairwiseDID")]
    from_did: String,
    #[serde(rename = "withPairwiseDIDVerKey")]
    from_vk: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ComMethodUpdated {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateComMethod {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "comMethod")]
    com_method: ComMethod,
}

#[derive(Debug, PartialEq)]
pub enum ComMethodType {
    A2A,
    Webhook,
}

impl Serialize for ComMethodType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let value = match self {
            ComMethodType::A2A => "1",
            ComMethodType::Webhook => "2",
        };
        Value::String(value.to_string()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ComMethodType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;
        match value.as_str() {
            Some("1") => Ok(ComMethodType::A2A),
            Some("2") => Ok(ComMethodType::Webhook),
            _ => Err(de::Error::custom("Unexpected communication method type."))
        }
    }
}

impl UpdateComMethod {
    fn build(com_method: ComMethod) -> UpdateComMethod {
        UpdateComMethod {
            msg_type: MessageTypes::build(A2AMessageKinds::UpdateComMethod),
            com_method,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ComMethod {
    id: String,
    #[serde(rename = "type")]
    e_type: ComMethodType,
    value: String,
}

pub fn connect_v2(my_did: &str, my_vk: &str, agency_did: &str) -> VcxResult<(String, String)> {
    /* STEP 1 - CONNECT */
    let message = A2AMessage::Version2(
        A2AMessageV2::Connect(Connect::build(my_did, my_vk))
    );

    let mut response = send_message_to_agency(&message, agency_did)?;

    let ConnectResponse { from_vk: agency_pw_vk, from_did: agency_pw_did, .. } =
        match response.remove(0) {
            A2AMessage::Version2(A2AMessageV2::ConnectResponse(resp)) =>
                resp,
            _ => return
                Err(VcxError::from_msg(
                    VcxErrorKind::InvalidHttpResponse,
                    "Message does not match any variant of ConnectResponse")
                )
        };

    agency_settings::set_config_value(agency_settings::CONFIG_REMOTE_TO_SDK_VERKEY, &agency_pw_vk);
    Ok((agency_pw_did, agency_pw_vk))
}

pub fn onboarding_v2(my_did: &str, my_vk: &str, agency_did: &str) -> VcxResult<(String, String)> {
    AgencyMockDecrypted::set_next_decrypted_response(constants::CONNECTED_RESPONSE_DECRYPTED);
    let (agency_pw_did, _) = connect_v2(my_did, my_vk, agency_did)?;

    /* STEP 2 - REGISTER */
    let message = A2AMessage::Version2(
        A2AMessageV2::SignUp(SignUp::build())
    );

    AgencyMockDecrypted::set_next_decrypted_response(constants::REGISTER_RESPONSE_DECRYPTED);
    let mut response = send_message_to_agency(&message, &agency_pw_did)?;

    let _response: SignUpResponse =
        match response.remove(0) {
            A2AMessage::Version2(A2AMessageV2::SignUpResponse(resp)) => resp,
            _ => return Err(VcxError::from_msg(VcxErrorKind::InvalidHttpResponse, "Message does not match any variant of SignUpResponse"))
        };

    /* STEP 3 - CREATE AGENT */
    let message = A2AMessage::Version2(
        A2AMessageV2::CreateAgent(CreateAgent::build())
    );
    AgencyMockDecrypted::set_next_decrypted_response(constants::AGENT_CREATED_DECRYPTED);
    let mut response = send_message_to_agency(&message, &agency_pw_did)?;

    let response: CreateAgentResponse =
        match response.remove(0) {
            A2AMessage::Version2(A2AMessageV2::CreateAgentResponse(resp)) => resp,
            _ => return Err(VcxError::from_msg(VcxErrorKind::InvalidHttpResponse, "Message does not match any variant of CreateAgentResponse"))
        };

    Ok((response.from_did, response.from_vk))
}

pub fn update_agent_webhook(webhook_url: &str) -> VcxResult<()> {
    info!("update_agent_webhook >>> webhook_url: {:?}", webhook_url);

    let com_method: ComMethod = ComMethod {
        id: String::from("123"),
        e_type: ComMethodType::Webhook,
        value: String::from(webhook_url),
    };

    match agency_settings::get_config_value(agency_settings::CONFIG_REMOTE_TO_SDK_DID) {
        Ok(to_did) => {
            update_agent_webhook_v2(&to_did, com_method)?;
        }
        Err(e) => warn!("Unable to update webhook (did you provide remote did in the config?): {}", e)
    }
    Ok(())
}

fn update_agent_webhook_v2(to_did: &str, com_method: ComMethod) -> VcxResult<()> {
    info!("> update_agent_webhook_v2");
    if agency_mocks_enabled() {
        warn!("update_agent_webhook_v2 ::: Indy mocks enabled, skipping updating webhook url.");
        return Ok(());
    }

    let message = A2AMessage::Version2(
        A2AMessageV2::UpdateComMethod(UpdateComMethod::build(com_method))
    );
    send_message_to_agency(&message, &to_did)?;
    Ok(())
}

pub fn send_message_to_agency(message: &A2AMessage, did: &str) -> VcxResult<Vec<A2AMessage>> {
    let data = prepare_message_for_agency(message, &did)?;

    let response = post_to_agency(&data)
        .map_err(|err| err.map(VcxErrorKind::InvalidHttpResponse, error_utils::INVALID_HTTP_RESPONSE.message))?;

    parse_response_from_agency(&response)
}

#[cfg(test)]
mod tests {
    use std::env;

    use utils::devsetup::{SetupLibraryAgencyV2, SetupMocks};
    use agency_comm::utils::agent_utils::{ComMethodType, update_agent_webhook};

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

    #[test]
    #[cfg(feature = "general_test")]
    #[cfg(feature = "to_restore")]
    fn test_update_agent_info() {
        let _setup = SetupMocks::init();
        // todo: Need to mock agency v2 response, only agency v1 mocking works
        update_agent_info("123", "value").unwrap();
    }

    #[cfg(feature = "agency_pool_tests")]
    #[test]
    fn test_update_agent_webhook_real() {
        let _setup = SetupLibraryAgencyV2::init();

        ::utils::devsetup::set_consumer(None);
        update_agent_webhook("https://example.org").unwrap();
    }
}
