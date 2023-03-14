use std::sync::Arc;

use crate::{errors::error::VcxResult, plugins::wallet::base_wallet::BaseWallet};

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PairwiseInfo {
    pub pw_did: String,
    pub pw_vk: String,
}

impl PairwiseInfo {
    pub async fn create(wallet: &Arc<dyn BaseWallet>) -> VcxResult<PairwiseInfo> {
        let (pw_did, pw_vk) = wallet.create_and_store_my_did(None, None).await?;
        Ok(PairwiseInfo { pw_did, pw_vk })
    }
}
