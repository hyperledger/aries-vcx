use serde_json::Value;
use url::Url;

use crate::agency_settings;
use crate::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::testing::mocking;
use crate::utils::{error_utils, validation};

// pub static VALID_AGENCY_CONFIG_KEYS: &[&str] = &[
//     CONFIG_AGENCY_ENDPOINT,
//     CONFIG_AGENCY_DID,
//     CONFIG_AGENCY_VERKEY,
//     CONFIG_REMOTE_TO_SDK_DID,
//     CONFIG_REMOTE_TO_SDK_VERKEY,
//     CONFIG_SDK_TO_REMOTE_DID,
//     CONFIG_SDK_TO_REMOTE_VERKEY,
//     CONFIG_ENABLE_TEST_MODE,
//     CONFIG_WALLET_HANDLE,
// ];

#[derive(Default, Deserialize, Clone)]
pub struct AgencyClient {
    wallet_handle: i32,
    pub agency_url: String,
    pub agency_did: String,
    pub agency_pwdid: String,
    pub agency_vk: String,
    pub agent_pwdid: String,
    pub agent_vk: String,
    pub my_pwdid: String,
    pub my_vk: String,
}

impl AgencyClient {
    pub fn get_wallet_handle(&self) -> AgencyClientResult<i32> { Ok(self.wallet_handle) }
    pub fn get_agency_url(&self) -> AgencyClientResult<String> { Ok(self.agency_url.clone()) }
    pub fn get_agency_did(&self) -> AgencyClientResult<String> { Ok(self.agency_did.clone()) }
    pub fn get_agency_pwdid(&self) -> AgencyClientResult<String> { Ok(self.agency_pwdid.clone()) }
    pub fn get_agency_vk(&self) -> AgencyClientResult<String> { Ok(self.agency_vk.clone()) }
    pub fn get_agent_pwdid(&self) -> AgencyClientResult<String> { Ok(self.agent_pwdid.clone()) }
    pub fn get_agent_vk(&self) -> AgencyClientResult<String> { Ok(self.agent_vk.clone()) }
    pub fn get_my_pwdid(&self) -> AgencyClientResult<String> { Ok(self.my_pwdid.clone()) }
    pub fn get_my_vk(&self) -> AgencyClientResult<String> { Ok(self.my_vk.clone()) }

    pub fn set_wallet_handle(&mut self, wh: i32) {
        self.wallet_handle = wh;
        crate::utils::wallet::set_wallet_handle(indy::WalletHandle(wh));
    }

    pub fn reset_wallet_handle(&mut self) {
        self.wallet_handle = indy::INVALID_WALLET_HANDLE.0;
        crate::utils::wallet::reset_wallet_handle();
    }
    pub fn set_agency_url(&mut self, url: &str) {
        let url = format!("{}/agency/msg", url);
        agency_settings::set_config_value(agency_settings::CONFIG_AGENCY_ENDPOINT, &url);
        self.agency_url = url;
    }
    pub fn set_agency_did(&mut self, did: &str) {
        agency_settings::set_config_value(agency_settings::CONFIG_AGENCY_DID, did);
        self.agency_did = did.to_string();
    }
    pub fn set_agency_pwdid(&mut self, pwdid: &str) {
        agency_settings::set_config_value(agency_settings::CONFIG_REMOTE_TO_SDK_DID, pwdid);
        self.agency_pwdid = pwdid.to_string();
    }
    pub fn set_agency_vk(&mut self, vk: &str) {
        agency_settings::set_config_value(agency_settings::CONFIG_AGENCY_VERKEY, vk);
        self.agency_vk = vk.to_string();
    }
    pub fn set_agent_pwdid(&mut self, pwdid: &str) {
        agency_settings::set_config_value(agency_settings::CONFIG_REMOTE_TO_SDK_DID, pwdid);
        self.agent_pwdid = pwdid.to_string();
    }
    pub fn set_agent_vk(&mut self, vk: &str) {
        agency_settings::set_config_value(agency_settings::CONFIG_REMOTE_TO_SDK_VERKEY, vk);
        self.agent_vk = vk.to_string();
    }
    pub fn set_my_pwdid(&mut self, pwdid: &str) {
        agency_settings::set_config_value(agency_settings::CONFIG_SDK_TO_REMOTE_DID, pwdid);
        self.my_pwdid = pwdid.to_string();
    }
    pub fn set_my_vk(&mut self, vk: &str) {
        agency_settings::set_config_value(agency_settings::CONFIG_SDK_TO_REMOTE_VERKEY, vk);
        self.my_vk = vk.to_string();
    }

    pub fn enable_test_mode(&self) { mocking::enable_agency_mocks() }
    pub fn disable_test_mode(&self) { mocking::disable_agency_mocks() }

    // TODO: This should eventually become private
    pub fn process_config_string(&mut self, config: &str, validate: bool) -> AgencyClientResult<u32> {
        trace!("AgencyClient::process_config_string >>> config {:?}, validate: {:?}", config, validate);

        let configuration: Value = serde_json::from_str(config)
            .map_err(|err| AgencyClientError::from_msg(AgencyClientErrorKind::InvalidJson, format!("Cannot parse config: {}", err)))?;

        if let Value::Object(ref map) = configuration {
            for (key, value) in map {
                trace!("AgencyClient::process_config_string >>> key {:?}, value {:?} ", key, value);
                let value = match value {
                    Value::String(value_) => value_,
                    _ => {
                        warn!("Unexpected config value type for key: {}, value: {:?}", key, value);
                        continue;
                    }
                };
                match key.as_ref() {
                    agency_settings::CONFIG_AGENCY_ENDPOINT => { self.set_agency_url(&value.to_string()); }
                    agency_settings::CONFIG_AGENCY_DID => { self.set_agency_did(&value.to_string()); }
                    agency_settings::CONFIG_AGENCY_VERKEY => { self.set_agency_vk(&value.to_string()); }
                    agency_settings::CONFIG_REMOTE_TO_SDK_DID => { self.set_agent_pwdid(&value.to_string()); }
                    agency_settings::CONFIG_REMOTE_TO_SDK_VERKEY => { self.set_agent_vk(&value.to_string()); }
                    agency_settings::CONFIG_SDK_TO_REMOTE_DID => { self.set_my_pwdid(&value.to_string()); }
                    agency_settings::CONFIG_SDK_TO_REMOTE_VERKEY => { self.set_my_vk(&value.to_string()); }
                    agency_settings::CONFIG_ENABLE_TEST_MODE => { self.enable_test_mode(); }
                    _ => { trace!("AgencyClient::process_config_string >>> ignoring key {}", key); }
                }
            }
        }
        if validate { self.validate()?; };
        Ok(error_utils::SUCCESS.code_num)
    }

    pub fn validate(&self) -> AgencyClientResult<u32> {
        trace!("AgencyClient::validate >>>");

        agency_settings::validate_mandotory_config_val(&self.agency_did, AgencyClientErrorKind::InvalidDid, validation::validate_did)?;
        agency_settings::validate_mandotory_config_val(&self.agency_vk, AgencyClientErrorKind::InvalidVerkey, validation::validate_verkey)?;

        agency_settings::validate_mandotory_config_val(&self.my_pwdid, AgencyClientErrorKind::InvalidDid, validation::validate_did)?;
        agency_settings::validate_mandotory_config_val(&self.my_vk, AgencyClientErrorKind::InvalidVerkey, validation::validate_verkey)?;

        agency_settings::validate_mandotory_config_val(&self.agent_pwdid, AgencyClientErrorKind::InvalidDid, validation::validate_did)?;
        agency_settings::validate_mandotory_config_val(&self.agent_vk, AgencyClientErrorKind::InvalidVerkey, validation::validate_verkey)?;

        agency_settings::validate_mandotory_config_val(&self.agency_url, AgencyClientErrorKind::InvalidUrl, Url::parse)?;

        Ok(error_utils::SUCCESS.code_num)
    }

    // TODO: This should be implemented in the module doing the tests
    pub fn set_testing_defaults_agency(&mut self) -> u32 {
        trace!("set_testing_defaults_agency >>>");

        let default_did = "VsKV7grR1BUE29mG2Fm2kX";
        let default_verkey = "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR";
        let default_url = "http://127.0.0.1:8080";

        self.set_agency_url(default_url);
        self.set_agency_did(default_did);
        self.set_agency_vk(default_verkey);
        self.set_agent_pwdid(default_did);
        self.set_agent_vk(default_verkey);
        self.set_my_pwdid(default_did);
        self.set_my_vk(default_verkey);

        agency_settings::set_testing_defaults_agency();

        error_utils::SUCCESS.code_num
    }

    // pub fn new(config: &str, wallet_handle: i32, validate: bool) -> AgencyClientResult<Self> {
    //     let mut agency_client = Self::default();
    //     agency_client.process_config_string(config, wallet_handle, validate)?;
    //     Ok(agency_client)
    // }
}
