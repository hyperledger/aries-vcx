use aries_askar::{kms::SecretBytes, PassKey, Store, StoreKeyMethod};

use crate::errors::error::AriesVcxCoreError;

pub mod askar_record_wallet;

#[derive(Clone)]
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
}
