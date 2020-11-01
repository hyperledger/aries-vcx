use agency_comm::{A2AMessage, A2AMessageKinds, A2AMessageV2, agency_settings, parse_response_from_agency, prepare_message_for_agency};
use agency_comm::message_type::MessageTypes;
use agency_comm::mocking::AgencyMock;
use agency_comm::util::post_u8;
use error::{VcxError, VcxErrorKind, VcxResult};
use settings;
use utils::{httpclient, validation};
use utils::constants::UPDATE_PROFILE_RESPONSE;

#[derive(Debug)]
pub struct UpdateProfileDataBuilder {
    to_did: String,
    agent_payload: String,
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
            agent_payload: String::new(),
        }
    }

    pub fn to(&mut self, did: &str) -> VcxResult<&mut Self> {
        validation::validate_did(did)?;
        self.to_did = did.to_string();
        Ok(self)
    }

    pub fn name(&mut self, name: &str) -> VcxResult<&mut Self> {
        let config = ConfigOption { name: "name".to_string(), value: name.to_string() };
        self.configs.push(config);
        Ok(self)
    }

    pub fn logo_url(&mut self, url: &str) -> VcxResult<&mut Self> {
        validation::validate_url(url)?;
        let config = ConfigOption { name: "logoUrl".to_string(), value: url.to_string() };
        self.configs.push(config);
        Ok(self)
    }

    pub fn webhook_url(&mut self, url: &Option<String>) -> VcxResult<&mut Self> {
        if let Some(x) = url {
            validation::validate_url(x)?;
            let config = ConfigOption { name: "notificationWebhookUrl".to_string(), value: x.to_string() };
            self.configs.push(config);
        }
        Ok(self)
    }

    pub fn use_public_did(&mut self, did: &Option<String>) -> VcxResult<&mut Self> {
        if let Some(x) = did {
            let config = ConfigOption { name: "publicDid".to_string(), value: x.to_string() };
            self.configs.push(config);
        };
        Ok(self)
    }

    pub fn send_secure(&mut self) -> VcxResult<()> {
        trace!("UpdateProfileData::send_secure >>>");

        AgencyMock::set_next_response(UPDATE_PROFILE_RESPONSE.to_vec());

        let data = self.prepare_request()?;

        let response = post_u8(&data)?;

        self.parse_response(response)
    }

    fn prepare_request(&self) -> VcxResult<Vec<u8>> {
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

    fn parse_response(&self, response: Vec<u8>) -> VcxResult<()> {
        let mut response = parse_response_from_agency(&response)?;

        match response.remove(0) {
            A2AMessage::Version2(A2AMessageV2::UpdateConfigsResponse(_)) => Ok(()),
            _ => Err(VcxError::from_msg(VcxErrorKind::InvalidHttpResponse, "Message does not match any variant of UpdateConfigsResponse"))
        }
    }
}

#[cfg(test)]
mod tests {
    use agency_comm::{agency_settings, update_data};
    use agency_comm::mocking::AgencyMockDecrypted;
    use libindy::utils::signus::create_and_store_my_did;
    use utils::constants::{MY1_SEED, MY2_SEED, MY3_SEED};
    use utils::devsetup::*;
    use utils::mockdata::mockdata_agency::AGENCY_CONFIGS_UPDATED;

    use super::*;

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
            .logo_url(&url).unwrap()
            .prepare_request().unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_update_data_set_values_and_post() {
        let _setup = SetupLibraryWallet::init();

        let (agent_did, agent_vk) = create_and_store_my_did(Some(MY2_SEED), None).unwrap();
        let (_my_did, my_vk) = create_and_store_my_did(Some(MY1_SEED), None).unwrap();
        let (_agency_did, agency_vk) = create_and_store_my_did(Some(MY3_SEED), None).unwrap();

        agency_settings::set_config_value(agency_settings::CONFIG_AGENCY_VERKEY, &agency_vk);
        agency_settings::set_config_value(agency_settings::CONFIG_REMOTE_TO_SDK_VERKEY, &agent_vk);
        agency_settings::set_config_value(agency_settings::CONFIG_SDK_TO_REMOTE_VERKEY, &my_vk);

        let msg = update_data()
            .to(agent_did.as_ref()).unwrap()
            .name("name").unwrap()
            .logo_url("https://random.com").unwrap()
            .prepare_request().unwrap();
        assert!(msg.len() > 0);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_parse_update_profile_response() {
        let _setup = SetupMocks::init();
        AgencyMockDecrypted::set_next_decrypted_response(AGENCY_CONFIGS_UPDATED);
        UpdateProfileDataBuilder::create().parse_response(Vec::from("<something_ecrypted>")).unwrap();
    }
}
