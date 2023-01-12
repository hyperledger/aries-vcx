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

#[cfg(feature = "local_nodes_cheqd_pool")]
use utils::cheqd_ledger;
use utils::{cheqd_keys, cheqd_setup};

mod utils;

#[cfg(feature = "cheqd")]
mod high_cases {
    use super::*;
    const BIP39_PASSPHRASE: &str = "";

    #[cfg(test)]
    mod add_random {
        use super::*;

        #[test]
        fn test_add_random() {
            let alias = "some_alias";
            let setup = cheqd_setup::CheqdSetup::new();
            let result = cheqd_keys::add_random(setup.wallet_handle, alias).unwrap();
            println!("Data: {:?} ", result);
        }
    }

    #[cfg(test)]
    mod add_from_mnemonic {
        use super::*;

        #[test]
        fn test_add_from_mnemonic() {
            let alias = "some_alias_2";
            let mnemonic = "sell table balcony salad acquire love hover resist give baby liquid process lecture awkward injury crucial rack stem prepare bar unable among december ankle";
            let setup = cheqd_setup::CheqdSetup::new();
            let result = cheqd_keys::add_from_mnemonic(
                setup.wallet_handle,
                alias,
                mnemonic,
                BIP39_PASSPHRASE,
            )
            .unwrap();
            println!("Mnemonic: {:?}, Data: {:?}", mnemonic, result);
        }
    }

    mod key_info {
        use super::*;

        #[derive(Deserialize, Serialize)]
        struct PublicKeyJson {
            /// `@type` field e.g. `/cosmos.crypto.ed25519.PubKey`.
            #[serde(rename = "@type")]
            type_url: String,

            /// Key data: standard Base64 encoded with padding.
            key: String,
        }

        #[test]
        fn test_key_info() {
            let alias = "some_alias";
            let setup = cheqd_setup::CheqdSetup::new();
            cheqd_keys::add_random(setup.wallet_handle, alias).unwrap();
            let result = cheqd_keys::get_info(setup.wallet_handle, alias).unwrap();
            println!("Data: {:?} ", result);
        }

        #[test]
        fn test_get_list_keys() {
            let alias_1 = "some_alias_1";
            let alias_2 = "some_alias_2";

            let setup = cheqd_setup::CheqdSetup::new();

            let key_1 = cheqd_keys::add_random(setup.wallet_handle, alias_1).unwrap();
            let key_2 = cheqd_keys::add_random(setup.wallet_handle, alias_2).unwrap();

            let key_1 = serde_json::from_str::<serde_json::Value>(&key_1).unwrap();
            let key_2 = serde_json::from_str::<serde_json::Value>(&key_2).unwrap();

            let account_id_1 = key_1["account_id"].as_str().clone().unwrap();
            let pub_key_1 = key_1["pub_key"].as_str().clone().unwrap();
            let pub_key_1 = serde_json::from_str::<PublicKeyJson>(pub_key_1)
                .unwrap()
                .key;

            let account_id_2 = key_2["account_id"].as_str().clone().unwrap();
            let pub_key_2 = key_2["pub_key"].as_str().clone().unwrap();
            let pub_key_2 = serde_json::from_str::<PublicKeyJson>(pub_key_2)
                .unwrap()
                .key;

            let result = cheqd_keys::get_list_keys(setup.wallet_handle).unwrap();
            println!("List keys: {:?}", result);
            println!("Key1: {:?}", key_1);
            println!("Key2: {:?}", key_2);

            assert!(result.contains(&account_id_1));
            assert!(result.contains(&account_id_2));

            assert!(result.contains(&pub_key_1));
            assert!(result.contains(&pub_key_2));

            println!("Data: {:?} ", result);
        }
    }

    mod sign {
        #[cfg(feature = "local_nodes_cheqd_pool")]
        use super::*;

        #[test]
        #[cfg(feature = "local_nodes_cheqd_pool")]
        fn test_sign() {
            let setup = cheqd_setup::CheqdSetup::new();

            // Message
            let msg = cheqd_ledger::bank::build_msg_send(
                &setup.account_id,
                "second_account",
                "1000000",
                &setup.denom,
            )
            .unwrap();

            // Transaction
            let tx = cheqd_ledger::auth::build_tx(
                &setup.pool_alias,
                &setup.pub_key,
                &msg,
                0,
                0,
                90000,
                2250000u64,
                "ncheq",
                setup.get_timeout_height(),
                "memo",
            )
            .unwrap();

            let result =
                cheqd_ledger::auth::sign_tx(setup.wallet_handle, &setup.key_alias, &tx).unwrap();
            println!("Data: {:?} ", result);
        }
    }
}
