use vdrtools::WalletHandle;

use vdrtools::{DidMethod, DidValue, KeyInfo, Locator, MyDidInfo};

use crate::errors::error::prelude::*;
use crate::global::settings;
use crate::indy::utils::mocks::did_mocks::{did_mocks_enabled, DidMocks};
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
