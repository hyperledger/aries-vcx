#![cfg_attr(feature = "fatal_warnings", deny(warnings))]
#![cfg(feature = "local_nodes_cheqd_pool")]

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

#[cfg(feature = "cheqd")]
use utils::{cheqd_keys, cheqd_ledger, cheqd_pool, cheqd_setup};

#[cfg(feature = "cheqd")]
mod high_cases {
    use super::*;

    mod build_query_simulate {
        use super::*;
        use serde_json::Value;

        #[test]
        #[cfg(feature = "cheqd")]
        fn test_build_query_simulate() {
            let setup = cheqd_setup::CheqdSetup::new();
            let amount_for_transfer = "100";
            let (account_number, account_sequence) = setup
                .get_base_account_number_and_sequence(&setup.account_id)
                .unwrap();

            ///// Create second account

            let second_alias = "second_alias";
            let second_account_response =
                cheqd_keys::add_random(setup.wallet_handle, second_alias).unwrap();
            let second_account_response: Value =
                serde_json::from_str(&second_account_response).unwrap();
            let second_account = second_account_response
                .as_object()
                .unwrap()
                .get("account_id")
                .unwrap()
                .as_str()
                .unwrap();
            println!("Second account response: {:?}", second_account_response);

            // Msg send amount
            let msg = cheqd_ledger::bank::build_msg_send(
                &setup.account_id,
                second_account,
                amount_for_transfer,
                &setup.denom,
            )
            .unwrap();

            // Transaction
            let tx = cheqd_ledger::auth::build_tx(
                &setup.pool_alias,
                &setup.pub_key,
                &msg,
                account_number,
                account_sequence,
                90000,
                2250000u64,
                "ncheq",
                setup.get_timeout_height(),
                "memo",
            )
            .unwrap();

            // Sign
            let signed =
                cheqd_ledger::auth::sign_tx(setup.wallet_handle, &setup.key_alias, &tx).unwrap();

            let query = cheqd_ledger::tx::build_query_simulate(&signed).unwrap();
            let query_resp = cheqd_pool::abci_query(&setup.pool_alias, &query).unwrap();
            let resp = cheqd_ledger::tx::parse_query_simulate_resp(&query_resp).unwrap();

            let gas_simulate: Value = serde_json::from_str(&resp).unwrap();

            let gas_used = gas_simulate
                .as_object()
                .unwrap()
                .get("gas_info")
                .unwrap()
                .as_object()
                .unwrap()
                .get("gas_used")
                .unwrap()
                .as_u64()
                .unwrap();
            println!("gas_used: {:?}", gas_used);

            // Transaction
            let tx = cheqd_ledger::auth::build_tx(
                &setup.pool_alias,
                &setup.pub_key,
                &msg,
                account_number,
                account_sequence,
                gas_used,
                2250000u64,
                "ncheq",
                setup.get_timeout_height(),
                "memo",
            )
            .unwrap();

            // Sign
            let signed =
                cheqd_ledger::auth::sign_tx(setup.wallet_handle, &setup.key_alias, &tx).unwrap();

            // Broadcast
            let result = cheqd_pool::broadcast_tx_commit(&setup.pool_alias, &signed).unwrap();
            let result: Value = serde_json::from_str(&result).unwrap();

            let gas_wanted: u64 = result
                .as_object()
                .unwrap()
                .get("check_tx")
                .unwrap()
                .as_object()
                .unwrap()
                .get("gas_wanted")
                .unwrap()
                .as_str()
                .unwrap()
                .parse()
                .unwrap();

            println!("{:?}", gas_wanted);
            assert_eq!(gas_used, gas_wanted)
        }
    }
}
