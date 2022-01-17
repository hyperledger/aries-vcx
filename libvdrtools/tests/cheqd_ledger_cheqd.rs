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

use serde_json::Value;
use utils::{cheqd_ledger, cheqd_pool, cheqd_setup, did};

#[cfg(feature = "cheqd")]
mod high_cases {
    use super::*;

    #[cfg(test)]
    mod create_did {
        use super::*;

        #[test]
        #[cfg(feature = "cheqd")]
        fn test_cheqd_create_did() {
            let setup = cheqd_setup::CheqdSetup::new();

            // Create DID
            let (did, verkey) =
                did::create_my_did(setup.wallet_handle, &cheqd_ledger::cheqd::did_info()).unwrap();

            // Send DID
            let msg = cheqd_ledger::cheqd::build_msg_create_did(&did, &verkey).unwrap();
            let resp =
                cheqd_ledger::cheqd::sign_and_broadcast_cheqd_msg(&setup, &did, msg).unwrap();

            // Parse response
            let tx_resp_parsed = cheqd_ledger::cheqd::parse_msg_create_did_resp(&resp).unwrap();
            println!("Tx response: {:?}", tx_resp_parsed);
            let tx_resp: Value = serde_json::from_str(&tx_resp_parsed).unwrap();
            let resp_id = tx_resp
                .as_object()
                .unwrap()
                .get("id")
                .unwrap()
                .as_str()
                .unwrap();

            assert_eq!(did, resp_id);
        }
    }

    #[cfg(test)]
    mod get_did {
        use super::*;

        #[test]
        #[cfg(feature = "cheqd")]
        fn test_cheqd_get_did() {
            let setup = cheqd_setup::CheqdSetup::new();

            // Create Did
            let (did, verkey) =
                did::create_my_did(setup.wallet_handle, &cheqd_ledger::cheqd::did_info()).unwrap();

            // Send Did
            let msg = cheqd_ledger::cheqd::build_msg_create_did(&did, &verkey).unwrap();
            println!("CreateDid message: {:?}", msg.clone());
            let _resp =
                cheqd_ledger::cheqd::sign_and_broadcast_cheqd_msg(&setup, &did, msg).unwrap();
            // TODO VE-3079 compare response vs get result

            // Get DID request
            let query = cheqd_ledger::cheqd::build_query_get_did(did.as_str()).unwrap();
            let query_resp = cheqd_pool::abci_query(&setup.pool_alias, &query).unwrap();

            // Parse response
            let query_resp = cheqd_ledger::cheqd::parse_query_get_did_resp(&query_resp).unwrap();
            println!("Query response: {:?}", query_resp);

            // Check response
            let query_resp: Value = serde_json::from_str(&query_resp).unwrap();
            let resp_did = query_resp.as_object().unwrap().get("did").unwrap();
            let resp_id = resp_did.get("id").unwrap().as_str().unwrap();
            assert_eq!(did, resp_id);
        }
    }

    #[cfg(test)]
    mod get_tx_by_hash {
        use super::*;

        // TODO: Use other message and remove `cheqd_nym_enable`
        #[test]
        #[cfg(feature = "cheqd")]
        #[ignore] // TODO VE-3079 debug intermittent test
        fn test_get_tx_by_hash() {

            let setup = cheqd_setup::CheqdSetup::new();
            let to_account = "cheqd1l9sq0se0jd3vklyrrtjchx4ua47awug5vsyeeh";
            let amount = "1000000";
            let msg = cheqd_ledger::bank::build_msg_send(
                &setup.account_id,
                to_account,
                amount,
                &setup.denom,
            ).unwrap();

            let resp = setup.build_and_sign_and_broadcast_tx(&msg).unwrap();
            println!("Response broadcast tx:{:?}", resp);
            let resp_json: Value = serde_json::from_str(&resp).unwrap();
            let hash = resp_json["hash"].as_str().unwrap();

            println!("Requested hash: {:?}", hash);

            let get_tx_req = cheqd_ledger::tx::build_query_get_tx_by_hash(&hash).unwrap();
            let result = cheqd_pool::abci_query(&setup.pool_alias, &get_tx_req).unwrap();

            let query_resp_parsed = cheqd_ledger::tx::parse_query_get_tx_by_hash_resp(result.as_str()).unwrap();
            println!("Query get txn by hash result: {:?}", query_resp_parsed);
            assert!(query_resp_parsed.contains(setup.account_id.as_str()));
            assert!(query_resp_parsed.contains(amount));
            assert!(query_resp_parsed.contains(to_account));
        }
    }

    #[cfg(test)]
    mod update_did {
        use super::*;

        #[test]
        #[cfg(feature = "cheqd")]
        fn test_cheqd_update_did() {
            let setup = cheqd_setup::CheqdSetup::new();

            // Create Did
            let (did, verkey) = did::create_my_did(setup.wallet_handle, &cheqd_ledger::cheqd::did_info()).unwrap();

            // Send Did
            let msg = cheqd_ledger::cheqd::build_msg_create_did(&did, &verkey).unwrap();
            let _resp = cheqd_ledger::cheqd::sign_and_broadcast_cheqd_msg(&setup, &did, msg).unwrap();
            // TODO VE-3079 compare response vs get result


            // Get DID request
            let query = cheqd_ledger::cheqd::build_query_get_did(did.as_str()).unwrap();
            let query_resp = cheqd_pool::abci_query(&setup.pool_alias, &query).unwrap();
            let query_resp = cheqd_ledger::cheqd::parse_query_get_did_resp(&query_resp).unwrap();
            println!("Query response: {:?}", query_resp);

            // Get tx_hash
            let query_resp: Value = serde_json::from_str(&query_resp).unwrap();
            let resp_metadata = query_resp.as_object().unwrap().get("metadata").unwrap();
            let resp_version_id = resp_metadata.get("version_id").unwrap().as_str().unwrap();

            // Generate new verkey
            let new_verkey = did::replace_keys_start(setup.wallet_handle, &did, "{}").unwrap();
            did::replace_keys_apply(setup.wallet_handle, &did).unwrap();

            // Build msg_update_did
            let msg = cheqd_ledger::cheqd::build_msg_update_did(
                &did,
                new_verkey.as_str(),
                &resp_version_id,
            ).unwrap();
            let resp =
                cheqd_ledger::cheqd::sign_and_broadcast_cheqd_msg(&setup, &did, msg).unwrap();

            // Parse the response
            let tx_resp_parsed = cheqd_ledger::cheqd::parse_msg_update_did_resp(&resp).unwrap();
            let tx_resp_parsed: Value = serde_json::from_str(&tx_resp_parsed).unwrap();
            let resp_id = tx_resp_parsed.get("id").unwrap().as_str().unwrap();

            assert_eq!(did, resp_id);
        }
    }
}
