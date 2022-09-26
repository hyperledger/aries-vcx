use vdrtools_sys::WalletHandle;
use vdrtools::anoncreds;
use crate::error::{VcxError, VcxResult};

pub async fn libindy_create_and_store_credential_def(
    wallet_handle: WalletHandle,
    issuer_did: &str,
    schema_json: &str,
    tag: &str,
    sig_type: Option<&str>,
    config_json: &str,
) -> VcxResult<(String, String)> {
    anoncreds::issuer_create_and_store_credential_def(
        wallet_handle,
        issuer_did,
        schema_json,
        tag,
        sig_type,
        config_json,
    )
        .await
        .map_err(VcxError::from)
}
