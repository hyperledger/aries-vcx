#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

pub mod utils;

#[cfg(test)]
#[cfg(feature = "pool_tests")]
mod integration_tests {
    use aries_vcx::messages::connection::did::Did;
    use aries_vcx::messages::did_doc::service_aries::AriesService;
    use aries_vcx::utils::constants::{DEFAULT_SCHEMA_ATTRS, SCHEMA_DATA};
    use aries_vcx::utils::devsetup::{SetupWalletPool, SetupProfile};
    use aries_vcx::common::keys::{get_verkey_from_ledger, rotate_verkey};
    use aries_vcx::common::ledger::transactions::{add_new_did, write_endpoint_legacy, write_endpoint, get_service, clear_attr};
    use aries_vcx::common::test_utils::create_and_store_nonrevocable_credential_def;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    use messages::did_doc::service_aries_public::EndpointDidSov;

    #[tokio::test]
    async fn test_open_close_pool() {
        SetupWalletPool::run(|setup| async move {
            assert!(setup.pool_handle > 0);
        }).await;
    }

    #[tokio::test]
    async fn test_get_credential_def() {
        // TODO - use SetupProfile::run after modular impls
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

    // TODO - future - bring back after all endorser methods added to baseledger
    // #[tokio::test]
    // async fn test_endorse_transaction() {
    //     SetupProfile::run_indy(|setup| async move {
    //         let ledger = Arc::clone(&setup.profile).inject_ledger();
    //         let (author_did, _) = add_new_did(&setup.profile, &setup.institution_did, None).await.unwrap();
    //         let (endorser_did, _) = add_new_did(&setup.profile, &setup.institution_did, Some("ENDORSER")).await.unwrap();
    
    //         let schema_request = ledger.build_schema_request(&author_did, SCHEMA_DATA).await.unwrap();
    //         let schema_request = append_request_endorser(&schema_request, &endorser_did).await.unwrap();
    //         let schema_request = multisign_request(setup.wallet_handle, &author_did, &schema_request)
    //             .await
    //             .unwrap();
    //         ledger.endorse_transaction(&endorser_did, &schema_request).await.unwrap();
    //     }).await;
    // }
    
    #[tokio::test]
    async fn test_add_get_service() {
        SetupProfile::run(|setup| async move {
        let did = setup.institution_did.clone();
        let expect_service = AriesService::default();
        write_endpoint_legacy(&setup.profile, &did, &expect_service).await.unwrap();
        thread::sleep(Duration::from_millis(50));
        let service = get_service(&setup.profile, &Did::new(&did).unwrap()).await.unwrap();
        assert_eq!(expect_service, service);

        // clean up written legacy service
        clear_attr(&setup.profile, &setup.institution_did, "service").await.unwrap();
        }).await;
    }

    #[tokio::test]
    async fn test_add_get_service_public() {
        SetupProfile::run(|setup| async move {
            let did = setup.institution_did.clone();
            let create_service = EndpointDidSov::create()
                .set_service_endpoint("https://example.org".into())
                .set_routing_keys(Some(vec!["did:sov:456".into()]));
            write_endpoint(&setup.profile, &did, &create_service).await.unwrap();
            thread::sleep(Duration::from_millis(50));
            let service = get_service(&setup.profile, &Did::new(&did).unwrap()).await.unwrap();
            let expect_recipient_key = get_verkey_from_ledger(&setup.profile, &setup.institution_did).await.unwrap();
            let expect_service = AriesService::default()
                .set_service_endpoint("https://example.org".into())
                .set_recipient_keys(vec![expect_recipient_key])
                .set_routing_keys(vec!["did:sov:456".into()]);
            assert_eq!(expect_service, service);
            
            // clean up written endpoint
            clear_attr(&setup.profile, &setup.institution_did, "endpoint").await.unwrap();
        }).await;
    }
}
