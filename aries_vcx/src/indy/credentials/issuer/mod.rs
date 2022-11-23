use vdrtools::{
    Locator,
    CredentialOffer,
    CredentialRequest,
    CredentialValues,
    RevocationRegistryId,
};

use vdrtools::WalletHandle;

use crate::error::{VcxError, VcxErrorKind, VcxResult};
use crate::global::settings;
use crate::indy::anoncreds;
use crate::indy::utils::LibindyMock;
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

    let res = Locator::instance()
        .issuer_controller
        .create_credential_offer(
            wallet_handle,
            vdrtools::CredentialDefinitionId(cred_def_id.into()),
        )
        .await?;

    Ok(res)
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

    let blob_handle = match tails_file {
        Some(x) => Some(anoncreds::blob_storage_open_reader(&x).await?),
        None => None,
    };

    let res = Locator::instance()
        .issuer_controller
        .new_credential(
            wallet_handle,
            serde_json::from_str::<CredentialOffer>(cred_offer_json)?,
            serde_json::from_str::<CredentialRequest>(cred_req_json)?,
            serde_json::from_str::<CredentialValues>(cred_values_json)?,
            rev_reg_id.map(RevocationRegistryId),
            blob_handle,
        ).await?;

    Ok(res)
}
