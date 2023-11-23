use std::sync::Arc;

use crate::{errors::error::VcxUniFFIResult, runtime::block_on};

use super::profile::ProfileHolder;
use aries_vcx::aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;

pub fn get_credentials(profile_holder: Arc<ProfileHolder>) -> VcxUniFFIResult<String> {
    block_on(async {
        let credentials = profile_holder
            .inner
            .anoncreds()
            .prover_get_credentials(profile_holder.inner.wallet(), Some("{}"))
            .await?;
        Ok(credentials)
    })
}
