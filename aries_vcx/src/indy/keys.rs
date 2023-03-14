use vdrtools::{DidMethod, DidValue, KeyInfo, Locator, MyDidInfo, WalletHandle};

use crate::{
    errors::error::prelude::*,
    global::settings,
    indy::utils::mocks::did_mocks::{did_mocks_enabled, DidMocks},
    utils,
};

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

    if settings::indy_mocks_enabled() {
        return Ok((utils::constants::DID.to_string(), utils::constants::VERKEY.to_string()));
    }

    let res = Locator::instance()
        .did_controller
        .create_and_store_my_did(
            wallet_handle,
            MyDidInfo {
                method_name: method_name.map(|m| DidMethod(m.into())),
                seed: seed.map(ToOwned::to_owned),
                ..MyDidInfo::default()
            },
        )
        .await?;

    Ok(res)
}

pub async fn libindy_replace_keys_start(wallet_handle: WalletHandle, did: &str) -> VcxResult<String> {
    if DidMocks::has_did_mock_responses() {
        warn!("libindy_replace_keys_start >> retrieving did mock response");
        return Ok(DidMocks::get_next_did_response());
    }

    let res = Locator::instance()
        .did_controller
        .replace_keys_start(wallet_handle, KeyInfo::default(), DidValue(did.into()))
        .await?;

    Ok(res)
}

pub async fn libindy_replace_keys_apply(wallet_handle: WalletHandle, did: &str) -> VcxResult<()> {
    if did_mocks_enabled() {
        warn!("libindy_replace_keys_apply >> retrieving did mock response");
        return Ok(());
    }

    Locator::instance()
        .did_controller
        .replace_keys_apply(wallet_handle, DidValue(did.into()))
        .await?;

    Ok(())
}

pub async fn get_verkey_from_wallet(wallet_handle: WalletHandle, did: &str) -> VcxResult<String> {
    if DidMocks::has_did_mock_responses() {
        warn!("get_verkey_from_wallet >> retrieving did mock response");
        return Ok(DidMocks::get_next_did_response());
    }

    let res = Locator::instance()
        .did_controller
        .key_for_local_did(wallet_handle, DidValue(did.into()))
        .await?;

    Ok(res)
}
