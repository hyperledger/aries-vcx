use indy::did;

use crate::{settings, utils};
use crate::error::prelude::*;
use crate::libindy::utils::wallet::get_wallet_handle;

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
