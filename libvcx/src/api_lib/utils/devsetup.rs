use aries_vcx::utils::devsetup::SetupWalletPoolAgency;

use crate::api_lib::global::agency_client::{reset_main_agency_client, set_main_agency_client};
use crate::api_lib::global::wallet::{reset_main_wallet_handle, set_main_wallet_handle};
use crate::aries_vcx::global::pool::{set_main_pool_handle, reset_main_pool_handle};

pub struct SetupGlobalsWalletPoolAgency {
    pub setup: SetupWalletPoolAgency,
}

impl SetupGlobalsWalletPoolAgency {
    pub async fn init() -> SetupGlobalsWalletPoolAgency {
        let setup = SetupWalletPoolAgency::init().await;
        set_main_wallet_handle(setup.wallet_handle);
        set_main_agency_client(setup.agency_client.clone());
        set_main_pool_handle(Some(setup.pool_handle));
        SetupGlobalsWalletPoolAgency { setup }
    }
}

impl Drop for SetupGlobalsWalletPoolAgency {
    fn drop(&mut self) {
        reset_main_wallet_handle();
        reset_main_agency_client();
        reset_main_pool_handle();
    }
}
