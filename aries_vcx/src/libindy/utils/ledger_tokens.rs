use std::collections::HashMap;
use std::fmt;

use indy::future::Future;
use serde_json::Value;

use crate::{libindy, settings, utils};
use crate::error::prelude::*;
use crate::libindy::utils::ledger::{append_txn_author_agreement_to_request, auth_rule, libindy_sign_and_submit_request, libindy_sign_request, libindy_submit_request};
use crate::libindy::utils::wallet::get_wallet_handle;
use crate::utils::constants::{CREATE_TRANSFER_ACTION, SUBMIT_SCHEMA_RESPONSE};

static DEFAULT_FEES: &str = r#"{"0":0, "1":0, "3":0, "100":0, "101":2, "102":42, "103":0, "104":0, "105":0, "107":0, "108":0, "109":0, "110":0, "111":0, "112":0, "113":2, "114":2, "115":0, "116":0, "117":0, "118":0, "119":0, "10001":0}"#;

pub fn publish_txn_on_ledger(req: &str, txn_action: (&str, &str, &str, Option<&str>, Option<&str>)) -> VcxResult<(String)> {
    debug!("publish_txn_on_ledger(req: {}, txn_action: {:?})", req, txn_action);
    if settings::indy_mocks_enabled() {
        return Ok(SUBMIT_SCHEMA_RESPONSE.to_string());
    }
    let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;
    let txn_response = libindy_sign_and_submit_request(&did, req)?;
    Ok((txn_response))
}

// This is used for testing purposes only!!!
pub fn mint_tokens_and_set_fees(number_of_addresses: Option<u32>, tokens_per_address: Option<u64>, fees: Option<String>, seed: Option<String>) -> VcxResult<()> {
    trace!("mint_tokens_and_set_fees >>> number_of_addresses: {:?}, tokens_per_address: {:?}, fees: {:?}, seed: {:?}",
           number_of_addresses, tokens_per_address, fees, seed);

    Ok(())
}

pub fn add_new_did(role: Option<&str>) -> (String, String) {
    use indy::ledger;

    let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

    let (did, verkey) = libindy::utils::signus::create_and_store_my_did(None, None).unwrap();
    let mut req_nym = ledger::build_nym_request(&institution_did, &did, Some(&verkey), None, role).wait().unwrap();

    req_nym = append_txn_author_agreement_to_request(&req_nym).unwrap();

    libindy::utils::ledger::libindy_sign_and_submit_request(&institution_did, &req_nym).unwrap();
    (did, verkey)
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use super::*;

    static ZERO_FEES: &str = r#"{"0":0, "1":0, "101":0, "10001":0, "102":0, "103":0, "104":0, "105":0, "107":0, "108":0, "109":0, "110":0, "111":0, "112":0, "113":0, "114":0, "115":0, "116":0, "117":0, "118":0, "119":0}"#;

    pub fn token_setup(number_of_addresses: Option<u32>, tokens_per_address: Option<u64>, use_zero_fees: bool) {
        let fees = if use_zero_fees { ZERO_FEES } else { DEFAULT_FEES };
        mint_tokens_and_set_fees(number_of_addresses, tokens_per_address, Some(fees.to_string()), None).unwrap();
    }
}