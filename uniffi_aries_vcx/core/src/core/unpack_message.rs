use std::sync::Arc;

use aries_vcx::aries_vcx_core::wallet::structs_io::UnpackMessageOutput;

use super::profile::ProfileHolder;
use crate::{errors::error::VcxUniFFIResult, runtime::block_on};

pub fn unpack_message(
    profile_holder: Arc<ProfileHolder>,
    packed_msg: String,
) -> VcxUniFFIResult<UnpackMessageOutput> {
    block_on(async {
        let packed_bytes = packed_msg.as_bytes();
        let wallet = profile_holder.inner.inject_wallet();
        let unpacked_message = wallet.unpack_message(packed_bytes).await?;
        Ok(unpacked_message)
    })
}
