use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::profile::ProfileHolder;
use crate::{errors::error::VcxUniFFIResult, runtime::block_on};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct UnpackMessage {
    pub message: String,
    pub recipient_verkey: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_verkey: Option<String>,
}

pub fn unpack_message(
    profile_holder: Arc<ProfileHolder>,
    packed_msg: String,
) -> VcxUniFFIResult<UnpackMessage> {
    block_on(async {
        let packed_bytes = packed_msg.as_bytes();
        let wallet = profile_holder.inner.inject_wallet();
        let unpacked_bytes = wallet.unpack_message(&packed_bytes).await?;
        let unpacked_string = String::from_utf8(unpacked_bytes)?;
        let unpacked_message = serde_json::from_str::<UnpackMessage>(&unpacked_string)?;
        Ok(unpacked_message)
    })
}
