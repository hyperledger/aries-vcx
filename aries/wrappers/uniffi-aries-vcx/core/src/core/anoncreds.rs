use std::sync::Arc;

use aries_vcx::aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;

use super::profile::ProfileHolder;

use crate::{errors::error::VcxUniFFIResult, runtime::block_on};

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
