use async_trait::async_trait;
use serde::Deserialize;

use super::{key_method::KeyMethod, AskarWallet};
use crate::{errors::error::VcxCoreResult, wallet::base_wallet::ManageWallet};

#[derive(Clone, Debug, Deserialize)]
pub struct AskarWalletConfig {
    pub db_url: String,
    pub key_method: KeyMethod,
    pub pass_key: String,
    pub profile: String,
}

impl AskarWalletConfig {
    pub fn new(db_url: &str, key_method: KeyMethod, pass_key: &str, profile: &str) -> Self {
        Self {
            db_url: db_url.into(),
            key_method,
            pass_key: pass_key.into(),
            profile: profile.into(),
        }
    }

    pub fn db_url(&self) -> &str {
        &self.db_url
    }

    pub fn key_method(&self) -> &KeyMethod {
        &self.key_method
    }

    pub fn pass_key(&self) -> &str {
        &self.pass_key
    }

    pub fn profile(&self) -> &str {
        &self.profile
    }
}

#[async_trait]
impl ManageWallet for AskarWalletConfig {
    type ManagedWalletType = AskarWallet;

    async fn create_wallet(&self) -> VcxCoreResult<Self::ManagedWalletType> {
        let askar_wallet = AskarWallet::create(self, false).await?;
        Ok(askar_wallet)
    }

    async fn open_wallet(&self) -> VcxCoreResult<Self::ManagedWalletType> {
        Ok(AskarWallet::open(self).await?)
    }

    async fn delete_wallet(&self) -> VcxCoreResult<()> {
        todo!();
    }
}
