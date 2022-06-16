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

#[cfg(feature = "cheqd")]
use utils::{cheqd_pool, cheqd_setup, cheqd_ledger, Setup};
#[cfg(feature = "cheqd")]
use utils::test;
#[cfg(feature = "cheqd")]
use serde_json::Value;

#[cfg(feature = "cheqd")]
mod high_cases {
    use super::*;

    #[cfg(test)]
    mod add {
        use super::*;

        #[test]
        fn test_add_persistent() {
            let setup = Setup::empty();
            let _result = cheqd_pool::add(&setup.name, "rpc_address", "chain_id", None).unwrap();
            // TODO VE-3079 check result
            assert!(test::check_cheqd_pool_exists(&setup.name));

            let result = cheqd_pool::add(&setup.name, "rpc_address", "chain_id", None);
            assert!(result.is_err());
        }

        #[test]
        fn test_add_in_memory() {
            let setup = Setup::empty();
            let _result = cheqd_pool::add(&setup.name, "rpc_address", "chain_id", Some("InMemory")).unwrap();
            // TODO VE-3079 check result
            assert!(!test::check_cheqd_pool_exists(&setup.name));

            // try to add InMemory pool with the same alias
            let result = cheqd_pool::add(&setup.name, "rpc_address", "chain_id", Some("InMemory"));
            assert!(result.is_err());

            // try to add Persistent pool with the same alias
            let result = cheqd_pool::add(&setup.name, "rpc_address", "chain_id", Some("Persistent"));
            assert!(result.is_err());

            test::cleanup_storage(&setup.name);
        }
    }

    #[cfg(test)]
    mod get_config {
        use super::*;

        #[test]
        fn test_get_config() {
            let setup = Setup::empty();
            cheqd_pool::add(&setup.name, "rpc_address", "chain_id", None).unwrap();
            let result = cheqd_pool::get_config(&setup.name).unwrap();
            test::cleanup_storage(&setup.name);


            println!("Data: {:?} ", result);
        }

        #[test]
        fn test_get_config_in_memory_pool() {
            let setup = Setup::empty();
            cheqd_pool::add(&setup.name, "rpc_address", "chain_id", Some("InMemory")).unwrap();
            let result = cheqd_pool::get_config(&setup.name).unwrap();
            println!("Data: {:?} ", result);
        }

        #[test]
        fn get_all_config() {
            let pool_name_1 = "test_pool_1";
            let pool_name_2 = "test_pool_2";
            const RPC_ADDRESS: &str = "rpc_address";
            const CHAIN_ID: &str = "chain_id";

            test::cleanup_storage(&pool_name_1);
            test::cleanup_storage(&pool_name_2);

            cheqd_pool::add(&pool_name_1, RPC_ADDRESS, CHAIN_ID, None).unwrap();
            cheqd_pool::add(&pool_name_2, RPC_ADDRESS, CHAIN_ID, None).unwrap();

            let result = cheqd_pool::get_all_config().unwrap();
            let result: Vec<Value> = serde_json::from_str(&result).unwrap();

            let expect_pool_1 = &json!({
                "alias": pool_name_1.to_string(),
                "rpc_address": RPC_ADDRESS.to_string(),
                "chain_id": CHAIN_ID.to_string()
            });
            let expect_pool_2 = &json!({
                "alias": pool_name_2.to_string(),
                "rpc_address": RPC_ADDRESS.to_string(),
                "chain_id": CHAIN_ID.to_string()
            });

            println!("Data: {:?} ", result);

            test::cleanup_storage(&pool_name_1);
            test::cleanup_storage(&pool_name_2);

            assert!(result.contains(expect_pool_1));
            assert!(result.contains(expect_pool_2));
        }

    }

    #[cfg(test)]
    mod broadcast_tx_commit {
        use super::*;
        use utils::did;

        #[test]
        #[cfg(feature = "cheqd")]
        fn test_broadcast_tx_commit() {
            let setup = cheqd_setup::CheqdSetup::new();

            let (account_number, account_sequence) = setup.get_base_account_number_and_sequence(&setup.account_id).unwrap();

            // Create DID
            let (did, verkey) = did::create_my_did(setup.wallet_handle, &cheqd_ledger::cheqd::did_info()).unwrap();

            // Send DID
            let msg = cheqd_ledger::cheqd::build_msg_create_did(&did, &verkey).unwrap();

            let signed_msg = cheqd_ledger::cheqd::sign_msg_request(setup.wallet_handle, &did, &msg).unwrap();

            // Transaction
            let tx = cheqd_ledger::auth::build_tx(
                &setup.pool_alias,
                &setup.pub_key,
                &signed_msg,
                account_number,
                account_sequence,
                90000,
                2250000u64,
                "ncheq",
                setup.get_timeout_height(),
                "memo",
            ).unwrap();

            // Sign
            let signed = cheqd_ledger::auth::sign_tx(setup.wallet_handle, &setup.key_alias, &tx).unwrap();

            // Broadcast
            cheqd_pool::broadcast_tx_commit(&setup.pool_alias, &signed).unwrap();
        }
    }

    #[cfg(test)]
    mod abci_query {
        use super::*;
        use utils::did;

        #[test]
        #[cfg(feature = "cheqd")]
        fn test_abci_query() {
            let setup = cheqd_setup::CheqdSetup::new();
            ///// Transaction sending

            let (account_number, account_sequence) = setup.get_base_account_number_and_sequence(&setup.account_id).unwrap();

            // Create DID
            let (did, verkey) = did::create_my_did(setup.wallet_handle, &cheqd_ledger::cheqd::did_info()).unwrap();

            // Send DID
            let msg = cheqd_ledger::cheqd::build_msg_create_did(&did, &verkey).unwrap();

            let signed_msg = cheqd_ledger::cheqd::sign_msg_request(setup.wallet_handle, &did, &msg).unwrap();

            // Transaction
            let tx = cheqd_ledger::auth::build_tx(
                &setup.pool_alias,
                &setup.pub_key,
                &signed_msg,
                account_number,
                account_sequence,
                90000,
                2250000u64,
                "ncheq",
                setup.get_timeout_height(),
                "memo",
            ).unwrap();

            // Signature
            let signed = cheqd_ledger::auth::sign_tx(setup.wallet_handle, &setup.key_alias, &tx).unwrap();

            // Broadcast
            let resp = cheqd_pool::broadcast_tx_commit(&setup.pool_alias, &signed).unwrap();

            // Parse the response
            let tx_resp_parsed = cheqd_ledger::cheqd::parse_msg_create_did_resp(&resp).unwrap();
            println!("Tx response: {:?}", tx_resp_parsed);
            let tx_resp: Value = serde_json::from_str(&tx_resp_parsed).unwrap();

            ///// Querying

            let query = cheqd_ledger::cheqd::build_query_get_did(tx_resp["id"].as_str().unwrap()).unwrap();

            let query_resp = cheqd_pool::abci_query(&setup.pool_alias, &query).unwrap();
            let query_resp_parsed = cheqd_ledger::cheqd::parse_query_get_did_resp(&query_resp).unwrap();
            println!("Query response: {:?}", query_resp_parsed);

            assert!(true);
        }
    }

    #[cfg(test)]
    mod abci_info {
        use super::*;
        use utils::environment;

        #[test]
        #[cfg(feature = "cheqd")]
        fn test_abci_info() {
            let setup = cheqd_setup::CheqdSetup::new();
            let query_resp = cheqd_pool::abci_info(&setup.pool_alias).unwrap();
            println!("Query response: {:?}", query_resp);

            assert!(true);
        }

        #[test]
        fn test_abci_info_in_memory_config() {
            let setup = Setup::empty();
            let cheqd_test_pool_ip = environment::cheqd_test_pool_ip();
            let cheqd_test_chain_id = environment::cheqd_test_chain_id();
            cheqd_pool::add(&setup.name, &cheqd_test_pool_ip, &cheqd_test_chain_id, Some("InMemory")).unwrap();
            let query_resp = cheqd_pool::abci_info(&setup.name).unwrap();
            println!("Query response: {:?}", query_resp);
        }
    }
}
