use vdrtools_sys::WalletHandle;
use crate::error::{VcxError, VcxErrorKind, VcxResult};
use crate::global::settings;
use crate::libindy::anoncreds;
use crate::libindy::utils::LibindyMock;
use crate::utils;
use crate::utils::constants::LIBINDY_CRED_OFFER;

pub async fn libindy_issuer_create_credential_offer(
    wallet_handle: WalletHandle,
    cred_def_id: &str,
) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        let rc = LibindyMock::get_result();
        if rc != 0 {
            return Err(VcxError::from(VcxErrorKind::InvalidState));
        };
        return Ok(LIBINDY_CRED_OFFER.to_string());
    }
    vdrtools::anoncreds::issuer_create_credential_offer(wallet_handle, cred_def_id)
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_issuer_create_credential(
    wallet_handle: WalletHandle,
    cred_offer_json: &str,
    cred_req_json: &str,
    cred_values_json: &str,
    rev_reg_id: Option<String>,
    tails_file: Option<String>,
) -> VcxResult<(String, Option<String>, Option<String>)> {
    if settings::indy_mocks_enabled() {
        return Ok((utils::constants::CREDENTIAL_JSON.to_owned(), None, None));
    }

    let revocation = rev_reg_id.as_deref();

    let blob_handle = match tails_file {
        Some(x) => anoncreds::blob_storage_open_reader(&x).await?,
        None => -1,
    };
    vdrtools::anoncreds::issuer_create_credential(
        wallet_handle,
        cred_offer_json,
        cred_req_json,
        cred_values_json,
        revocation,
        blob_handle,
    )
        .await
        .map_err(VcxError::from)
}
