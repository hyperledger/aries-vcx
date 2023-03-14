use std::sync::Arc;

use aries_vcx::{
    core::profile::{indy_profile::IndySdkProfile, profile::Profile},
    global::settings::indy_mocks_enabled,
    plugins::wallet::{base_wallet::BaseWallet, indy_wallet::IndySdkWallet},
    utils::mockdata::profile::mock_profile::MockProfile,
    vdrtools::{PoolHandle, WalletHandle},
};

use super::{pool::get_main_pool_handle, wallet::get_main_wallet_handle};
use crate::errors::error::LibvcxResult;

pub fn indy_wallet_handle_to_wallet(wallet_handle: WalletHandle) -> Arc<dyn BaseWallet> {
    Arc::new(IndySdkWallet::new(wallet_handle))
}

pub fn indy_handles_to_profile(wallet_handle: WalletHandle, pool_handle: PoolHandle) -> Arc<dyn Profile> {
    Arc::new(IndySdkProfile::new(wallet_handle, pool_handle))
}

pub fn get_main_wallet() -> Arc<dyn BaseWallet> {
    indy_wallet_handle_to_wallet(get_main_wallet_handle())
}

pub fn get_main_profile() -> LibvcxResult<Arc<dyn Profile>> {
    if indy_mocks_enabled() {
        return Ok(Arc::new(MockProfile {}));
    }
    Ok(indy_handles_to_profile(
        get_main_wallet_handle(),
        get_main_pool_handle()?,
    ))
}

// constructs an indy profile under the condition where a pool_handle is NOT required
// - e.g. where only a Wallet is used (no ledger interactions). Should be used sparingly.
pub fn get_main_profile_optional_pool() -> Arc<dyn Profile> {
    if indy_mocks_enabled() {
        return Arc::new(MockProfile {});
    }
    // attempt to get the pool_handle if possible, else use '-1'
    let pool_handle = get_main_pool_handle().ok().map_or(-1, |p| p);
    indy_handles_to_profile(get_main_wallet_handle(), pool_handle)
}
