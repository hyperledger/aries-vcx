#![allow(dead_code, unused_macros)]

use serde_json::Value;
use vdrtoolsrs::IndyError;

use crate::utils::{cheqd_keys, cheqd_pool, environment};
#[cfg(feature = "local_nodes_cheqd_pool")]
use crate::utils::{cheqd_ledger, cheqd_ledger::auth};

use super::test;
use super::{logger, wallet, WalletHandle};

const BIP39_PASSPHRASE: &str = "";
pub const MAX_GAS: u64 = 90000;
pub const MAX_COIN_AMOUNT: u64 = 2250000u64;

fn setup() -> String {
    let name = crate::utils::rand_utils::get_rand_string(10);
    test::cleanup_storage(&name);
    logger::set_default_logger();
    name
}

fn tear_down(name: &str, wallet_handle: WalletHandle) {
    wallet::close_wallet(wallet_handle).unwrap();
    test::cleanup_storage(name);
}

fn wallet_config(name: &str) -> String {
    json!({ "id": name }).to_string()
}

pub struct CheqdSetup {
    pub name: String,
    pub pool_alias: String,
    pub key_alias: String,
    pub account_id: String,
    pub pub_key: String,
    pub wallet_handle: WalletHandle,
    pub denom: String,
}

impl CheqdSetup {
    pub fn new() -> CheqdSetup {
        let name = setup();

        // Wallet
        let wallet_config = wallet_config(&name);
        let (wallet_handle, _) = wallet::create_and_open_default_wallet(&wallet_config).unwrap();

        // Account
        let key_alias = "alice";
        let mnemonic = "sketch mountain erode window enact net enrich smoke claim kangaroo another visual write meat latin bacon pulp similar forum guilt father state erase bright";
        // let mnemonic = "shed drama more wrestle rural face example urban phrase practice day glow category list vehicle suggest deal surge clog idle cool foam dice exact";
        let (account_id, pub_key) =
            CheqdSetup::create_key(wallet_handle, key_alias, mnemonic).unwrap();

        // Pool
        let cheqd_test_pool_ip = environment::cheqd_test_pool_ip();
        let cheqd_test_chain_id = environment::cheqd_test_chain_id();
        cheqd_pool::add(&name, &cheqd_test_pool_ip, &cheqd_test_chain_id, None).unwrap();

        // Denom
        let denom = environment::cheqd_denom();

        let setup = CheqdSetup {
            name: name.clone(),
            pool_alias: name,
            key_alias: key_alias.to_string(),
            account_id,
            pub_key,
            wallet_handle,
            denom,
        };

        setup
    }

    pub fn create_key(
        wallet_handle: WalletHandle,
        alias: &str,
        mnemonic: &str,
    ) -> Result<(String, String), IndyError> {
        let key = cheqd_keys::add_from_mnemonic(wallet_handle, alias, mnemonic, BIP39_PASSPHRASE)
            .unwrap();
        let key: Value = serde_json::from_str(&key).unwrap();
        println!("Cheqd setup. Create key: {:?}", key);

        let account_id = key["account_id"].as_str().unwrap().to_string();
        let pub_key = key["pub_key"].as_str().unwrap().to_string();
        Ok((account_id, pub_key))
    }

    #[cfg(feature = "local_nodes_cheqd_pool")]
    pub fn get_base_account_number_and_sequence(
        &self,
        account_id: &str,
    ) -> Result<(u64, u64), IndyError> {
        let req = auth::build_query_account(account_id).unwrap();
        let resp = cheqd_pool::abci_query(&self.pool_alias, &req).unwrap();
        let resp = auth::parse_query_account_resp(&resp).unwrap();
        println!("Cheqd setup. Get account: {:?}", resp);

        let resp: Value = serde_json::from_str(&resp).unwrap();
        let account = resp["account"].as_object().unwrap();

        let base_account = if account["type_url"] == "ModuleAccount" {
            let module_account = account["value"].as_object().unwrap();
            module_account["base_account"].as_object().unwrap()
        } else if account["type_url"] == "BaseVestingAccount" {
            let base_vesting_account = account["value"].as_object().unwrap();
            base_vesting_account["base_account"].as_object().unwrap()
        } else if account["type_url"] == "ContinuousVestingAccount" {
            let continuous_vesting_account = account["value"].as_object().unwrap();
            let base_vesting_account = continuous_vesting_account["base_vesting_account"]
                .as_object()
                .unwrap();
            base_vesting_account["base_account"].as_object().unwrap()
        } else if account["type_url"] == "DelayedVestingAccount" {
            let delayed_vesting_account = account["value"].as_object().unwrap();
            let base_vesting_account = delayed_vesting_account["base_vesting_account"]
                .as_object()
                .unwrap();
            base_vesting_account["base_account"].as_object().unwrap()
        } else if account["type_url"] == "PeriodicVestingAccount" {
            let periodic_vesting_account = account["value"].as_object().unwrap();
            let base_vesting_account = periodic_vesting_account["base_vesting_account"]
                .as_object()
                .unwrap();
            base_vesting_account["base_account"].as_object().unwrap()
        } else {
            account["value"].as_object().unwrap()
        };

        let account_number = base_account["account_number"].as_u64().unwrap();
        let account_sequence = base_account["sequence"].as_u64().unwrap();

        Ok((account_number, account_sequence))
    }

    #[cfg(feature = "local_nodes_cheqd_pool")]
    pub fn build_and_sign_and_broadcast_tx(&self, msg: &[u8]) -> Result<String, IndyError> {
        // Get account info
        let (account_number, account_sequence) =
            self.get_base_account_number_and_sequence(&self.account_id)?;

        // Tx
        let tx = cheqd_ledger::auth::build_tx(
            &self.pool_alias,
            &self.pub_key,
            &msg,
            account_number,
            account_sequence,
            MAX_GAS,
            MAX_COIN_AMOUNT,
            &self.denom,
            self.get_timeout_height(),
            "memo",
        )?;

        // Sign
        let signed = cheqd_ledger::auth::sign_tx(self.wallet_handle, &self.key_alias, &tx)?;

        // Broadcast
        let resp = cheqd_pool::broadcast_tx_commit(&self.pool_alias, &signed)?;

        Ok(resp)
    }

    #[cfg(feature = "local_nodes_cheqd_pool")]
    pub fn get_timeout_height(&self) -> u64 {
        const TIMEOUT: u64 = 20;
        let info: String = cheqd_pool::abci_info(&self.pool_alias).unwrap();
        let info: Value = serde_json::from_str(&info).unwrap();
        let current_height = info["response"]["last_block_height"]
            .as_str()
            .unwrap()
            .parse::<u64>()
            .unwrap();
        println!("Cheqd setup. Last block height: {:?}", current_height);

        return current_height + TIMEOUT;
    }
}

impl Drop for CheqdSetup {
    fn drop(&mut self) {
        tear_down(&self.name, self.wallet_handle);
    }
}
