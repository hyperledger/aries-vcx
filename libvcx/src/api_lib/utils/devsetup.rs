use aries_vcx::agency_client::agency_client::AgencyClient;
use aries_vcx::global::pool::{reset_main_pool_handle, set_main_pool_handle};
use aries_vcx::indy::WalletHandle;
use aries_vcx::utils::devsetup::SetupWalletPoolAgency;
use crate::api_lib::global::agency_client::{reset_main_agency_client, set_main_agency_client};
use crate::api_lib::global::wallet::{reset_main_wallet_handle, set_main_wallet_handle};

pub struct SetupGlobalsWalletPoolAgency {
    setup: SetupWalletPoolAgency
}

impl SetupGlobalsWalletPoolAgency {
    pub async fn init() -> SetupGlobalsWalletPoolAgency {
        let setup = SetupWalletPoolAgency::init().await;
        set_main_wallet_handle(setup.wallet_handle);
        set_main_agency_client(setup.agency_client.clone());
        SetupGlobalsWalletPoolAgency {
            setup
        }
    }
}

impl Drop for SetupGlobalsWalletPoolAgency {
    fn drop(&mut self) {
        reset_main_wallet_handle();
        reset_main_agency_client();
    }
}
