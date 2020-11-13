use futures::Future;
use indy::did;

use crate::utils::error::prelude::*;
use crate::utils::wallet::get_wallet_handle;

pub fn create_and_store_my_did(seed: Option<&str>, method_name: Option<&str>) -> AgencyClientResult<(String, String)> {
    let my_did_json = json!({"seed": seed, "method_name": method_name});

    did::create_and_store_my_did(get_wallet_handle(), &my_did_json.to_string())
        .wait()
        .map_err(AgencyClientError::from)
}
