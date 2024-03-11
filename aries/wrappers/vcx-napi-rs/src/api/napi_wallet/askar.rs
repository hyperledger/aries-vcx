use std::{ops::Deref, sync::Arc};

use libvcx_core::aries_vcx::aries_vcx_core::wallet::askar::AskarWallet;
use napi_derive::napi;

#[napi]
pub struct NapiWallet(Arc<AskarWallet>);

impl NapiWallet {
    pub fn new(wallet: Arc<AskarWallet>) -> Self {
        Self(wallet)
    }
}

impl Deref for NapiWallet {
    type Target = Arc<AskarWallet>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
