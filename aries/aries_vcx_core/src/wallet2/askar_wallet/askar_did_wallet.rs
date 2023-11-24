use aries_askar::entry::{EntryTag, TagFilter};
use aries_askar::kms::{KeyAlg, LocalKey};
use async_trait::async_trait;

use serde::{Deserialize, Serialize};

use crate::errors::error::{AriesVcxCoreError, VcxCoreResult};
use crate::wallet2::askar_wallet::AskarWallet;
use crate::wallet2::{DidWallet, SigType};

use super::RngMethod;

pub struct DidEntry {
    category: String,
    name: String,
    value: DidData,
    tags: Vec<EntryTag>,
}

impl DidEntry {
    pub fn new(category: &str, name: &str, value: &DidData, tags: &Vec<EntryTag>) -> Self {
        Self {
            category: category.to_owned(),
            name: name.to_owned(),
            value: value.to_owned(),
            tags: tags.to_owned(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DidData {
    did: String,
    verkey: String,
    current: bool,
}

impl DidData {
    pub fn new(did: &str, verkey: &str, current: bool) -> Self {
        Self {
            did: did.to_owned(),
            verkey: verkey.to_owned(),
            current,
        }
    }

    pub fn is_current(&self) -> bool {
        self.current
    }
}

#[derive(Clone, Default)]
pub struct DidAttrs {
    key_name: String,
    tags: Option<Vec<EntryTag>>,
}

impl DidAttrs {
    pub fn set_key_name(mut self, key_name: &str) -> Self {
        self.key_name = key_name.to_owned();
        self
    }

    pub fn set_tags(mut self, tags: Vec<EntryTag>) -> Self {
        self.tags = Some(tags);
        self
    }
}

#[derive(Default, Clone)]
pub struct FindDidKeyAttrs {
    did: String,
    tag_filter: Option<TagFilter>,
}

impl FindDidKeyAttrs {
    pub fn set_did(mut self, did: &str) -> Self {
        self.did = did.to_owned();
        self
    }

    pub fn set_tag_filter(mut self, tag_filter: TagFilter) -> Self {
        self.tag_filter = Some(tag_filter);
        self
    }
}

impl Default for KeyAttrs {
    fn default() -> Self {
        Self {
            name: Default::default(),
            alg: KeyAlg::Ed25519,
            seed: Default::default(),
            rng_method: Default::default(),
            metadata: Default::default(),
            tags: Default::default(),
            expiry_ms: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct KeyAttrs {
    name: String,
    alg: KeyAlg,
    seed: String,
    rng_method: RngMethod,
    metadata: Option<String>,
    tags: Option<Vec<EntryTag>>,
    expiry_ms: Option<i64>,
}

impl KeyAttrs {
    pub fn set_name(mut self, name: &str) -> Self {
        self.name = name.to_owned();
        self
    }

    pub fn set_alg(mut self, alg: KeyAlg) -> Self {
        self.alg = alg;
        self
    }

    pub fn set_seed(mut self, seed: &str) -> Self {
        self.seed = seed.to_owned();
        self
    }

    pub fn set_rng_method(mut self, rng_method: RngMethod) -> Self {
        self.rng_method = rng_method;
        self
    }

    pub fn set_metadata(mut self, metadata: &str) -> Self {
        self.metadata = Some(metadata.into());
        self
    }

    pub fn set_tags(mut self, tags: Vec<EntryTag>) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn set_expiry_ms(mut self, expiry_ms: i64) -> Self {
        self.expiry_ms = Some(expiry_ms);
        self
    }
}

#[async_trait]
impl DidWallet for AskarWallet {
    type DidAttrs = DidAttrs;
    type CreatedDid = String;
    type DidKey = Option<LocalKey>;
    type KeyAttrs = KeyAttrs;
    type FindDidKeyAttrs = FindDidKeyAttrs;
    type CreatedKey = LocalKey;

    async fn create_key(&self, key_attrs: Self::KeyAttrs) -> Result<LocalKey, AriesVcxCoreError> {
        let mut session = self.open_session().await?;

        let key = LocalKey::from_seed(
            key_attrs.alg,
            key_attrs.seed.as_bytes(),
            key_attrs.rng_method.into(),
        )?;

        session
            .insert_key(
                &key_attrs.name,
                &key,
                key_attrs.metadata.as_deref(),
                key_attrs.tags.as_deref(),
                key_attrs.expiry_ms,
            )
            .await?;

        Ok(key)
    }

    async fn create_did(&self, attrs: Self::DidAttrs) -> VcxCoreResult<Self::CreatedDid> {
        let mut session = self.open_session().await?;

        let local_key = self.fetch_local_key(&mut session, &attrs.key_name).await?;

        let verkey = self.local_pubkey_as_bs58(&local_key)?;

        let did = verkey[0..16].to_string();

        self.insert_did(
            &mut session,
            &attrs.key_name,
            &did,
            &verkey,
            attrs.tags.as_deref(),
        )
        .await?;
        Ok(did)
    }

    async fn did_key(&self, attrs: Self::FindDidKeyAttrs) -> VcxCoreResult<Self::DidKey> {
        let mut session = self.open_session().await?;

        let data = self
            .find_did_entry(&mut session, &attrs.did, attrs.tag_filter)
            .await?;

        if let Some(entry) = data {
            let local_key = self.fetch_local_key(&mut session, &entry.name).await?;
            return Ok(Some(local_key));
        }

        Ok(None)
    }

    async fn replace_did_key(
        &self,
        attrs: Self::FindDidKeyAttrs,
        key_name: &str,
    ) -> VcxCoreResult<Self::DidKey> {
        let mut tx = self.backend.transaction(self.profile.clone()).await?;

        let data = self
            .find_did_entry(&mut tx, &attrs.did, attrs.tag_filter)
            .await?;

        if let Some(mut did_entry) = data {
            let local_key = self.fetch_local_key(&mut tx, key_name).await?;

            let verkey = self.local_pubkey_as_bs58(&local_key)?;

            self.insert_did(
                &mut tx,
                key_name,
                &attrs.did,
                &verkey,
                Some(did_entry.tags.clone()).as_deref(),
            )
            .await?;

            did_entry.value.current = false;
            let did_entry_data = serde_json::to_string(&did_entry.value)?;

            tx.replace(
                &did_entry.category,
                &did_entry.name,
                did_entry_data.as_bytes(),
                Some(&did_entry.tags),
                None,
            )
            .await?;
            tx.commit().await?;
            return Ok(Some(local_key));
        }

        Ok(None)
    }

    async fn sign(&self, key_name: &str, msg: &[u8], sig_type: SigType) -> VcxCoreResult<Vec<u8>> {
        let mut session = self.open_session().await?;
        let res = session.fetch_key(key_name, false).await?;

        if let Some(key) = res {
            let local_key = key.load_local_key()?;
            let res = local_key.sign_message(msg, Some(sig_type.into()))?;
            return Ok(res);
        }

        Ok(vec![])
    }

    async fn verify(
        &self,
        key_name: &str,
        msg: &[u8],
        signature: &[u8],
        sig_type: SigType,
    ) -> VcxCoreResult<bool> {
        let mut session = self.open_session().await?;

        if let Some(key) = session.fetch_key(key_name, false).await? {
            let local_key = key.load_local_key()?;
            let res = local_key.verify_signature(msg, signature, Some(sig_type.into()))?;
            return Ok(res);
        }

        Ok(false)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use aries_askar::StoreKeyMethod;
    use uuid::Uuid;

    use crate::{errors::error::AriesVcxCoreErrorKind, wallet2::askar_wallet::AskarWallet};

    async fn create_test_wallet() -> AskarWallet {
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

    #[tokio::test]
    async fn test_askar_should_sign_and_verify() {
        let wallet = create_test_wallet().await;

        let first_key_name = "first".to_string();
        let first_key_attrs = KeyAttrs::default()
            .set_name(&first_key_name)
            .set_seed("foo");

        wallet.create_key(first_key_attrs).await.unwrap();

        let second_key_name = "second".to_string();
        let second_key_attrs = KeyAttrs::default()
            .set_name(&second_key_name)
            .set_seed("bar");

        wallet.create_key(second_key_attrs).await.unwrap();

        let msg = "sign this message";
        let sig = wallet
            .sign(&first_key_name, msg.as_bytes(), SigType::EdDSA)
            .await
            .unwrap();

        assert!(wallet
            .verify(&first_key_name, msg.as_bytes(), &sig, SigType::EdDSA)
            .await
            .unwrap());
        assert!(!wallet
            .verify(&second_key_name, msg.as_bytes(), &sig, SigType::EdDSA)
            .await
            .unwrap());

        let err = wallet
            .verify(&first_key_name, msg.as_bytes(), &sig, SigType::ES384)
            .await
            .unwrap_err();

        assert_eq!(AriesVcxCoreErrorKind::WalletUnexpected, err.kind());
        assert!(err.to_string().contains("Unsupported signature type"));
    }

    #[tokio::test]
    async fn test_askar_should_replace_did_key() {
        let wallet = create_test_wallet().await;

        let first_key_name = "first".to_string();
        let first_key_attrs = KeyAttrs::default()
            .set_name(&first_key_name)
            .set_seed("foo");

        wallet.create_key(first_key_attrs).await.unwrap();

        let second_key_name = "second".to_string();
        let second_key_attrs = KeyAttrs::default()
            .set_name(&second_key_name)
            .set_seed("bar");

        wallet.create_key(second_key_attrs).await.unwrap();

        let did_attrs = DidAttrs::default().set_key_name(&first_key_name);

        let did = wallet.create_did(did_attrs.clone()).await.unwrap();
        let find_did_attrs = FindDidKeyAttrs::default().set_did(&did);

        let second_key = wallet
            .replace_did_key(find_did_attrs.clone(), &second_key_name)
            .await
            .unwrap()
            .unwrap();

        let third_key_name = "third".to_string();
        let third_key_attrs = KeyAttrs::default()
            .set_name(&third_key_name)
            .set_seed("baz");

        wallet.create_key(third_key_attrs).await.unwrap();

        let third_key = wallet
            .replace_did_key(find_did_attrs, &third_key_name)
            .await
            .unwrap()
            .unwrap();

        let did_key_attrs = FindDidKeyAttrs::default().set_did(&did);
        let last_key = wallet.did_key(did_key_attrs).await.unwrap().unwrap();

        assert_eq!(
            wallet.local_pubkey_as_bs58(&third_key).unwrap(),
            wallet.local_pubkey_as_bs58(&last_key).unwrap()
        );
        assert_ne!(
            wallet.local_pubkey_as_bs58(&second_key).unwrap(),
            wallet.local_pubkey_as_bs58(&last_key).unwrap()
        );
    }

    #[tokio::test]
    async fn test_askar_should_not_create_key_repeatedly() {
        let wallet = create_test_wallet().await;

        let first_key_name = "first".to_string();
        let first_key_attrs = KeyAttrs::default().set_name(&first_key_name);
        wallet.create_key(first_key_attrs.clone()).await.unwrap();
        let create_err = wallet.create_key(first_key_attrs).await.unwrap_err();

        assert_eq!(
            AriesVcxCoreErrorKind::DuplicationWalletRecord,
            create_err.kind()
        );
    }
}
