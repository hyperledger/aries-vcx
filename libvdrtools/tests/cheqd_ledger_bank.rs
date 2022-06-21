#![cfg(feature = "cheqd")]
#![cfg(feature = "local_nodes_cheqd_pool")]

#![cfg_attr(feature = "fatal_warnings", deny(warnings))]

#[macro_use]
extern crate derivative;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate log;

#[macro_use]
mod utils;

use utils::{cheqd_ledger, cheqd_pool, cheqd_keys, cheqd_setup};
use serde_json::Value;

#[cfg(feature = "cheqd")]
mod high_cases {
    #[cfg(feature = "local_nodes_cheqd_pool")]
    use super::*;

    #[cfg(test)]
    mod query_balance {
        use super::*;

        #[test]
        #[cfg(feature = "cheqd")]
        fn test_query_balance() {
            let setup = cheqd_setup::CheqdSetup::new();
            let amount_for_transfer = "100";

            ///// Query get current balance

            let query = cheqd_ledger::bank::bank_build_query_balance(&setup.account_id, &setup.denom).unwrap();
            let query_resp = cheqd_pool::abci_query(&setup.pool_alias, &query).unwrap();
            let query_resp = cheqd_ledger::bank::parse_query_balance_resp(&query_resp).unwrap();
            println!("Query response: {:?}", query_resp);

            let balance_response: Value = serde_json::from_str(&query_resp).unwrap();

            let current_balance = balance_response.as_object().unwrap()
                .get("balance").unwrap().as_object().unwrap()
                .get("amount").unwrap().as_str().unwrap();
            println!("current_balance: {:?}", current_balance);

            ///// Create second account

            let second_alias = "second_alias";
            let second_account_response = cheqd_keys::add_random(setup.wallet_handle, second_alias).unwrap();
            let second_account_response: Value = serde_json::from_str(&second_account_response).unwrap();
            let second_account = second_account_response.as_object().unwrap()
                .get("account_id").unwrap().as_str().unwrap();
            println!("Second account response: {:?}", second_account_response);

            // Msg send amount
            let msg = cheqd_ledger::bank::build_msg_send(
                &setup.account_id,
                second_account,
                amount_for_transfer,
                &setup.denom,
            ).unwrap();

            // Build, sign, broadcast tx
            let resp = setup.build_and_sign_and_broadcast_tx(&msg).unwrap();

            // Parse
            let tx_resp_parsed = cheqd_ledger::bank::parse_msg_send_resp(&resp).unwrap();
            let tx_resp_parsed: Value = serde_json::from_str(&tx_resp_parsed).unwrap();
            println!("Tx resp: {:?}", tx_resp_parsed);

            ///// Query get balance after send

            let query = cheqd_ledger::bank::bank_build_query_balance(&setup.account_id, &setup.denom).unwrap();
            let query_resp = cheqd_pool::abci_query(&setup.pool_alias, &query).unwrap();
            let query_resp = cheqd_ledger::bank::parse_query_balance_resp(&query_resp).unwrap();
            println!("Query response: {:?}", query_resp);

            let new_balance_response: Value = serde_json::from_str(&query_resp).unwrap();

            let new_balance = new_balance_response.as_object().unwrap()
                .get("balance").unwrap().as_object().unwrap()
                .get("amount").unwrap().as_str().unwrap();
            println!("new_balance: {:?}", new_balance);

            let expected_result: Value = json!({
                "balance": {
                    "denom": setup.denom,
                    "amount": (current_balance.parse::<u64>().unwrap() - amount_for_transfer.parse::<u64>().unwrap() - cheqd_setup::MAX_COIN_AMOUNT).to_string()
                }
            });

            assert_eq!(expected_result, new_balance_response);
        }
    }
}
