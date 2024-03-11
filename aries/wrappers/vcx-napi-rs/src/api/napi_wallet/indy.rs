use std::{ops::Deref, sync::Arc};

use libvcx_core::aries_vcx::aries_vcx_core::wallet::indy::IndySdkWallet;
use napi_derive::napi;

#[napi]
pub struct NapiWallet(Arc<IndySdkWallet>);

impl NapiWallet {
    pub fn new(wallet: Arc<IndySdkWallet>) -> Self {
        Self(wallet)
    }
}

impl Deref for NapiWallet {
    type Target = Arc<IndySdkWallet>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
