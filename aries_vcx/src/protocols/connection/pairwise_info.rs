use vdrtools_sys::WalletHandle;

use crate::error::VcxResult;
use crate::indy::keys::create_and_store_my_did;

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
    pub async fn create(wallet_handle: WalletHandle) -> VcxResult<PairwiseInfo> {
        let (pw_did, pw_vk) = create_and_store_my_did(wallet_handle, None, None).await?;
        Ok(PairwiseInfo { pw_did, pw_vk })
    }
}
