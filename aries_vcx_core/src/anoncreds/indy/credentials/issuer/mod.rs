use vdrtools::{
    CredentialOffer, CredentialRequest, CredentialValues, DidValue, Locator, RevocationRegistryId,
};

use crate::{
    anoncreds::indy::{general, general::blob_storage_open_reader},
    errors::error::VcxCoreResult,
    global::settings,
    indy::utils::parse_and_validate,
    utils,
    utils::constants::LIBINDY_CRED_OFFER,
    wallet::indy::wallet_non_secrets::{get_rev_reg_delta, set_rev_reg_delta},
    WalletHandle,
};

pub async fn libindy_issuer_create_credential_offer(
    wallet_handle: WalletHandle,
    cred_def_id: &str,
) -> VcxCoreResult<String> {
    if settings::indy_mocks_enabled() {
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
) -> VcxCoreResult<(String, Option<String>, Option<String>)> {
    if settings::indy_mocks_enabled() {
        return Ok((utils::constants::CREDENTIAL_JSON.to_owned(), None, None));
    }

    let blob_handle = match tails_file {
        Some(x) => Some(blob_storage_open_reader(&x).await?),
        None => None,
    };

    let res = Locator::instance()
        .issuer_controller
        .new_credential(
            wallet_handle,
            parse_and_validate::<CredentialOffer>(cred_offer_json)?,
            parse_and_validate::<CredentialRequest>(cred_req_json)?,
            parse_and_validate::<CredentialValues>(cred_values_json)?,
            rev_reg_id.map(RevocationRegistryId),
            blob_handle,
        )
        .await?;

    Ok(res)
}

pub const BLOB_STORAGE_TYPE: &str = "default";

pub async fn libindy_create_and_store_revoc_reg(
    wallet_handle: WalletHandle,
    issuer_did: &str,
    cred_def_id: &str,
    tails_dir: &str,
    max_creds: u32,
    tag: &str,
) -> VcxCoreResult<(String, String, String)> {
    trace!(
        "creating revocation: {}, {}, {}",
        cred_def_id,
        tails_dir,
        max_creds
    );

    let tails_config = json!({"base_dir": tails_dir,"uri_pattern": ""}).to_string();

    let writer = Locator::instance()
        .blob_storage_controller
        .open_writer(BLOB_STORAGE_TYPE.into(), tails_config)
        .await?;

    let res = Locator::instance()
        .issuer_controller
        .create_and_store_revocation_registry(
            wallet_handle,
            DidValue(issuer_did.into()),
            None,
            tag.into(),
            vdrtools::CredentialDefinitionId(cred_def_id.into()),
            vdrtools::RevocationRegistryConfig {
                issuance_type: Some(vdrtools::IssuanceType::ISSUANCE_BY_DEFAULT),
                max_cred_num: Some(max_creds),
            },
            writer,
        )
        .await?;

    Ok(res)
}

pub async fn libindy_issuer_revoke_credential(
    wallet_handle: WalletHandle,
    tails_file: &str,
    rev_reg_id: &str,
    cred_rev_id: &str,
) -> VcxCoreResult<String> {
    let blob_handle = general::blob_storage_open_reader(tails_file).await?;

    let res = Locator::instance()
        .issuer_controller
        .revoke_credential(
            wallet_handle,
            blob_handle,
            vdrtools::RevocationRegistryId(rev_reg_id.into()),
            cred_rev_id.into(),
        )
        .await?;

    Ok(res)
}

pub async fn libindy_issuer_merge_revocation_registry_deltas(
    old_delta: &str,
    new_delta: &str,
) -> VcxCoreResult<String> {
    let res = Locator::instance()
        .issuer_controller
        .merge_revocation_registry_deltas(
            parse_and_validate(old_delta)?,
            parse_and_validate(new_delta)?,
        )?;

    Ok(res)
}

pub async fn revoke_credential_local(
    wallet_handle: WalletHandle,
    tails_file: &str,
    rev_reg_id: &str,
    cred_rev_id: &str,
) -> VcxCoreResult<()> {
    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    let mut new_delta_json =
        libindy_issuer_revoke_credential(wallet_handle, tails_file, rev_reg_id, cred_rev_id)
            .await?;

    debug!(
        "revoke_credential_local >>> new_delta_json: {}",
        new_delta_json
    );

    if let Some(old_delta_json) = get_rev_reg_delta(wallet_handle, rev_reg_id).await {
        debug!(
            "revoke_credential_local >>> old_delta_json: {}",
            old_delta_json
        );
        new_delta_json = libindy_issuer_merge_revocation_registry_deltas(
            old_delta_json.as_str(),
            new_delta_json.as_str(),
        )
        .await?;
        debug!(
            "revoke_credential_local >>> merged_delta_json: {}",
            new_delta_json
        );
    }

    set_rev_reg_delta(wallet_handle, rev_reg_id, &new_delta_json).await
}
