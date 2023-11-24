use aries_askar::{
    entry::EntryTag,
    kms::{KeyAlg, KeyEntry, LocalKey},
    PassKey, Session, Store, StoreKeyMethod,
};

use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};

use super::DidData;

pub mod askar_did_wallet;
pub mod askar_record_wallet;

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
    pub backend: Store,
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

    async fn fetch_local_key(
        &self,
        session: &mut Session,
        key_name: &str,
    ) -> VcxCoreResult<LocalKey> {
        let key_entry = self.fetch_key_entry(session, &key_name).await?;

        Ok(key_entry.load_local_key()?)
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

    fn local_key_to_bs58_pubkey(&self, local_key: &LocalKey) -> VcxCoreResult<String> {
        Ok(bs58::encode(local_key.to_public_bytes()?).into_string())
    }

    fn local_key_to_bs58_name(&self, local_key: &LocalKey) -> VcxCoreResult<String> {
        let public_bytes = local_key.to_public_bytes()?;
        let res = &bs58::encode(public_bytes).into_string()[0..16];
        Ok(res.to_string())
    }

    async fn insert_key(
        &self,
        session: &mut Session,
        alg: KeyAlg,
        seed: &[u8],
        rng_method: RngMethod,
    ) -> Result<(String, LocalKey), AriesVcxCoreError> {
        let key = LocalKey::from_seed(alg, seed, rng_method.into())?;

        let key_name = self.local_key_to_bs58_name(&key)?;

        session
            .insert_key(&key_name, &key, None, None, None)
            .await?;

        Ok((key_name, key))
    }

    async fn find_did(&self, session: &mut Session, did: &str) -> VcxCoreResult<Option<DidData>> {
        let entries = session.fetch_all(Some(&did), None, None, false).await?;

        for entry in entries.iter() {
            if let Some(val) = entry.value.as_opt_str() {
                let res: DidData = serde_json::from_str(val)?;
                return Ok(Some(res));
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

        let did_data = DidData {
            did: did.into(),
            verkey: verkey.into(),
        };
        let did_data = serde_json::to_string(&did_data)?;

        let res = session
            .insert(key_name, did, did_data.as_bytes(), tags, None)
            .await?;

        Ok(res)
    }
}

pub(crate) mod test_helper {
    use aries_askar::StoreKeyMethod;
    use uuid::Uuid;

    use super::AskarWallet;

    pub async fn create_test_wallet() -> AskarWallet {
        AskarWallet::create(
            "sqlite://:memory:",
            StoreKeyMethod::Unprotected,
            None.into(),
            true,
            Some(Uuid::new_v4().to_string()),
        )
        .await
        .unwrap()
    }
}
