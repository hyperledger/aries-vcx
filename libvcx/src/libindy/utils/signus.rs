use indy::future::Future;
use indy::did;

use crate::{settings, utils};
use crate::error::prelude::*;
use crate::libindy::utils::wallet::get_wallet_handle;

pub fn create_and_store_my_did(seed: Option<&str>, method_name: Option<&str>) -> VcxResult<(String, String)> {
    trace!("create_and_store_my_did >>> seed: {:?}, method_name: {:?}", seed, method_name);
    if settings::indy_mocks_enabled() {
        return Ok((utils::constants::DID.to_string(), utils::constants::VERKEY.to_string()));
    }

    let my_did_json = json!({"seed": seed, "method_name": method_name});

    let wallet_handle = get_wallet_handle();
    trace!("create_and_store_my_did >>> seed: {:?}, method_name: {:?} wallet_handle={:?}", seed, method_name, wallet_handle);
    let res = did::create_and_store_my_did(wallet_handle, &my_did_json.to_string())
        .wait()
        .map_err(VcxError::from);
    warn!("create_and_store_my_did >>> created res={:?}", res);
    res
}
