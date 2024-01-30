use std::sync::Arc;

use async_trait::async_trait;
use indy_api_types::{domain::wallet::default_key_derivation_method, errors::IndyErrorKind};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use vdrtools::Locator;

use crate::{
    errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult},
    wallet::{base_wallet::ManageWallet, indy::IndySdkWallet},
};

use super::{indy_utils::parse_key_derivation_method, BaseWallet};

#[derive(Clone, Debug, TypedBuilder, Serialize, Deserialize)]
#[builder(field_defaults(default))]
pub struct WalletConfig {
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

#[async_trait]
impl ManageWallet for WalletConfig {
    // type Wallet = AnyWallet;

    async fn create_wallet(&self) -> VcxCoreResult<Arc<dyn BaseWallet>> {
        let handle = Locator::instance()
            .wallet_controller
            .open(
                vdrtools::types::domain::wallet::Config {
                    id: self.wallet_name.clone(),
                    storage_type: self.wallet_type.clone(),
                    storage_config: self
                        .storage_config
                        .as_deref()
                        .map(serde_json::from_str)
                        .transpose()?,
                    cache: None,
                },
                vdrtools::types::domain::wallet::Credentials {
                    key: self.wallet_key.clone(),
                    key_derivation_method: parse_key_derivation_method(
                        &self.wallet_key_derivation,
                    )?,

                    rekey: self.rekey.clone(),
                    rekey_derivation_method: self
                        .rekey_derivation_method
                        .as_deref()
                        .map(parse_key_derivation_method)
                        .transpose()?
                        .unwrap_or_else(default_key_derivation_method),

                    storage_credentials: self
                        .storage_credentials
                        .as_deref()
                        .map(serde_json::from_str)
                        .transpose()?,
                },
            )
            .await?;

        Ok(Arc::new(IndySdkWallet::new(handle)))
    }

    async fn open_wallet(&self) -> VcxCoreResult<Arc<dyn BaseWallet>> {
        let handle = Locator::instance()
            .wallet_controller
            .open(
                vdrtools::types::domain::wallet::Config {
                    id: self.wallet_name.clone(),
                    storage_type: self.wallet_type.clone(),
                    storage_config: self
                        .storage_config
                        .as_deref()
                        .map(serde_json::from_str)
                        .transpose()?,
                    cache: None,
                },
                vdrtools::types::domain::wallet::Credentials {
                    key: self.wallet_key.clone(),
                    key_derivation_method: parse_key_derivation_method(
                        &self.wallet_key_derivation,
                    )?,

                    rekey: self.rekey.clone(),
                    rekey_derivation_method: self
                        .rekey_derivation_method
                        .as_deref()
                        .map(parse_key_derivation_method)
                        .transpose()?
                        .unwrap_or_else(default_key_derivation_method),

                    storage_credentials: self
                        .storage_credentials
                        .as_deref()
                        .map(serde_json::from_str)
                        .transpose()?,
                },
            )
            .await?;

        Ok(Arc::new(IndySdkWallet::new(handle)))
    }

    async fn delete_wallet(&self) -> VcxCoreResult<()> {
        let credentials = vdrtools::types::domain::wallet::Credentials {
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

        let res = Locator::instance()
            .wallet_controller
            .delete(
                vdrtools::types::domain::wallet::Config {
                    id: self.wallet_name.clone(),
                    storage_type: self.wallet_type.clone(),
                    storage_config: self
                        .storage_config
                        .as_deref()
                        .map(serde_json::from_str)
                        .transpose()?,
                    cache: None,
                },
                credentials,
            )
            .await;

        match res {
            Ok(_) => Ok(()),

            Err(err) if err.kind() == IndyErrorKind::WalletAccessFailed => {
                Err(AriesVcxCoreError::from_msg(
                    AriesVcxCoreErrorKind::WalletAccessFailed,
                    format!(
                        "Can not open wallet \"{}\". Invalid key has been provided.",
                        &self.wallet_name
                    ),
                ))
            }

            Err(err) if err.kind() == IndyErrorKind::WalletNotFound => {
                Err(AriesVcxCoreError::from_msg(
                    AriesVcxCoreErrorKind::WalletNotFound,
                    format!("Wallet \"{}\" not found or unavailable", &self.wallet_name,),
                ))
            }

            Err(err) => Err(err.into()),
        }
    }
}
