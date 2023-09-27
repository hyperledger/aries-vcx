use aries_vcx_core::wallet::base_wallet::BaseWallet;

use crate::errors::error::VcxResult;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PairwiseInfo {
    pub pw_did: String,
    pub pw_vk: String,
}

impl PairwiseInfo {
    pub async fn create(wallet: &impl BaseWallet) -> VcxResult<PairwiseInfo> {
        let (pw_did, pw_vk) = wallet.create_and_store_my_did(None, None).await?;
        Ok(PairwiseInfo { pw_did, pw_vk })
    }
}
