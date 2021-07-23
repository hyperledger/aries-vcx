use crate::error::VcxResult;
use crate::libindy::utils::signus::create_and_store_my_did;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub fn create() -> VcxResult<PairwiseInfo> {
        let (pw_did, pw_vk) = create_and_store_my_did(None, None)?;
        Ok(PairwiseInfo { pw_did, pw_vk })
    }
}
