use std::sync::Arc;

use crate::error::VcxResult;
use crate::plugins::wallet::base_wallet::BaseWallet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PairwiseInfo {
    pub pw_did: String,
    pub pw_vk: String,
}

impl Default for PairwiseInfo {
    fn default() -> PairwiseInfo {
        PairwiseInfo {
            pw_did: String::new(),
            pw_vk: String::new(),
        }
    }
}

impl PairwiseInfo {
    pub async fn create(wallet: &Arc<dyn BaseWallet>) -> VcxResult<PairwiseInfo> {
        let (pw_did, pw_vk) = wallet.create_and_store_my_did(None, None).await?;
        Ok(PairwiseInfo { pw_did, pw_vk })
    }
}
