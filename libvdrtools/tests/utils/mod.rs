#![allow(dead_code, unused_macros)]

use vdrtoolsrs::{ErrorCode, PoolHandle, WalletHandle, INVALID_POOL_HANDLE, INVALID_WALLET_HANDLE};

pub mod callback;

#[path = "../../indy-utils/src/environment.rs"]
pub mod environment;

pub mod anoncreds;
pub mod blob_storage;
pub mod constants;
pub mod crypto;
pub mod did;
pub mod ledger;
pub mod non_secrets;
pub mod pairwise;
pub mod pool;
pub mod results;
pub mod types;
pub mod wallet;
//pub mod payments;
pub mod cache;
#[cfg(feature = "cheqd")]
pub mod cheqd_keys;
#[cfg(feature = "cheqd")]
pub mod cheqd_ledger;
#[cfg(feature = "cheqd")]
pub mod cheqd_pool;
#[cfg(feature = "cheqd")]
pub mod cheqd_setup;
pub mod logger;
pub mod metrics;
pub mod rand_utils;
pub mod vdr;

#[macro_use]
#[allow(unused_macros)]
#[path = "../../indy-utils/src/test.rs"]
pub mod test;

pub mod timeout;

#[path = "../../indy-utils/src/sequence.rs"]
pub mod sequence;

#[macro_use]
#[allow(unused_macros)]
#[path = "../../indy-utils/src/ctypes.rs"]
pub mod ctypes;

#[macro_use]
#[path = "../../src/utils/qualifier.rs"]
pub mod qualifier;

#[path = "../../indy-utils/src/inmem_wallet.rs"]
pub mod inmem_wallet;

#[path = "../../indy-utils/src/wql.rs"]
pub mod wql;

#[path = "../../src/domain/mod.rs"]
pub mod domain;

fn setup() -> String {
    let name = crate::utils::rand_utils::get_rand_string(10);
    test::cleanup_storage(&name);
    logger::set_default_logger();
    name
}

fn tear_down(name: &str) {
    test::cleanup_storage(name);
}

pub struct Setup {
    pub name: String,
    pub wallet_config: String,
    pub wallet_handle: WalletHandle,
    pub pool_handle: PoolHandle,
    pub did: String,
    pub verkey: String,
    pub attached_wallets: Vec<(String, WalletHandle)>,
}

impl Setup {
    pub fn empty() -> Setup {
        let name = setup();
        Setup {
            name,
            wallet_config: String::new(),
            wallet_handle: INVALID_WALLET_HANDLE,
            pool_handle: INVALID_POOL_HANDLE,
            did: String::new(),
            verkey: String::new(),
            attached_wallets: Vec::new(),
        }
    }

    pub fn wallet() -> Setup {
        let name = setup();
        let (wallet_handle, wallet_config) = wallet::create_and_open_default_wallet(&name).unwrap();
        Setup {
            name,
            wallet_config,
            wallet_handle,
            pool_handle: INVALID_POOL_HANDLE,
            did: String::new(),
            verkey: String::new(),
            attached_wallets: Vec::new(),
        }
    }

    pub fn plugged_wallet() -> Setup {
        let name = setup();
        let (wallet_handle, wallet_config) = wallet::create_and_open_plugged_wallet().unwrap();
        Setup {
            name,
            wallet_config,
            wallet_handle,
            pool_handle: INVALID_POOL_HANDLE,
            did: String::new(),
            verkey: String::new(),
            attached_wallets: Vec::new(),
        }
    }

    #[cfg(feature = "local_nodes_pool")]
    pub fn pool() -> Setup {
        let name = setup();
        let pool_handle = pool::create_and_open_pool_ledger(&name).unwrap();
        Setup {
            name,
            wallet_config: String::new(),
            wallet_handle: INVALID_WALLET_HANDLE,
            pool_handle,
            did: String::new(),
            verkey: String::new(),
            attached_wallets: Vec::new(),
        }
    }

    #[cfg(feature = "local_nodes_pool")]
    pub fn pool_in_memory() -> Setup {
        let name = setup();
        let pool_handle = pool::open_in_memory_pool_ledger(&name).unwrap();
        let (wallet_handle, wallet_config) = wallet::create_and_open_default_wallet(&name).unwrap();
        Setup {
            name,
            wallet_config,
            wallet_handle,
            pool_handle,
            did: String::new(),
            verkey: String::new(),
            attached_wallets: Vec::new(),
        }
    }

    #[cfg(feature = "local_nodes_pool")]
    pub fn wallet_and_pool() -> Setup {
        let name = setup();
        let (wallet_handle, wallet_config) = wallet::create_and_open_default_wallet(&name).unwrap();
        let pool_handle = pool::create_and_open_pool_ledger(&name).unwrap();
        Setup {
            name,
            wallet_config,
            wallet_handle,
            pool_handle,
            did: String::new(),
            verkey: String::new(),
            attached_wallets: Vec::new(),
        }
    }

    #[cfg(feature = "local_nodes_pool")]
    pub fn trustee() -> Setup {
        let mut setup = Setup::wallet_and_pool();
        let (did, verkey) =
            did::create_and_store_my_did(setup.wallet_handle, Some(constants::TRUSTEE_SEED))
                .unwrap();
        setup.did = did;
        setup.verkey = verkey;
        setup
    }

    #[cfg(feature = "local_nodes_pool")]
    pub fn trustee_fully_qualified() -> Setup {
        let mut setup = Setup::wallet_and_pool();
        let (did, verkey) =
            did::create_and_store_my_did_v1(setup.wallet_handle, Some(constants::TRUSTEE_SEED))
                .unwrap();
        setup.did = did;
        setup.verkey = verkey;
        setup
    }

    #[cfg(feature = "local_nodes_pool")]
    pub fn steward() -> Setup {
        let mut setup = Setup::wallet_and_pool();
        let (did, verkey) =
            did::create_and_store_my_did(setup.wallet_handle, Some(constants::STEWARD_SEED))
                .unwrap();
        setup.did = did;
        setup.verkey = verkey;
        setup
    }

    #[cfg(feature = "local_nodes_pool")]
    pub fn endorser() -> Setup {
        let mut setup = Setup::wallet_and_pool();
        let (did, verkey) = did::create_store_and_publish_did(
            setup.wallet_handle,
            setup.pool_handle,
            "ENDORSER",
            None,
        )
        .unwrap();
        setup.did = did;
        setup.verkey = verkey;
        setup
    }

    #[cfg(feature = "local_nodes_pool")]
    pub fn new_identity() -> Setup {
        let mut setup = Setup::wallet_and_pool();
        let (did, verkey) = did::create_store_and_publish_did(
            setup.wallet_handle,
            setup.pool_handle,
            "TRUSTEE",
            None,
        )
        .unwrap();
        setup.did = did;
        setup.verkey = verkey;
        setup
    }

    pub fn did() -> Setup {
        let name = setup();
        let (wallet_handle, wallet_config) = wallet::create_and_open_default_wallet(&name).unwrap();
        let (did, verkey) = did::create_and_store_my_did(wallet_handle, None).unwrap();
        Setup {
            name,
            wallet_config,
            wallet_handle,
            pool_handle: 0,
            did,
            verkey,
            attached_wallets: Vec::new(),
        }
    }

    pub fn local_trustee() -> Setup {
        let name = setup();
        let (wallet_handle, wallet_config) = wallet::create_and_open_default_wallet(&name).unwrap();
        let (did, verkey) =
            did::create_and_store_my_did(wallet_handle, Some(constants::TRUSTEE_SEED)).unwrap();
        Setup {
            name,
            wallet_config,
            wallet_handle,
            pool_handle: 0,
            did,
            verkey,
            attached_wallets: Vec::new(),
        }
    }

    pub fn did_fully_qualified() -> Setup {
        let name = setup();
        let (wallet_handle, wallet_config) = wallet::create_and_open_default_wallet(&name).unwrap();
        let (did, verkey) = did::create_and_store_my_did_v1(wallet_handle, None).unwrap();
        Setup {
            name,
            wallet_config,
            wallet_handle,
            pool_handle: 0,
            did,
            verkey,
            attached_wallets: Vec::new(),
        }
    }

    pub fn key() -> Setup {
        let name = setup();
        let (wallet_handle, wallet_config) = wallet::create_and_open_default_wallet(&name).unwrap();
        let verkey = crypto::create_key(wallet_handle, None).unwrap();
        Setup {
            name,
            wallet_config,
            wallet_handle,
            pool_handle: INVALID_POOL_HANDLE,
            did: String::new(),
            verkey,
            attached_wallets: Vec::new(),
        }
    }

    pub fn attach_wallet(&mut self, wallet_config: String, wallet_handle: WalletHandle) {
        self.attached_wallets.push((wallet_config, wallet_handle));
    }

    //    pub fn payment() -> Setup {
    //        let name = setup();
    //        payments::mock_method::init();
    //        Setup { name, wallet_config: String::new(), wallet_handle: INVALID_WALLET_HANDLE, pool_handle: INVALID_POOL_HANDLE, did: String::new(), verkey: String::new() }
    //    }
    //
    //    pub fn payment_wallet() -> Setup {
    //        let name = setup();
    //        let (wallet_handle, wallet_config) = wallet::create_and_open_default_wallet(&name).unwrap();
    //        payments::mock_method::init();
    //        Setup { name, wallet_config, wallet_handle, pool_handle: INVALID_POOL_HANDLE, did: String::new(), verkey: String::new() }
    //    }
}

impl Drop for Setup {
    fn drop(&mut self) {
        if self.wallet_handle != INVALID_WALLET_HANDLE {
            wallet::close_and_delete_wallet(self.wallet_handle, &self.wallet_config).unwrap();
        }

        let wallets = self.attached_wallets.drain(0..);
        for (conf, handle) in wallets {
            wallet::close_and_delete_wallet(handle, &conf).unwrap();
        }

        if self.pool_handle != INVALID_POOL_HANDLE {
            pool::close(self.pool_handle).unwrap();
        }

        tear_down(&self.name);
    }
}
