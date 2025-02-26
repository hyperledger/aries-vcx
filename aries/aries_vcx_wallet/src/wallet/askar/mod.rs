use aries_askar::{
    entry::{Entry, EntryTag},
    kms::{KeyAlg, KeyEntry, LocalKey},
    Session, Store,
};
use async_trait::async_trait;
use public_key::Key;

use self::{askar_utils::local_key_to_bs58_public_key, askar_wallet_config::AskarWalletConfig};
use super::{
    base_wallet::{
        did_value::DidValue, key_value::KeyValue, record_category::RecordCategory, BaseWallet,
    },
    record_tags::RecordTags,
};
use crate::errors::error::{VcxWalletError, VcxWalletResult};

mod all_askar_records;
mod askar_did_wallet;
pub mod askar_import_config;
mod askar_record_wallet;
mod askar_utils;
pub mod askar_wallet_config;
mod entry;
mod entry_tags;
pub mod key_method;
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
    async fn export_wallet(&self, _path: &str, _backup_key: &str) -> VcxWalletResult<()> {
        todo!()
    }

    async fn close_wallet(&self) -> VcxWalletResult<()> {
        todo!()
    }

    async fn create_key(
        &self,
        name: &str,
        value: KeyValue,
        tags: &RecordTags,
    ) -> VcxWalletResult<()> {
        let mut session = self.session().await?;
        let tg: Vec<_> = tags.clone().into();
        let key = LocalKey::from_secret_bytes(KeyAlg::Ed25519, &value.signkey().decode()?[0..32])?;
        Ok(session
            .insert_key(name, &key, None, None, Some(&tg), None)
            .await?)
    }
}

impl AskarWallet {
    pub async fn create(
        wallet_config: &AskarWalletConfig,
        recreate: bool,
    ) -> VcxWalletResult<Self> {
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

    pub async fn open(wallet_config: &AskarWalletConfig) -> VcxWalletResult<Self> {
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

    async fn fetch(
        &self,
        session: &mut Session,
        category: RecordCategory,
        name: &str,
        for_update: bool,
    ) -> VcxWalletResult<Entry> {
        let maybe_entry = session
            .fetch(&category.to_string(), name, for_update)
            .await
            .map_err(|err| match err.kind() {
                aries_askar::ErrorKind::NotFound => {
                    VcxWalletError::record_not_found_from_details(category, name)
                }
                _ => err.into(),
            })?;

        maybe_entry.ok_or_else(|| VcxWalletError::record_not_found_from_details(category, name))
    }

    async fn fetch_local_key(
        &self,
        session: &mut Session,
        key_name: &str,
    ) -> VcxWalletResult<LocalKey> {
        Ok(self
            .fetch_key_entry(session, key_name)
            .await?
            .load_local_key()?)
    }

    async fn fetch_key_entry(
        &self,
        session: &mut Session,
        key_name: &str,
    ) -> VcxWalletResult<KeyEntry> {
        session.fetch_key(key_name, false).await?.ok_or_else(|| {
            VcxWalletError::record_not_found_from_details(RecordCategory::Key, key_name)
        })
    }

    async fn insert_key(
        &self,
        session: &mut Session,
        alg: KeyAlg,
        seed: &[u8],
    ) -> VcxWalletResult<(String, LocalKey)> {
        let key = LocalKey::from_secret_bytes(alg, seed)?;
        let key_name = local_key_to_bs58_public_key(&key)?.into_inner();
        session
            .insert_key(&key_name, &key, None, None, None, None)
            .await?;
        Ok((key_name, key))
    }

    async fn find_did(
        &self,
        session: &mut Session,
        did: &str,
        category: RecordCategory,
    ) -> VcxWalletResult<Option<DidValue>> {
        let entry = self.fetch(session, category, did, false).await?;

        if let Some(val) = entry.value.as_opt_str() {
            Ok(serde_json::from_str(val)?)
        } else {
            Err(VcxWalletError::InvalidInput(
                "wallet entry value is not a valid character sequence".into(),
            ))
        }
    }

    async fn find_current_did(
        &self,
        session: &mut Session,
        did: &str,
    ) -> VcxWalletResult<Option<DidValue>> {
        self.find_did(session, did, RecordCategory::Did).await
    }

    async fn insert_did(
        &self,
        session: &mut Session,
        did: &str,
        category: &str,
        verkey: &Key,
        tags: Option<&[EntryTag]>,
    ) -> VcxWalletResult<()> {
        if (session.fetch(did, category, false).await?).is_some() {
            Err(VcxWalletError::DuplicateRecord(format!(
                "category: {}, name: {}",
                RecordCategory::Did,
                category
            )))
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
    ) -> VcxWalletResult<()> {
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

    async fn session(&self) -> VcxWalletResult<Session> {
        Ok(self.backend.session(Some(self.profile.clone())).await?)
    }

    async fn transaction(&self) -> VcxWalletResult<Session> {
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
