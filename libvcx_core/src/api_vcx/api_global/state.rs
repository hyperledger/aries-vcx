use crate::api_vcx::api_global::agency_client::reset_main_agency_client;
use crate::api_vcx::api_global::pool::{close_main_pool, reset_global_ledger_components};

use crate::api_vcx::api_global::settings::get_config_value;
use crate::api_vcx::api_global::wallet::close_main_wallet;
use crate::errors::error::LibvcxResult;
use aries_vcx::aries_vcx_core::indy::ledger::pool;
use aries_vcx::aries_vcx_core::indy::wallet::{delete_wallet, WalletConfig};
use aries_vcx::global::settings::{
    reset_config_values_ariesvcx, CONFIG_WALLET_KEY, CONFIG_WALLET_KEY_DERIVATION, CONFIG_WALLET_NAME,
    CONFIG_WALLET_TYPE, DEFAULT_POOL_NAME, DEFAULT_WALLET_NAME, UNINITIALIZED_WALLET_KEY, WALLET_KDF_DEFAULT,
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

    let _ = reset_config_values_ariesvcx();
    reset_main_agency_client();
    match reset_global_ledger_components() {
        Ok(_) => {}
        Err(err) => {
            error!("Failed to reset global pool: {}", err);
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::api_vcx::api_global::profile::get_main_wallet;
    use crate::api_vcx::api_global::state::state_vcx_shutdown;
    use crate::api_vcx::api_handle::credential::credential_create_with_offer;
    use crate::api_vcx::api_handle::disclosed_proof::create_with_proof_request;
    use crate::api_vcx::api_handle::schema::create_and_publish_schema;
    use crate::api_vcx::api_handle::{
        credential, credential_def, disclosed_proof, issuer_credential, mediated_connection, proof, schema,
    };
    use aries_vcx::aries_vcx_core::INVALID_WALLET_HANDLE;
    use aries_vcx::utils::devsetup::SetupMocks;
    use aries_vcx::utils::mockdata::mockdata_credex::ARIES_CREDENTIAL_OFFER;
    use aries_vcx::utils::mockdata::mockdata_proof::ARIES_PROOF_REQUEST_PRESENTATION;

    #[tokio::test]
    async fn test_shutdown() {
        let _setup = SetupMocks::init();

        let data = r#"["name","male"]"#;
        let connection = mediated_connection::test_utils::build_test_connection_inviter_invited().await;
        let credential_def = credential_def::create("SID".to_string(), "id".to_string(), "tag".to_string(), false)
            .await
            .unwrap();
        let issuer_credential = issuer_credential::issuer_credential_create("1".to_string()).unwrap();
        let proof = proof::create_proof(
            "1".to_string(),
            "[]".to_string(),
            "[]".to_string(),
            r#"{"support_revocation":false}"#.to_string(),
            "Optional".to_owned(),
        )
        .await
        .unwrap();
        let schema = create_and_publish_schema("5", "name".to_string(), "0.1".to_string(), data.to_string())
            .await
            .unwrap();
        let disclosed_proof = create_with_proof_request("id", ARIES_PROOF_REQUEST_PRESENTATION).unwrap();
        let credential = credential_create_with_offer("name", ARIES_CREDENTIAL_OFFER).unwrap();

        state_vcx_shutdown();
        assert_eq!(mediated_connection::is_valid_handle(connection), false);
        assert_eq!(issuer_credential::is_valid_handle(issuer_credential), false);
        assert_eq!(schema::is_valid_handle(schema), false);
        assert_eq!(proof::is_valid_handle(proof), false);
        assert_eq!(credential_def::is_valid_handle(credential_def), false);
        assert_eq!(credential::is_valid_handle(credential), false);
        assert_eq!(disclosed_proof::is_valid_handle(disclosed_proof), false);
        assert!(get_main_wallet().is_err());
    }
}
