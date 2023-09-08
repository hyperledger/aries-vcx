use std::sync::Arc;

use aries_vcx::errors::error::{AriesVcxError, AriesVcxErrorKind};
use vdrtools::domain::crypto::pack::UnpackMessage;

use super::profile::ProfileHolder;
use crate::{errors::error::VcxUniFFIResult, runtime::block_on};

pub fn unpack_message(profile_holder: Arc<ProfileHolder>, packed_msg: Vec<u8>) -> VcxUniFFIResult<Vec<u8>> {
    block_on(async {
        let wallet = profile_holder.inner.inject_wallet();
        Ok(wallet.unpack_message(&packed_msg).await?)
    })
}

pub fn unpack_message_and_return_message(
    profile_holder: Arc<ProfileHolder>,
    packed_msg: String,
) -> VcxUniFFIResult<String> {
    block_on(async {
        let packed_bytes = packed_msg.as_bytes();
        let wallet = profile_holder.inner.inject_wallet();
        let unpacked_bytes = wallet.unpack_message(&packed_bytes).await?;
        let unpacked_string = match String::from_utf8(unpacked_bytes) {
            Ok(str) => str,
            Err(err) => todo!(),
        };
        let value = match serde_json::from_str::<UnpackMessage>(&unpacked_string) {
            Ok(str) => str,
            Err(err) => todo!(),
        };
        Ok(value.message)
    })
}
