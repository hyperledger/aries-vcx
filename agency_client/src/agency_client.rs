use std::sync::Arc;

use shared_vcx::validation::{did::validate_did, verkey::validate_verkey};
use url::Url;

use crate::{
    configuration::AgencyClientConfig,
    errors::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult},
    wallet::base_agency_client_wallet::{BaseAgencyClientWallet, StubAgencyClientWallet},
};

#[derive(Clone, Debug)]
pub struct AgencyClient {
    wallet: Arc<dyn BaseAgencyClientWallet>,
    pub agency_url: String,
    pub agency_did: String,
    pub agency_vk: String,
    pub agent_pwdid: String,
    pub agent_vk: String,
    pub my_pwdid: String,
    pub my_vk: String,
}

impl AgencyClient {
    pub fn get_wallet(&self) -> Arc<dyn BaseAgencyClientWallet> {
        Arc::clone(&self.wallet)
    }

    pub fn get_agency_url_full(&self) -> String {
        format!("{}/agency/msg", self.agency_url.clone())
    }
    pub(crate) fn get_agency_url_config(&self) -> String {
        self.agency_url.clone()
    }

    pub fn get_agency_did(&self) -> String {
        self.agency_did.clone()
    }
    pub fn get_agency_vk(&self) -> String {
        self.agency_vk.clone()
    }

    pub fn get_agent_pwdid(&self) -> String {
        self.agent_pwdid.clone()
    }
    pub fn get_agent_vk(&self) -> String {
        self.agent_vk.clone()
    }

    pub fn get_my_vk(&self) -> String {
        self.my_vk.clone()
    }

    pub fn set_wallet(&mut self, wallet: Arc<dyn BaseAgencyClientWallet>) {
        self.wallet = wallet
    }

    pub(crate) fn set_agency_url(&mut self, url: &str) {
        self.agency_url = url.to_string();
    }
    pub(crate) fn set_agency_did(&mut self, did: &str) {
        self.agency_did = did.to_string();
    }
    pub(crate) fn set_agency_vk(&mut self, vk: &str) {
        self.agency_vk = vk.to_string();
    }
    pub(crate) fn set_agent_pwdid(&mut self, pwdid: &str) {
        self.agent_pwdid = pwdid.to_string();
    }
    pub(crate) fn set_agent_vk(&mut self, vk: &str) {
        self.agent_vk = vk.to_string();
    }
    pub(crate) fn set_my_pwdid(&mut self, pwdid: &str) {
        self.my_pwdid = pwdid.to_string();
    }
    pub(crate) fn set_my_vk(&mut self, vk: &str) {
        self.my_vk = vk.to_string();
    }

    pub fn configure(
        mut self,
        wallet: Arc<dyn BaseAgencyClientWallet>,
        config: &AgencyClientConfig,
    ) -> AgencyClientResult<Self> {
        info!("AgencyClient::configure >>> config {:?}", config);

        validate_did(&config.agency_did)?;
        validate_verkey(&config.agency_verkey)?;
        validate_did(&config.sdk_to_remote_did)?;
        validate_verkey(&config.sdk_to_remote_verkey)?;
        validate_did(&config.remote_to_sdk_did)?;
        validate_verkey(&config.remote_to_sdk_verkey)?;

        match Url::parse(&config.agency_endpoint) {
            Err(_) => Err(AgencyClientError::from_msg(
                AgencyClientErrorKind::InvalidUrl,
                format!("Endpoint {} is not valid url", &config.agency_endpoint),
            )),
            _ => Ok(()),
        }?;

        self.set_agency_url(&config.agency_endpoint);
        self.set_agency_did(&config.agency_did);
        self.set_agency_vk(&config.agency_verkey);
        self.set_agent_pwdid(&config.remote_to_sdk_did);
        self.set_agent_vk(&config.remote_to_sdk_verkey);
        self.set_my_pwdid(&config.sdk_to_remote_did);
        self.set_my_vk(&config.sdk_to_remote_verkey);
        self.set_wallet(wallet);

        Ok(self)
    }

    pub fn set_testing_defaults_agency(&mut self) {
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
    }

    pub fn new() -> Self {
        AgencyClient {
            wallet: Arc::new(StubAgencyClientWallet {}),
            agency_url: "".to_string(),
            agency_did: "".to_string(),
            agency_vk: "".to_string(),
            agent_pwdid: "".to_string(),
            agent_vk: "".to_string(),
            my_pwdid: "".to_string(),
            my_vk: "".to_string(),
        }
    }

    // todo: use this in favor of `fn new()`
    // pub fn new(config: &str, wallet_handle: WalletHandle, validate: bool) -> AgencyClientResult<Self>
    // {     let mut agency_client = Self::default();
    //     agency_client.process_config_string(config, wallet_handle, validate)?;
    //     Ok(agency_client)
    // }
}
