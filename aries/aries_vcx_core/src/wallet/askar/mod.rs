use aries_askar::{
    entry::EntryTag,
    kms::{KeyAlg, KeyEntry, LocalKey},
    Session, Store,
};
use async_trait::async_trait;
use public_key::Key;

use self::{
    askar_utils::local_key_to_bs58_name, askar_wallet_config::AskarWalletConfig,
    rng_method::RngMethod,
};
use super::base_wallet::{did_value::DidValue, record_category::RecordCategory, BaseWallet};
use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};

mod all_askar_records;
mod askar_did_wallet;
pub mod askar_import_config;
mod askar_record_wallet;
mod askar_utils;
pub mod askar_wallet_config;
mod entry;
mod entry_tags;
pub mod key_method;
mod key_value;
mod pack;
mod packing_types;
mod partial_record;
mod rng_method;
mod sig_type;
mod unpack;

#[derive(Debug)]
pub struct AskarWallet {
    backend: Store,
    profile: String,
}

#[async_trait]
impl BaseWallet for AskarWallet {
    async fn export_wallet(&self, _path: &str, _backup_key: &str) -> VcxCoreResult<()> {
        todo!()
    }

    async fn close_wallet(&self) -> VcxCoreResult<()> {
        todo!()
    }
}

impl AskarWallet {
    pub async fn create(
        wallet_config: &AskarWalletConfig,
        recreate: bool,
    ) -> Result<Self, AriesVcxCoreError> {
        let backend = Store::provision(
            wallet_config.db_url(),
            (*wallet_config.key_method()).into(),
            wallet_config.pass_key().into(),
            Some(wallet_config.profile().to_owned()),
            recreate,
        )
        .await?;

        Ok(Self {
            backend,
            profile: wallet_config.profile().into(),
        })
    }

    pub async fn open(wallet_config: &AskarWalletConfig) -> Result<Self, AriesVcxCoreError> {
        Ok(Self {
            backend: Store::open(
                wallet_config.db_url(),
                Some((*wallet_config.key_method()).into()),
                wallet_config.pass_key().into(),
                Some(wallet_config.profile().into()),
            )
            .await?,
            profile: wallet_config.profile().into(),
        })
    }

    async fn fetch_local_key(
        &self,
        session: &mut Session,
        key_name: &str,
    ) -> VcxCoreResult<LocalKey> {
        Ok(self
            .fetch_key_entry(session, key_name)
            .await?
            .load_local_key()?)
    }

    async fn fetch_key_entry(
        &self,
        session: &mut Session,
        key_name: &str,
    ) -> Result<KeyEntry, AriesVcxCoreError> {
        session.fetch_key(key_name, false).await?.ok_or_else(|| {
            AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::WalletRecordNotFound,
                format!("no key with name '{}' found in wallet", key_name),
            )
        })
    }

    async fn insert_key(
        &self,
        session: &mut Session,
        alg: KeyAlg,
        seed: &[u8],
        rng_method: RngMethod,
    ) -> Result<(String, LocalKey), AriesVcxCoreError> {
        let key = LocalKey::from_seed(alg, seed, rng_method.into())?;
        let key_name = local_key_to_bs58_name(&key)?;
        session
            .insert_key(&key_name, &key, None, None, None)
            .await?;
        Ok((key_name, key))
    }

    async fn find_did(
        &self,
        session: &mut Session,
        did: &str,
        category: RecordCategory,
    ) -> VcxCoreResult<Option<DidValue>> {
        if let Some(entry) = session.fetch(&category.to_string(), did, false).await? {
            if let Some(val) = entry.value.as_opt_str() {
                return Ok(Some(serde_json::from_str(val)?));
            }
        }

        Ok(None)
    }

    async fn find_current_did(
        &self,
        session: &mut Session,
        did: &str,
    ) -> VcxCoreResult<Option<DidValue>> {
        self.find_did(session, did, RecordCategory::Did).await
    }

    async fn insert_did(
        &self,
        session: &mut Session,
        did: &str,
        category: &str,
        verkey: &Key,
        tags: Option<&[EntryTag]>,
    ) -> VcxCoreResult<()> {
        if (session.fetch(did, category, false).await?).is_some() {
            Err(AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::DuplicationDid,
                "did with given verkey already exists",
            ))
        } else {
            Ok(session
                .insert(
                    category,
                    did,
                    serde_json::to_string(&DidValue::new(verkey))?.as_bytes(),
                    tags,
                    None,
                )
                .await?)
        }
    }

    async fn update_did(
        &self,
        session: &mut Session,
        did: &str,
        category: &str,
        verkey: &Key,
        tags: Option<&[EntryTag]>,
    ) -> VcxCoreResult<()> {
        session
            .replace(
                category,
                did,
                serde_json::to_string(&DidValue::new(verkey))?.as_bytes(),
                tags,
                None,
            )
            .await?;

        Ok(())
    }

    async fn session(&self) -> VcxCoreResult<Session> {
        Ok(self.backend.session(Some(self.profile.clone())).await?)
    }

    async fn transaction(&self) -> VcxCoreResult<Session> {
        Ok(self.backend.transaction(Some(self.profile.clone())).await?)
    }
}

#[cfg(test)]
pub mod tests {
    use super::AskarWallet;
    use crate::wallet::{
        askar::{askar_wallet_config::AskarWalletConfig, key_method::KeyMethod},
        base_wallet::ManageWallet,
    };

    pub async fn dev_setup_askar_wallet() -> AskarWallet {
        use uuid::Uuid;

        let config = AskarWalletConfig::new(
            "sqlite://:memory:",
            KeyMethod::Unprotected,
            "",
            &Uuid::new_v4().to_string(),
        );

        config.create_wallet().await.unwrap()
    }
}
