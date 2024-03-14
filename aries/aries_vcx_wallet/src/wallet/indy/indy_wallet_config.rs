use async_trait::async_trait;
use indy_api_types::{
    domain::wallet::{default_key_derivation_method, Config, Credentials},
    errors::IndyErrorKind,
};
use log::warn;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use vdrtools::Locator;

use super::indy_utils::parse_key_derivation_method;
use crate::{
    errors::error::{VcxWalletError, VcxWalletResult},
    wallet::{base_wallet::ManageWallet, indy::IndySdkWallet},
};

#[derive(Clone, Debug, TypedBuilder, Serialize, Deserialize)]
#[builder(field_defaults(default))]
pub struct IndyWalletConfig {
    pub wallet_name: String,
    pub wallet_key: String,
    pub wallet_key_derivation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(strip_option))]
    pub wallet_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(strip_option))]
    pub storage_config: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(strip_option))]
    pub storage_credentials: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(strip_option))]
    pub rekey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(strip_option))]
    pub rekey_derivation_method: Option<String>,
}

impl IndyWalletConfig {
    pub fn to_config_and_creds(&self) -> VcxWalletResult<(Config, Credentials)> {
        let creds = Credentials {
            key: self.wallet_key.clone(),
            key_derivation_method: parse_key_derivation_method(&self.wallet_key_derivation)?,

            rekey: None,
            rekey_derivation_method: default_key_derivation_method(),

            storage_credentials: self
                .storage_credentials
                .as_deref()
                .map(serde_json::from_str)
                .transpose()?,
        };

        let config = Config {
            id: self.wallet_name.clone(),
            storage_type: self.wallet_type.clone(),
            storage_config: self
                .storage_config
                .as_deref()
                .map(serde_json::from_str)
                .transpose()?,
            cache: None,
        };

        Ok((config, creds))
    }
}

#[async_trait]
impl ManageWallet for IndyWalletConfig {
    type ManagedWalletType = IndySdkWallet;

    async fn create_wallet(&self) -> VcxWalletResult<Self::ManagedWalletType> {
        let (config, creds) = self.to_config_and_creds()?;

        let res = Locator::instance()
            .wallet_controller
            .create(config, creds)
            .await;

        match res {
            Ok(()) => self.open_wallet().await,

            Err(err) if err.kind() == IndyErrorKind::WalletAlreadyExists => {
                warn!(
                    "wallet \"{}\" already exists. skipping creation",
                    self.wallet_name
                );
                self.open_wallet().await
            }

            Err(err) => Err(VcxWalletError::create_wallet_error(err)),
        }
    }

    async fn open_wallet(&self) -> VcxWalletResult<Self::ManagedWalletType> {
        let (config, creds) = self.to_config_and_creds()?;

        let handle = Locator::instance()
            .wallet_controller
            .open(config, creds)
            .await?;

        Ok(IndySdkWallet::new(handle))
    }

    async fn delete_wallet(&self) -> VcxWalletResult<()> {
        let (config, creds) = self.to_config_and_creds()?;

        let res = Locator::instance()
            .wallet_controller
            .delete(config, creds)
            .await;

        Ok(res.map(|_| ())?)

        // match res {
        //     Ok(_) => Ok(()),

        //     Err(err) if err.kind() == IndyErrorKind::WalletAccessFailed => {
        //         Err(VcxWalletError::WalletAccessFailed(format!(
        //             "Can not open wallet \"{}\". Invalid key has been provided.",
        //             &self.wallet_name
        //         )))
        //     }

        //     Err(err) if err.kind() == IndyErrorKind::WalletNotFound => {
        //         Err(VcxWalletError::WalletNotFound(format!(
        //             "Wallet \"{}\" not found or unavailable",
        //             &self.wallet_name
        //         )))
        //     }

        //     Err(err) => Err(err.into()),
        // }
    }
}
