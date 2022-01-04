use crate::{A2AMessage, A2AMessageKinds, A2AMessageV2, agency_settings, parse_response_from_agency, prepare_message_for_agency};
use crate::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::message_type::MessageTypes;
use crate::mocking::AgencyMock;
use crate::utils::{constants, validation};
use crate::utils::comm::post_to_agency;

#[derive(Debug)]
pub struct UpdateProfileDataBuilder {
    to_did: String,
    configs: Vec<ConfigOption>,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct ConfigOption {
    name: String,
    value: String,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct UpdateConfigs {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    configs: Vec<ConfigOption>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct UpdateConfigsResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
}

impl UpdateProfileDataBuilder {
    pub fn create() -> UpdateProfileDataBuilder {
        trace!("UpdateProfileData::create_message >>>");

        UpdateProfileDataBuilder {
            to_did: String::new(),
            configs: Vec::new(),
        }
    }

    pub fn to(&mut self, did: &str) -> AgencyClientResult<&mut Self> {
        validation::validate_did(did)?;
        self.to_did = did.to_string();
        Ok(self)
    }

    pub fn name(&mut self, name: &str) -> AgencyClientResult<&mut Self> {
        let config = ConfigOption { name: "name".to_string(), value: name.to_string() };
        self.configs.push(config);
        Ok(self)
    }

    pub fn webhook_url(&mut self, url: &Option<String>) -> AgencyClientResult<&mut Self> {
        if let Some(x) = url {
            validation::validate_url(x)?;
            let config = ConfigOption { name: "notificationWebhookUrl".to_string(), value: x.to_string() };
            self.configs.push(config);
        }
        Ok(self)
    }

    pub fn use_public_did(&mut self, did: &Option<String>) -> AgencyClientResult<&mut Self> {
        if let Some(x) = did {
            let config = ConfigOption { name: "publicDid".to_string(), value: x.to_string() };
            self.configs.push(config);
        };
        Ok(self)
    }

    pub async fn send_secure(&mut self) -> AgencyClientResult<()> {
        trace!("UpdateProfileData::send_secure >>>");

        AgencyMock::set_next_response(constants::UPDATE_PROFILE_RESPONSE.to_vec());

        let data = self.prepare_request()?;

        let response = post_to_agency(&data).await?;

        self.parse_response(response)
    }

    fn prepare_request(&self) -> AgencyClientResult<Vec<u8>> {
        let message = A2AMessage::Version2(
            A2AMessageV2::UpdateConfigs(
                UpdateConfigs {
                    msg_type: MessageTypes::build(A2AMessageKinds::UpdateConfigs),
                    configs: self.configs.clone(),
                }
            )
        );

        let agency_did = agency_settings::get_config_value(agency_settings::CONFIG_REMOTE_TO_SDK_DID)?;

        prepare_message_for_agency(&message, &agency_did)
    }

    fn parse_response(&self, response: Vec<u8>) -> AgencyClientResult<()> {
        let mut response = parse_response_from_agency(&response)?;

        match response.remove(0) {
            A2AMessage::Version2(A2AMessageV2::UpdateConfigsResponse(_)) => Ok(()),
            _ => Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, "Message does not match any variant of UpdateConfigsResponse"))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{agency_settings, update_data};
    use crate::mocking::AgencyMockDecrypted;
    use crate::utils::test_constants::AGENCY_CONFIGS_UPDATED;
    use crate::utils::test_utils::SetupMocks;
    use crate::utils::update_profile::UpdateProfileDataBuilder;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_update_data_post() {
        let _setup = SetupMocks::init();

        let to_did = "8XFh8yBzrpJQmNyZzgoTqB";
        let name = "name";
        let url = "https://random.com";
        let _msg = update_data()
            .to(to_did).unwrap()
            .name(&name).unwrap()
            .prepare_request().unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_parse_update_profile_response() {
        let _setup = SetupMocks::init();
        AgencyMockDecrypted::set_next_decrypted_response(AGENCY_CONFIGS_UPDATED);
        UpdateProfileDataBuilder::create().parse_response(Vec::from("<something_ecrypted>")).unwrap();
    }
}
