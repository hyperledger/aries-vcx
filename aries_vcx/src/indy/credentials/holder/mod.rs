use vdrtools::{
    Credential, CredentialDefinition, CredentialOffer, CredentialRequestMetadata, DidValue, Locator,
    RevocationRegistryDefinition, WalletHandle,
};

use crate::{errors::error::VcxResult, global::settings, utils};

pub async fn libindy_prover_store_credential(
    wallet_handle: WalletHandle,
    cred_id: Option<&str>,
    cred_req_meta: &str,
    cred_json: &str,
    cred_def_json: &str,
    rev_reg_def_json: Option<&str>,
) -> VcxResult<String> {
    trace!(
        "libindy_prover_store_credential >>> cred_id: {:?}, cred_req_meta: {}, cred_json: {}, cred_def_json: {}, \
         rev_reg_def_json: {:?}",
        cred_id,
        cred_req_meta,
        cred_json,
        cred_def_json,
        rev_reg_def_json,
    );

    if settings::indy_mocks_enabled() {
        return Ok("cred_id".to_string());
    }

    let cred_req_meta = serde_json::from_str::<CredentialRequestMetadata>(cred_req_meta)?;

    let cred_json = serde_json::from_str::<Credential>(cred_json)?;

    let cred_def_json = serde_json::from_str::<CredentialDefinition>(cred_def_json)?;

    let rev_reg_def_json = match rev_reg_def_json {
        None => None,
        Some(s) => Some(serde_json::from_str::<RevocationRegistryDefinition>(s)?),
    };

    let res = Locator::instance()
        .prover_controller
        .store_credential(
            wallet_handle,
            cred_id.map(ToOwned::to_owned),
            cred_req_meta,
            cred_json,
            cred_def_json,
            rev_reg_def_json,
        )
        .await?;

    Ok(res)
}

pub async fn libindy_prover_get_credential(wallet_handle: WalletHandle, cred_id: &str) -> VcxResult<String> {
    trace!("libindy_prover_get_credential >>> cred_id: {:?}", cred_id,);

    let res = Locator::instance()
        .prover_controller
        .get_credential(wallet_handle, cred_id.into())
        .await?;

    Ok(res)
}

pub async fn libindy_prover_delete_credential(wallet_handle: WalletHandle, cred_id: &str) -> VcxResult<()> {
    Locator::instance()
        .prover_controller
        .delete_credential(wallet_handle, cred_id.into())
        .await?;

    Ok(())
}

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

pub async fn libindy_prover_create_credential_req(
    wallet_handle: WalletHandle,
    prover_did: &str,
    credential_offer_json: &str,
    credential_def_json: &str,
    master_secret_name: &str,
) -> VcxResult<(String, String)> {
    if settings::indy_mocks_enabled() {
        return Ok((utils::constants::CREDENTIAL_REQ_STRING.to_owned(), String::new()));
    }

    let cred_offer = serde_json::from_str::<CredentialOffer>(credential_offer_json)?;

    let cred_def = serde_json::from_str::<CredentialDefinition>(credential_def_json)?;

    let res = Locator::instance()
        .prover_controller
        .create_credential_request(
            wallet_handle,
            DidValue(prover_did.into()),
            cred_offer,
            cred_def,
            master_secret_name.into(),
        )
        .await?;

    Ok(res)
}
