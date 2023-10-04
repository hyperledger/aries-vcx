use crate::api_vcx::api_global::{
    agency_client::reset_main_agency_client,
    pool::{close_main_pool, reset_ledger_components},
    wallet::close_main_wallet,
};

pub fn state_vcx_shutdown() {
    info!("vcx_shutdown >>>");

    if let Ok(()) = futures::executor::block_on(close_main_wallet()) {}
    if let Ok(()) = futures::executor::block_on(close_main_pool()) {}

    crate::api_vcx::api_handle::schema::release_all();
    crate::api_vcx::api_handle::mediated_connection::release_all();
    crate::api_vcx::api_handle::issuer_credential::release_all();
    crate::api_vcx::api_handle::credential_def::release_all();
    crate::api_vcx::api_handle::proof::release_all();
    crate::api_vcx::api_handle::disclosed_proof::release_all();
    crate::api_vcx::api_handle::credential::release_all();

    reset_main_agency_client();
    match reset_ledger_components() {
        Ok(_) => {}
        Err(err) => {
            error!("Failed to reset global pool: {}", err);
        }
    }
}
