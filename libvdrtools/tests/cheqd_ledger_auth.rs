#![cfg(feature = "cheqd")]
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

#[cfg(feature = "local_nodes_cheqd_pool")]
use serde_json::Value;
#[cfg(feature = "local_nodes_cheqd_pool")]
use utils::{cheqd_ledger, cheqd_pool, cheqd_setup};

mod high_cases {
    #[cfg(feature = "local_nodes_cheqd_pool")]
    use super::*;

    #[cfg(test)]
    mod build_tx {
        #[cfg(feature = "local_nodes_cheqd_pool")]
        use super::*;

        #[test]
        #[cfg(feature = "local_nodes_cheqd_pool")]
        fn test_build_tx() {
            let setup = cheqd_setup::CheqdSetup::new();

            let (account_number, account_sequence) = setup
                .get_base_account_number_and_sequence(&setup.account_id)
                .unwrap();

            // Message
            let msg = cheqd_ledger::bank::build_msg_send(
                &setup.account_id,
                "second_account",
                "1000000",
                &setup.denom,
            )
            .unwrap();

            // Tx
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

            println!("Tx: {:?}", tx);
            assert_ne!(tx.len(), 0);
        }
    }

    #[cfg(test)]
    mod query_account {
        #[cfg(feature = "local_nodes_cheqd_pool")]
        use super::*;
        #[cfg(feature = "local_nodes_cheqd_pool")]
        use rstest::rstest;

        #[test]
        #[cfg(feature = "cheqd")]
        #[cfg(feature = "local_nodes_cheqd_pool")]
        fn test_query_account() {
            let setup = cheqd_setup::CheqdSetup::new();

            let query = cheqd_ledger::auth::build_query_account(&setup.account_id).unwrap();
            let resp = cheqd_pool::abci_query(&setup.pool_alias, &query).unwrap();
            let parsed = cheqd_ledger::auth::parse_query_account_resp(&resp).unwrap();

            println!("Parsed query response: {:?}", parsed);
        }

        #[cfg(feature = "local_nodes_cheqd_pool")]
        fn get_account_type_from_str(account_resp: String) -> String {
            let resp: Value = serde_json::from_str(&account_resp).unwrap();
            let account = resp["account"].as_object().unwrap();
            let account_type = account["type_url"].as_str().unwrap().to_string();
            return account_type.clone();
        }

        #[cfg(feature = "cheqd")]
        #[rstest(
            alias,
            account_id,
            expected_type,
            case(
                "baseVesting",
                "cheqd1lkqddnapqvz2hujx2trpj7xj6c9hmuq7uhl0md",
                "BaseVestingAccount"
            ),
            case(
                "continuousVesting",
                "cheqd1353p46macvn444rupg2jstmx3tmz657yt9gl4l",
                "ContinuousVestingAccount"
            ),
            case(
                "delayedVesting",
                "cheqd1njwu33lek5jt4kzlmljkp366ny4qpqusahpyrj",
                "DelayedVestingAccount"
            ),
            case(
                "periodicVesting",
                "cheqd1uyngr0l3xtyj07js9sdew9mk50tqeq8lghhcfr",
                "PeriodicVestingAccount"
            )
        )]
        #[cfg(feature = "local_nodes_cheqd_pool")]
        fn test_query_accounts(alias: &str, account_id: &str, expected_type: &str) {
            trace!("test_query_accounts >> alias {}", alias); // TODO VE-3079 unused alias
            let setup = cheqd_setup::CheqdSetup::new();
            let query = cheqd_ledger::auth::build_query_account(account_id).unwrap();
            let resp = cheqd_pool::abci_query(&setup.pool_alias, &query).unwrap();
            let account_resp = cheqd_ledger::auth::parse_query_account_resp(resp.as_str()).unwrap();
            assert_eq!(
                expected_type.to_string(),
                get_account_type_from_str(account_resp)
            )
        }
    }
}
