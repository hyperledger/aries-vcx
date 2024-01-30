use async_trait::async_trait;
use vdrtools::types::domain::wallet::default_key_derivation_method;

use serde::{Deserialize, Serialize};
use vdrtools::Locator;

use crate::{errors::error::VcxCoreResult, wallet::base_wallet::ImportWallet};

use super::indy_utils::parse_key_derivation_method;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RestoreWalletConfigs {
    pub wallet_name: String,
    pub wallet_key: String,
    pub exported_wallet_path: String,
    pub backup_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_key_derivation: Option<String>,
}

#[async_trait]
impl ImportWallet for RestoreWalletConfigs {
    async fn import_wallet(&self) -> VcxCoreResult<()> {
        Locator::instance()
            .wallet_controller
            .import(
                vdrtools::types::domain::wallet::Config {
                    id: self.wallet_name.clone(),
                    ..Default::default()
                },
                vdrtools::types::domain::wallet::Credentials {
                    key: self.wallet_key.clone(),
                    key_derivation_method: self
                        .wallet_key_derivation
                        .as_deref()
                        .map(parse_key_derivation_method)
                        .transpose()?
                        .unwrap_or_else(default_key_derivation_method),

                    rekey: None,
                    rekey_derivation_method: default_key_derivation_method(), // default value

                    storage_credentials: None, // default value
                },
                vdrtools::types::domain::wallet::ExportConfig {
                    key: self.backup_key.clone(),
                    path: self.exported_wallet_path.clone(),

                    key_derivation_method: default_key_derivation_method(),
                },
            )
            .await?;

        Ok(())
    }
}
