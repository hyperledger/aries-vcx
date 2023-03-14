use vdrtools::{DidValue, Locator, PoolHandle, WalletHandle};

use crate::{
    common::primitives::revocation_registry::RevocationRegistryDefinition,
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    global::settings,
    indy::{
        anoncreds,
        ledger::transactions::{
            build_rev_reg_delta_request, build_rev_reg_request, check_response, sign_and_submit_to_ledger,
        },
        wallet_non_secrets::{clear_rev_reg_delta, get_rev_reg_delta, set_rev_reg_delta},
    },
    utils::parse_and_validate,
};

pub const BLOB_STORAGE_TYPE: &str = "default";

// consider relocating out of primitive
pub async fn libindy_create_and_store_revoc_reg(
    wallet_handle: WalletHandle,
    issuer_did: &str,
    cred_def_id: &str,
    tails_dir: &str,
    max_creds: u32,
    tag: &str,
) -> VcxResult<(String, String, String)> {
    trace!("creating revocation: {}, {}, {}", cred_def_id, tails_dir, max_creds);

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

// consider relocating out of primitive
pub async fn libindy_issuer_revoke_credential(
    wallet_handle: WalletHandle,
    tails_file: &str,
    rev_reg_id: &str,
    cred_rev_id: &str,
) -> VcxResult<String> {
    let blob_handle = anoncreds::blob_storage_open_reader(tails_file).await?;

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

// consider relocating out of primitive
pub async fn libindy_issuer_merge_revocation_registry_deltas(old_delta: &str, new_delta: &str) -> VcxResult<String> {
    let res = Locator::instance()
        .issuer_controller
        .merge_revocation_registry_deltas(parse_and_validate(old_delta)?, parse_and_validate(new_delta)?)?;

    Ok(res)
}

pub async fn publish_rev_reg_def(
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    issuer_did: &str,
    rev_reg_def: &RevocationRegistryDefinition,
) -> VcxResult<()> {
    trace!("publish_rev_reg_def >>> issuer_did: {}, rev_reg_def: ...", issuer_did);
    if settings::indy_mocks_enabled() {
        debug!("publish_rev_reg_def >>> mocked success");
        return Ok(());
    }

    let rev_reg_def_json = serde_json::to_string(&rev_reg_def).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!("Failed to serialize rev_reg_def: {:?}, error: {:?}", rev_reg_def, err),
        )
    })?;

    let rev_reg_def_req = build_rev_reg_request(issuer_did, &rev_reg_def_json).await?;

    let response = sign_and_submit_to_ledger(wallet_handle, pool_handle, issuer_did, &rev_reg_def_req).await?;

    check_response(&response)
}

pub async fn publish_rev_reg_delta(
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    issuer_did: &str,
    rev_reg_id: &str,
    revoc_reg_delta_json: &str,
) -> VcxResult<String> {
    trace!(
        "publish_rev_reg_delta >>> issuer_did: {}, rev_reg_id: {}, revoc_reg_delta_json: {}",
        issuer_did,
        rev_reg_id,
        revoc_reg_delta_json
    );

    let request = build_rev_reg_delta_request(issuer_did, rev_reg_id, revoc_reg_delta_json).await?;

    let response = sign_and_submit_to_ledger(wallet_handle, pool_handle, issuer_did, &request).await?;

    check_response(&response)?;

    Ok(response)
}

// consider moving out of indy dir as this aggregates multiple calls
pub async fn revoke_credential_local(
    wallet_handle: WalletHandle,
    tails_file: &str,
    rev_reg_id: &str,
    cred_rev_id: &str,
) -> VcxResult<()> {
    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    let mut new_delta_json =
        libindy_issuer_revoke_credential(wallet_handle, tails_file, rev_reg_id, cred_rev_id).await?;

    debug!("revoke_credential_local >>> new_delta_json: {}", new_delta_json);

    if let Some(old_delta_json) = get_rev_reg_delta(wallet_handle, rev_reg_id).await {
        debug!("revoke_credential_local >>> old_delta_json: {}", old_delta_json);
        new_delta_json =
            libindy_issuer_merge_revocation_registry_deltas(old_delta_json.as_str(), new_delta_json.as_str()).await?;
        debug!("revoke_credential_local >>> merged_delta_json: {}", new_delta_json);
    }

    set_rev_reg_delta(wallet_handle, rev_reg_id, &new_delta_json).await
}

// consider moving out of indy dir as this aggregates multiple calls
pub async fn publish_local_revocations(
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    submitter_did: &str,
    rev_reg_id: &str,
) -> VcxResult<()> {
    if let Some(delta) = get_rev_reg_delta(wallet_handle, rev_reg_id).await {
        publish_rev_reg_delta(wallet_handle, pool_handle, submitter_did, rev_reg_id, &delta).await?;

        info!(
            "publish_local_revocations >>> rev_reg_delta published for rev_reg_id {}",
            rev_reg_id
        );

        match clear_rev_reg_delta(wallet_handle, rev_reg_id).await {
            Ok(_) => {
                info!(
                    "publish_local_revocations >>> rev_reg_delta storage cleared for rev_reg_id {}",
                    rev_reg_id
                );
                Ok(())
            }
            Err(err) => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::RevDeltaFailedToClear,
                format!(
                    "Failed to clear revocation delta storage for rev_reg_id: {}, error: {}",
                    rev_reg_id, err
                ),
            )),
        }
    } else {
        Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::RevDeltaNotFound,
            format!(
                "Failed to publish revocation delta for revocation registry {}, no delta found. Possibly already \
                 published?",
                rev_reg_id
            ),
        ))
    }
}
