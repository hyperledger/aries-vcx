use indy::did;
use indy::future::TryFutureExt;
use indy_sys::WalletHandle;
use serde_json::Value;

use crate::utils;
use crate::error::prelude::*;
use crate::global::settings;
use crate::global::wallet::get_main_wallet_handle;
use crate::libindy::utils::ledger;
use crate::libindy::utils::mocks::did_mocks::{did_mocks_enabled, DidMocks};

pub async fn main_wallet_create_and_store_my_did(seed: Option<&str>, method_name: Option<&str>) -> VcxResult<(String, String)> {
    create_and_store_my_did(get_main_wallet_handle(), seed, method_name).await
}

pub async fn create_and_store_my_did(wallet_handle: WalletHandle, seed: Option<&str>, method_name: Option<&str>) -> VcxResult<(String, String)> {
    trace!("create_and_store_my_did >>> seed: {:?}, method_name: {:?}", seed, method_name);
    let my_did_json = json!({"seed": seed, "method_name": method_name});
    if settings::indy_mocks_enabled() {
        return Ok((utils::constants::DID.to_string(), utils::constants::VERKEY.to_string()));
    }

    let res = did::create_and_store_my_did(wallet_handle, &my_did_json.to_string())
        .await
        .map_err(VcxError::from);
    res
}

pub async fn libindy_replace_keys_start(did: &str) -> VcxResult<String> {
    if DidMocks::has_did_mock_responses() {
        warn!("libindy_replace_keys_start >> retrieving did mock response");
        Ok(DidMocks::get_next_did_response())
    } else {
        did::replace_keys_start(get_main_wallet_handle(), did, "{}")
            .map_err(VcxError::from)
            .await
    }
}

pub async fn rotate_verkey_apply(did: &str, temp_vk: &str) -> VcxResult<()> {
    let nym_request = ledger::libindy_build_nym_request(&did, &did, Some(&temp_vk), None, None).await?;
    let nym_request = ledger::append_txn_author_agreement_to_request(&nym_request).await?;
    let nym_result = ledger::libindy_sign_and_submit_request(&did, &nym_request).await?;
    let nym_result_json: Value = serde_json::from_str(&nym_result)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot deserialize {:?} into Value, err: {:?}", nym_result, err)))?;
    let response_type: String = nym_result_json["op"].as_str()
        .ok_or(VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot failed to convert {:?} into str", nym_result_json["op"])))?.to_string();
    if response_type != "REPLY" {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse, format!("Obained non-success ledger response: {}", nym_result_json)));
    }
    libindy_replace_keys_apply(&did).await
}

pub async fn rotate_verkey(did: &str) -> VcxResult<()> {
    let trustee_temp_verkey = libindy_replace_keys_start(did).await?;
    rotate_verkey_apply(did, &trustee_temp_verkey).await
}

pub async fn libindy_replace_keys_apply(did: &str) -> VcxResult<()> {
    if did_mocks_enabled() {
        warn!("libindy_replace_keys_apply >> retrieving did mock response");
        Ok(())
    } else {
        did::replace_keys_apply(get_main_wallet_handle(), did)
            .map_err(VcxError::from)
            .await
    }
}

pub async fn get_verkey_from_wallet(did: &str) -> VcxResult<String> {
    if DidMocks::has_did_mock_responses() {
        warn!("get_verkey_from_wallet >> retrieving did mock response");
        Ok(DidMocks::get_next_did_response())
    } else {
        did::key_for_local_did(get_main_wallet_handle(), did)
            .map_err(VcxError::from)
            .await
    }
}

pub async fn get_verkey_from_ledger(did: &str) -> VcxResult<String> {
    let nym_response: String = ledger::get_nym(did).await?;
    let nym_json: Value = serde_json::from_str(&nym_response)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot deserialize {:?} into Value, err: {:?}", nym_response, err)))?;
    let nym_data: String = nym_json["result"]["data"].as_str()
        .ok_or(VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot deserialize {:?} into String", nym_json["result"]["data"])))?.to_string();
    let nym_data: Value = serde_json::from_str(&nym_data)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot deserialize {:?} into Value, err: {:?}", nym_data, err)))?;
    Ok(nym_data["verkey"].as_str()
        .ok_or(VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot deserialize {:?} into String", nym_data["verkey"])))?
        .to_string())
}


#[cfg(test)]
mod test {
    use std::thread;
    use std::time::Duration;
    use super::*;

    use crate::utils::devsetup::*;
    use crate::utils::mockdata::mockdata_pool;
    use crate::libindy::utils::mocks::pool_mocks::PoolMocks;

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_rotate_verkey() {
        let _setup = SetupWithWalletAndAgency::init().await;
        let (did, verkey) = ledger::add_new_did(None).await;
        rotate_verkey(&did).await.unwrap();
        thread::sleep(Duration::from_millis(100));
        let local_verkey = get_verkey_from_wallet(&did).await.unwrap();
        let ledger_verkey = get_verkey_from_ledger(&did).await.unwrap();
        assert_ne!(verkey, ledger_verkey);
        assert_eq!(local_verkey, ledger_verkey);
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_rotate_verkey_fails() {
        let _setup = SetupPoolMocks::init().await;
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        PoolMocks::set_next_pool_response(mockdata_pool::RESPONSE_REQNACK);
        PoolMocks::set_next_pool_response(mockdata_pool::NYM_REQUEST_VALID);
        let local_verkey_1 = get_verkey_from_wallet(&did).await.unwrap();
        assert_eq!(rotate_verkey(&did).await.unwrap_err().kind(), VcxErrorKind::InvalidLedgerResponse);
        let local_verkey_2 = get_verkey_from_wallet(&did).await.unwrap();
        assert_eq!(local_verkey_1, local_verkey_2);
    }
}
