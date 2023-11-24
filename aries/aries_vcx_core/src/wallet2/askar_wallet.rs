use aries_askar::{
    entry::{Entry, EntryTag, TagFilter},
    kms::{KeyAlg, KeyEntry, LocalKey, SecretBytes},
    PassKey, Session, Store, StoreKeyMethod,
};
use async_trait::async_trait;
use futures::stream::BoxStream;

use crate::{
    errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult}
};

use super::{DidWallet, RecordWallet, SigType, Wallet};

pub enum RngMethod {
    Bls,
    RandomDet,
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
}

pub struct Record {
    pub category: String,
    pub name: String,
    pub value: SecretBytes,
    pub tags: Option<Vec<EntryTag>>,
    pub expiry_ms: Option<i64>,
}

pub struct RecordId {
    name: String,
    category: String,
    for_update: bool,
}

pub struct DidAttrs {
    key_name: String,
    category: String,
    tags: Option<Vec<EntryTag>>,
    expiry_ms: Option<i64>,
}

pub struct KeyAttrs {
    name: String,
    alg: KeyAlg,
    seed: String,
    rng_method: RngMethod,
    metadata: Option<String>,
    tags: Option<Vec<EntryTag>>,
    expiry_ms: Option<i64>,
}

#[async_trait]
impl Wallet for AskarWallet {}

#[async_trait]
impl DidWallet for AskarWallet {
    type DidAttrs = DidAttrs;
    type CreatedDid = ();
    type DidKey = Option<KeyEntry>;
    type KeyAttrs = KeyAttrs;

    async fn create_key(&self, key_attrs: Self::KeyAttrs) -> Result<(), AriesVcxCoreError> {
        let mut session = self.backend.session(self.profile.clone()).await?;

        let key = LocalKey::from_seed(
            key_attrs.alg,
            key_attrs.seed.as_bytes(),
            key_attrs.rng_method.into(),
        )?;
        Ok(session
            .insert_key(
                &key_attrs.name,
                &key,
                key_attrs.metadata.as_deref(),
                key_attrs.tags.as_deref(),
                key_attrs.expiry_ms,
            )
            .await?)
    }

    async fn create_did(&self, attrs: Self::DidAttrs) -> VcxCoreResult<Self::CreatedDid> {
        let mut session = self.backend.session(self.profile.clone()).await?;

        let key_entry = self.fetch_key_entry(&mut session, &attrs.key_name).await?;

        let local_key = key_entry.load_local_key()?;

        let did_bytes = &local_key.to_public_bytes()?[0..16];

        let did = bs58::encode(did_bytes).into_string();
        Ok(session
            .insert(
                &attrs.category,
                &did,
                &did_bytes,
                attrs.tags.as_deref(),
                attrs.expiry_ms,
            )
            .await?)
    }

    async fn did_key(&self, did: &str) -> VcxCoreResult<Self::DidKey> {
        let mut session = self.backend.session(self.profile.clone()).await?;
        Ok(session.fetch_key(did, false).await?)
    }

    async fn replace_did_key(&self, did: &str) -> VcxCoreResult<Self::DidKey> {
        todo!("Not yet implemented");
    }

    async fn sign(
        &self,
        verkey_name: &str,
        msg: &[u8],
        sig_type: SigType,
    ) -> VcxCoreResult<Vec<u8>> {
        let mut session = self.backend.session(self.profile.clone()).await?;
        let res = session.fetch_key(verkey_name, false).await?;

        if let Some(key) = res {
            let local_key = key.load_local_key()?;
            let res = local_key.sign_message(msg, Some(sig_type.into()))?;
            return Ok(res);
        }

        Ok(vec![])
    }

    async fn verify(
        &self,
        verkey_name: &str,
        msg: &[u8],
        signature: &[u8],
        sig_type: SigType,
    ) -> VcxCoreResult<bool> {
        let mut session = self.backend.session(self.profile.clone()).await?;

        if let Some(key) = session.fetch_key(verkey_name, false).await? {
            let local_key = key.load_local_key()?;
            let res = local_key.verify_signature(msg, signature, Some(sig_type.into()))?;
            return Ok(res);
        }

        Ok(false)
    }
}

pub struct SearchFilter {
    category: Option<String>,
    tag_filter: Option<TagFilter>,
    offset: Option<i64>,
    limit: Option<i64>,
}

#[async_trait]
impl RecordWallet for AskarWallet {
    type Record = Record;
    type RecordId = RecordId;
    type FoundRecord = Entry;
    type SearchFilter = SearchFilter;

    async fn add_record(&self, record: Self::Record) -> VcxCoreResult<()> {
        let mut session = self.backend.session(self.profile.clone()).await?;

        Ok(session
            .insert(
                &record.category,
                &record.name,
                &record.value,
                record.tags.as_deref(),
                record.expiry_ms,
            )
            .await?)
    }

    async fn get_record(&self, id: &Self::RecordId) -> VcxCoreResult<Self::FoundRecord> {
        let mut session = self.backend.session(self.profile.clone()).await?;

        session
            .fetch(&id.category, &id.name, id.for_update)
            .await?
            .ok_or_else(|| {
                AriesVcxCoreError::from_msg(
                    AriesVcxCoreErrorKind::WalletRecordNotFound,
                    "not found",
                )
            })
    }

    async fn update_record(&self, update: Self::Record) -> VcxCoreResult<()> {
        todo!("Not yet implemented");
    }

    async fn delete_record(&self, id: &Self::RecordId) -> VcxCoreResult<()> {
        todo!("Not yet implemented");
    }

    async fn search_record(
        &self,
        filter: Self::SearchFilter,
    ) -> VcxCoreResult<BoxStream<VcxCoreResult<Self::FoundRecord>>> {
        let mut res = self
            .backend
            .scan(
                self.profile.clone(),
                filter.category,
                filter.tag_filter,
                filter.offset,
                filter.limit,
            )
            .await?;
        let mut all: Vec<VcxCoreResult<Self::FoundRecord>> = vec![];
        let rs = res
            .fetch_next()
            .await
            .map_err(|err| AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::IOError, err))?;
        if let Some(found) = rs {
            all = found.into_iter().map(|entry| Ok(entry)).collect();
        }
        Ok(Box::pin(futures::stream::iter(all)))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::StreamExt;

    use crate::wallet2::askar_wallet::AskarWallet;

    #[tokio::test]
    async fn test_askar_should_find_records() {
        let wallet = AskarWallet::create(
            "sqlite:memory:",
            StoreKeyMethod::Unprotected,
            None.into(),
            true,
            None,
        )
        .await
        .unwrap();

        let record1 = Record {
            category: "my".into(),
            name: "foofar".into(),
            tags: None,
            value: "ff".into(),
            expiry_ms: None,
        };
        wallet.add_record(record1).await.unwrap();

        let record2 = Record {
            category: "my".into(),
            name: "foobar".into(),
            tags: None,
            value: "fb".into(),
            expiry_ms: None,
        };
        wallet.add_record(record2).await.unwrap();

        let record3 = Record {
            category: "your".into(),
            name: "football".into(),
            tags: None,
            value: "fbl".into(),
            expiry_ms: None,
        };
        wallet.add_record(record3).await.unwrap();

        let filter = SearchFilter{ category: Some("my".into()), offset: None, tag_filter: None, limit: None};

        let mut res = wallet.search_record(filter).await.unwrap();

        let mut all = vec![];
        while let Some(item) = res.next().await {
            all.push(item.unwrap());
        }
        assert_eq!(2, all.len());
    }
}
