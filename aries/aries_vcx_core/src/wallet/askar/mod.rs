use aries_askar::{
    entry::EntryTag,
    kms::{KeyAlg, KeyEntry, LocalKey},
    PassKey, Session, Store, StoreKeyMethod,
};
use public_key::Key;

use self::{askar_utils::local_key_to_bs58_name, rng_method::RngMethod};
use super::base_wallet::{did_value::DidValue, record_category::RecordCategory, BaseWallet};
use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};

mod askar_did_wallet;
mod askar_record_wallet;
mod askar_utils;
mod crypto_box;
mod entry;
mod entry_tags;
mod pack;
mod packing_types;
mod rng_method;
mod sig_type;
mod unpack;

#[derive(Debug)]
pub struct AskarWallet {
    backend: Store,
    profile: String,
}

impl BaseWallet for AskarWallet {}

impl AskarWallet {
    pub async fn create(
        db_url: &str,
        key_method: StoreKeyMethod,
        pass_key: PassKey<'_>,
        recreate: bool,
        profile: &str,
    ) -> Result<Self, AriesVcxCoreError> {
        let backend =
            Store::provision(db_url, key_method, pass_key, Some(profile.into()), recreate).await?;

        Ok(Self {
            backend,
            profile: profile.into(),
        })
    }

    pub async fn open(
        db_url: &str,
        key_method: Option<StoreKeyMethod>,
        pass_key: PassKey<'_>,
        profile: &str,
    ) -> Result<Self, AriesVcxCoreError> {
        Ok(Self {
            backend: Store::open(db_url, key_method, pass_key, Some(profile.into())).await?,
            profile: profile.into(),
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
    use crate::wallet::base_wallet::BaseWallet;

    pub async fn dev_setup_askar_wallet() -> Box<dyn BaseWallet> {
        use aries_askar::StoreKeyMethod;
        use uuid::Uuid;

        use crate::wallet::askar::AskarWallet;

        Box::new(
            AskarWallet::create(
                "sqlite://:memory:",
                StoreKeyMethod::Unprotected,
                None.into(),
                true,
                &Uuid::new_v4().to_string(),
            )
            .await
            .unwrap(),
        )
    }
}
