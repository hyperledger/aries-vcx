use std::future::Future;

use aries_vcx::utils::devsetup::SetupWalletPoolAgency;

use crate::api_vcx::api_global::{
    agency_client::{reset_main_agency_client, set_main_agency_client},
    pool::{reset_main_pool_handle, set_main_pool_handle},
    wallet::{reset_main_wallet_handle, set_main_wallet_handle},
};

pub struct SetupGlobalsWalletPoolAgency {
    pub setup: SetupWalletPoolAgency,
}

impl SetupGlobalsWalletPoolAgency {
    pub async fn run<F>(f: impl FnOnce(Self) -> F)
    where
        F: Future<Output = ()>,
    {
        SetupWalletPoolAgency::run(|setup| async move {
            set_main_wallet_handle(setup.wallet_handle);
            set_main_agency_client(setup.agency_client.clone());
            set_main_pool_handle(Some(setup.pool_handle));

            f(SetupGlobalsWalletPoolAgency { setup }).await;

            reset_main_wallet_handle();
            reset_main_agency_client();
            reset_main_pool_handle();
        })
        .await;
    }
}
