use aries_askar::kms::LocalKey;

use crate::errors::error::VcxCoreResult;

pub fn local_key_to_private_key_bytes(local_key: &LocalKey) -> VcxCoreResult<Vec<u8>> {
    Ok(local_key.to_secret_bytes()?.to_vec())
}

pub fn local_key_to_public_key_bytes(local_key: &LocalKey) -> VcxCoreResult<Vec<u8>> {
    Ok(local_key.to_public_bytes()?.to_vec())
}
