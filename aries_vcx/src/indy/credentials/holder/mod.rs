use vdrtools::{
    Credential, CredentialDefinition, CredentialOffer, CredentialRequestMetadata, DidValue, Locator,
    RevocationRegistryDefinition,
};

use crate::errors::error::VcxResult;
use crate::global::settings;
use crate::utils;
use vdrtools::WalletHandle;

pub async fn libindy_prover_create_master_secret(
    wallet_handle: WalletHandle,
    master_secret_id: &str,
) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok(settings::DEFAULT_LINK_SECRET_ALIAS.to_string());
    }

    let res = Locator::instance()
        .prover_controller
        .create_master_secret(wallet_handle, Some(master_secret_id.into()))
        .await?;

    Ok(res)
}
