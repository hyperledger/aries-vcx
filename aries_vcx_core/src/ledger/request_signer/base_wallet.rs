use std::sync::Arc;

use async_trait::async_trait;
use indy_vdr::pool::PreparedRequest;

use crate::{errors::error::VcxCoreResult, wallet::base_wallet::BaseWallet};

use super::RequestSigner;

pub struct BaseWalletRequestSigner {
    wallet: Arc<dyn BaseWallet>,
}

impl BaseWalletRequestSigner {
    pub fn new(wallet: Arc<dyn BaseWallet>) -> Self {
        Self { wallet }
    }
}

#[async_trait]
impl RequestSigner for BaseWalletRequestSigner {
    async fn sign(&self, did: &str, request: &PreparedRequest) -> VcxCoreResult<Vec<u8>> {
        let to_sign = request.get_signature_input()?;
        let signer_verkey = self.wallet.key_for_local_did(did).await?;
        let signature = self.wallet.sign(&signer_verkey, to_sign.as_bytes()).await?;
        Ok(signature)
    }
}
