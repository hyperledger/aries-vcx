use indy::did;
use indy::future::TryFutureExt;
use indy_sys::{WalletHandle, PoolHandle};
use serde_json::Value;

use crate::error::prelude::*;
use crate::global::settings;
use crate::libindy::utils::ledger;
use crate::libindy::utils::mocks::did_mocks::{did_mocks_enabled, DidMocks};
use crate::utils;

pub async fn create_and_store_my_did(
    wallet_handle: WalletHandle,
    seed: Option<&str>,
    method_name: Option<&str>,
) -> VcxResult<(String, String)> {
    trace!(
        "create_and_store_my_did >>> seed: {:?}, method_name: {:?}",
        seed,
        method_name
    );
    let my_did_json = json!({"seed": seed, "method_name": method_name});
    if settings::indy_mocks_enabled() {
        return Ok((utils::constants::DID.to_string(), utils::constants::VERKEY.to_string()));
    }

    let res = did::create_and_store_my_did(wallet_handle, &my_did_json.to_string())
        .await
        .map_err(VcxError::from);
    res
}

pub async fn libindy_replace_keys_start(wallet_handle: WalletHandle, did: &str) -> VcxResult<String> {
    if DidMocks::has_did_mock_responses() {
        warn!("libindy_replace_keys_start >> retrieving did mock response");
        Ok(DidMocks::get_next_did_response())
    } else {
        did::replace_keys_start(wallet_handle, did, "{}")
            .map_err(VcxError::from)
            .await
    }
}

pub async fn rotate_verkey_apply(wallet_handle: WalletHandle, pool_handle: PoolHandle, did: &str, temp_vk: &str) -> VcxResult<()> {
    let nym_request = ledger::libindy_build_nym_request(did, did, Some(temp_vk), None, None).await?;
    let nym_request = ledger::append_txn_author_agreement_to_request(&nym_request).await?;
    let nym_result = ledger::libindy_sign_and_submit_request(wallet_handle, pool_handle, did, &nym_request).await?;
    let nym_result_json: Value = serde_json::from_str(&nym_result).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::SerializationError,
            format!("Cannot deserialize {:?} into Value, err: {:?}", nym_result, err),
        )
    })?;
    let response_type: String = nym_result_json["op"]
        .as_str()
        .ok_or(VcxError::from_msg(
            VcxErrorKind::SerializationError,
            format!("Cannot failed to convert {:?} into str", nym_result_json["op"]),
        ))?
        .to_string();
    if response_type != "REPLY" {
        return Err(VcxError::from_msg(
            VcxErrorKind::InvalidLedgerResponse,
            format!("Obained non-success ledger response: {}", nym_result_json),
        ));
    }
    libindy_replace_keys_apply(wallet_handle, did).await
}

pub async fn rotate_verkey(wallet_handle: WalletHandle, pool_handle: PoolHandle, did: &str) -> VcxResult<()> {
    let trustee_temp_verkey = libindy_replace_keys_start(wallet_handle, did).await?;
    rotate_verkey_apply(wallet_handle, pool_handle, did, &trustee_temp_verkey).await
}

pub async fn libindy_replace_keys_apply(wallet_handle: WalletHandle, did: &str) -> VcxResult<()> {
    if did_mocks_enabled() {
        warn!("libindy_replace_keys_apply >> retrieving did mock response");
        Ok(())
    } else {
        did::replace_keys_apply(wallet_handle, did)
            .map_err(VcxError::from)
            .await
    }
}

pub async fn get_verkey_from_wallet(wallet_handle: WalletHandle, did: &str) -> VcxResult<String> {
    if DidMocks::has_did_mock_responses() {
        warn!("get_verkey_from_wallet >> retrieving did mock response");
        Ok(DidMocks::get_next_did_response())
    } else {
        did::key_for_local_did(wallet_handle, did).map_err(VcxError::from).await
    }
}

pub async fn get_verkey_from_ledger(did: &str) -> VcxResult<String> {
    let nym_response: String = ledger::get_nym(did).await?;
    let nym_json: Value = serde_json::from_str(&nym_response).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::SerializationError,
            format!("Cannot deserialize {:?} into Value, err: {:?}", nym_response, err),
        )
    })?;
    let nym_data: String = nym_json["result"]["data"]
        .as_str()
        .ok_or(VcxError::from_msg(
            VcxErrorKind::SerializationError,
            format!("Cannot deserialize {:?} into String", nym_json["result"]["data"]),
        ))?
        .to_string();
    let nym_data: Value = serde_json::from_str(&nym_data).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::SerializationError,
            format!("Cannot deserialize {:?} into Value, err: {:?}", nym_data, err),
        )
    })?;
    Ok(nym_data["verkey"]
        .as_str()
        .ok_or(VcxError::from_msg(
            VcxErrorKind::SerializationError,
            format!("Cannot deserialize {:?} into String", nym_data["verkey"]),
        ))?
        .to_string())
}

#[cfg(test)]
mod test {
    use crate::libindy::utils::mocks::pool_mocks::{enable_pool_mocks, PoolMocks};
    use crate::utils::devsetup::*;
    use crate::utils::mockdata::mockdata_pool;

    use super::*;

    #[tokio::test]
    async fn test_rotate_verkey_fails() {
        let setup = SetupInstitutionWallet::init().await;
        enable_pool_mocks();

        PoolMocks::set_next_pool_response(mockdata_pool::RESPONSE_REQNACK);
        PoolMocks::set_next_pool_response(mockdata_pool::NYM_REQUEST_VALID);
        let local_verkey_1 = get_verkey_from_wallet(setup.wallet_handle, &setup.institution_did).await.unwrap();
        assert_eq!(
            rotate_verkey(setup.wallet_handle, 1, &setup.institution_did).await.unwrap_err().kind(),
            VcxErrorKind::InvalidLedgerResponse
        );
        let local_verkey_2 = get_verkey_from_wallet(setup.wallet_handle, &setup.institution_did).await.unwrap();
        assert_eq!(local_verkey_1, local_verkey_2);
    }
}
