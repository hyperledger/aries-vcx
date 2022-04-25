use indy::did;
use indy::future::TryFutureExt;
use serde_json::Value;

use crate::{settings, utils};
use crate::error::prelude::*;
use crate::libindy::utils::wallet::get_wallet_handle;
use crate::libindy::utils::ledger;

pub async fn create_and_store_my_did(seed: Option<&str>, method_name: Option<&str>) -> VcxResult<(String, String)> {
    trace!("create_and_store_my_did >>> seed: {:?}, method_name: {:?}", seed, method_name);
    if settings::indy_mocks_enabled() {
        return Ok((utils::constants::DID.to_string(), utils::constants::VERKEY.to_string()));
    }

    let my_did_json = json!({"seed": seed, "method_name": method_name});

    let res = did::create_and_store_my_did(get_wallet_handle(), &my_did_json.to_string())
        .await
        .map_err(VcxError::from);
    res
}

pub async fn rotate_verkey(did: &str) -> VcxResult<()> {
    let trustee_temp_verkey = libindy_replace_keys_start(did).await?;
    let nym_request = ledger::libindy_build_nym_request(&did, &did, Some(&trustee_temp_verkey), None, None).await?;
    let nym_result = ledger::libindy_sign_and_submit_request(&did, &nym_request).await?; // TODO: Verify success
    let nym_result_json: Value = serde_json::from_str(&nym_result)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot deserialize {:?} into Value, err: {:?}", nym_result, err)))?;
    let response_type: String = nym_result_json["op"].as_str()
        .ok_or(VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot failed to convert {:?} into str", nym_result_json["op"])))?.to_string();
    if response_type != "REPLY" {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse, format!("Obained non-success ledger response: {}", nym_result_json)));
    }
    libindy_replace_keys_apply(&did).await
}

pub async fn libindy_replace_keys_start(did: &str) -> VcxResult<String> {
    did::replace_keys_start(get_wallet_handle(), did, "{}")
        .map_err(VcxError::from)
        .await
}

pub async fn libindy_replace_keys_apply(did: &str) -> VcxResult<()> {
    did::replace_keys_apply(get_wallet_handle(), did)
        .map_err(VcxError::from)
        .await
}

pub async fn key_for_local_did(did: &str) -> VcxResult<String> {
    did::key_for_local_did(get_wallet_handle(), did)
        .map_err(VcxError::from)
        .await
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::utils::devsetup::SetupWithWalletAndAgency;

    async fn get_verkey_from_ledger(did: &str) -> String {
        let nym_response: String = ledger::get_nym(did).await.unwrap();
        let nym_json: Value = serde_json::from_str(&nym_response).unwrap();
        let nym_data: Value = serde_json::from_str(nym_json["result"]["data"].as_str().unwrap()).unwrap();
        nym_data["verkey"].as_str().unwrap().to_string()
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_rotate_verkey() {
        let _setup = SetupWithWalletAndAgency::init().await;
        let (did, verkey) = ledger::add_new_did(None).await;
        rotate_verkey(&did).await.unwrap();
        let local_verkey = key_for_local_did(&did).await.unwrap();
        let ledger_verkey = get_verkey_from_ledger(&did).await;
        assert_ne!(verkey, ledger_verkey);
        assert_eq!(local_verkey, ledger_verkey);
    }
}
