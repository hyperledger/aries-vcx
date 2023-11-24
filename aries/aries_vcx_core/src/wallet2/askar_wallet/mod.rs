use aries_askar::{
    entry::{EntryTag, TagFilter},
    kms::{KeyEntry, LocalKey},
    PassKey, Session, Store, StoreKeyMethod,
};

use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};

use self::askar_did_wallet::{DidData, DidEntry};

pub mod askar_did_wallet;

#[derive(Clone, Default)]
pub enum RngMethod {
    #[default]
    RandomDet,
    Bls,
}

impl From<RngMethod> for Option<&str> {
    fn from(value: RngMethod) -> Self {
        match value {
            RngMethod::RandomDet => None,
            RngMethod::Bls => Some("bls_keygen"),
        }
    }
}

#[derive(Debug)]
pub struct AskarWallet {
    backend: Store,
    profile: Option<String>,
}

impl AskarWallet {
    pub async fn create(
        db_url: &str,
        key_method: StoreKeyMethod,
        pass_key: PassKey<'_>,
        recreate: bool,
        profile: Option<String>,
    ) -> Result<Self, AriesVcxCoreError> {
        let backend =
            Store::provision(db_url, key_method, pass_key, profile.clone(), recreate).await?;

        Ok(Self { backend, profile })
    }

    pub async fn open(
        db_url: &str,
        key_method: Option<StoreKeyMethod>,
        pass_key: PassKey<'_>,
        profile: Option<String>,
    ) -> Result<Self, AriesVcxCoreError> {
        let backend = Store::open(db_url, key_method, pass_key, profile.clone()).await?;

        Ok(Self { backend, profile })
    }

    pub fn local_pubkey_as_bs58(&self, local_key: &LocalKey) -> VcxCoreResult<String> {
        Ok(bs58::encode(local_key.to_public_bytes()?).into_string())
    }

    async fn open_session(&self) -> VcxCoreResult<Session> {
        Ok(self.backend.session(self.profile.clone()).await?)
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

    async fn fetch_local_key(
        &self,
        session: &mut Session,
        key_name: &str,
    ) -> VcxCoreResult<LocalKey> {
        let key_entry = self.fetch_key_entry(session, &key_name).await?;

        Ok(key_entry.load_local_key()?)
    }

    async fn find_did_entry(
        &self,
        session: &mut Session,
        did: &str,
        tag_filter: Option<TagFilter>,
    ) -> VcxCoreResult<Option<DidEntry>> {
        let entries = session
            .fetch_all(Some(&did), tag_filter, None, false)
            .await?;

        for entry in entries.iter() {
            if let Some(val) = entry.value.as_opt_str() {
                let res: DidData = serde_json::from_str(val)?;
                if res.is_current() {
                    return Ok(Some(DidEntry::new(
                        &entry.category,
                        &entry.name,
                        &res,
                        &entry.tags,
                    )));
                }
            }
        }

        Ok(None)
    }

    async fn insert_did(
        &self,
        session: &mut Session,
        key_name: &str,
        did: &str,
        verkey: &str,
        tags: Option<&[EntryTag]>,
    ) -> VcxCoreResult<()> {
        if let Some(_) = session.fetch(&did, key_name, false).await? {
            return Err(AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::DuplicationDid,
                "did with given verkey already exists",
            ));
        }

        let did_data = DidData::new(did, verkey, true);
        let did_data = serde_json::to_string(&did_data)?;

        let res = session
            .insert(&did, key_name, did_data.as_bytes(), tags, None)
            .await?;

        Ok(res)
    }
}
