#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

pub mod utils;

#[cfg(test)]
#[cfg(feature = "agency_pool_tests")]
mod integration_tests {
    use crate::utils::force_debug_stack;
    use aries_vcx::messages::connection::did::Did;
    use aries_vcx::messages::did_doc::service_aries::AriesService;
    use aries_vcx::utils::constants::DEFAULT_SCHEMA_ATTRS;
    use aries_vcx::utils::devsetup::{SetupWalletPool, SetupProfile};
    use aries_vcx::xyz::keys::{get_verkey_from_ledger, rotate_verkey};
    use aries_vcx::xyz::ledger::transactions::add_new_did;
    use aries_vcx::xyz::test_utils::create_and_store_nonrevocable_credential_def;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[tokio::test]
    async fn test_open_close_pool() {
        SetupWalletPool::run(|setup| async move {
            assert!(setup.pool_handle > 0);
        }).await;
    }

    #[tokio::test]
    async fn test_get_credential_def() {
        // todo - use SetupProfile::run after modular impls
        SetupProfile::run_indy(|setup| async move {
        let (_, _, cred_def_id, cred_def_json, _) =
            create_and_store_nonrevocable_credential_def(&setup.profile, &setup.institution_did, DEFAULT_SCHEMA_ATTRS)
                .await;

        let ledger = Arc::clone(&setup.profile).inject_ledger();

        let r_cred_def_json = ledger.get_cred_def(&cred_def_id, None).await.unwrap();

        let def1: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
        let def2: serde_json::Value = serde_json::from_str(&r_cred_def_json).unwrap();
        assert_eq!(def1, def2);
        }).await;
    }

    #[tokio::test]
    async fn test_rotate_verkey() {
        SetupProfile::run(|setup| async move {
        let (did, verkey) = add_new_did(&setup.profile, &setup.institution_did, None).await.unwrap();
        rotate_verkey(&setup.profile, &did).await.unwrap();
        thread::sleep(Duration::from_millis(100));
        let local_verkey = setup.profile.inject_wallet().key_for_local_did(&did).await.unwrap();

        let ledger_verkey = get_verkey_from_ledger(&setup.profile, &did).await.unwrap();
        assert_ne!(verkey, ledger_verkey);
        assert_eq!(local_verkey, ledger_verkey);
        }).await;
    }

    // TODO - bring back after endorser methods added to baseledger
    // #[tokio::test]
    // async fn test_endorse_transaction() {
    //     SetupWalletPool::run(|setup| async move {
    //         endorse_transaction(setup.wallet_handle, setup.pool_handle, &endorser_did, &schema_request).await.unwrap();
    //     }).await;
    // }
    
    #[tokio::test]
    async fn test_add_get_service() {
        SetupProfile::run(|setup| async move {
        let ledger = Arc::clone(&setup.profile).inject_ledger();
        let did = setup.institution_did.clone();
        let expect_service = AriesService::default();

        ledger.add_service(&did, &expect_service).await.unwrap();
        thread::sleep(Duration::from_millis(50));
        let service = ledger.get_service(&Did::new(&did).unwrap()).await.unwrap();

        assert_eq!(expect_service, service)
        }).await;
    }
}
