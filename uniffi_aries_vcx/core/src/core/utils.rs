use std::sync::Arc;

use super::profile::ProfileHolder;
use crate::{errors::error::VcxUniFFIResult, runtime::block_on};

pub fn unpack_message(profile_holder: Arc<ProfileHolder>, packed_msg: Vec<u8>) -> VcxUniFFIResult<Vec<u8>> {
    block_on(async {
        let wallet = profile_holder.inner.inject_wallet();
        Ok(wallet.unpack_message(&packed_msg).await?)
    })
}
