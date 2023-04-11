use std::sync::Arc;

use aries_vcx::aries_vcx_core::indy::wallet::{create_and_open_wallet, WalletConfig};
use aries_vcx::core::profile::{profile::Profile, vdrtools_profile::VdrtoolsProfile};

use crate::{errors::error::VcxUniFFIResult, runtime::block_on};

pub struct ProfileHolder {
    pub inner: Arc<dyn Profile>,
}

impl ProfileHolder {}

pub fn new_indy_profile(wallet_config: WalletConfig) -> VcxUniFFIResult<Arc<ProfileHolder>> {
    block_on(async {
        let wh = create_and_open_wallet(&wallet_config).await?;
        let ph = 0;
        let profile = VdrtoolsProfile::new(wh, ph);

        Ok(Arc::new(ProfileHolder {
            inner: Arc::new(profile),
        }))
    })
}
