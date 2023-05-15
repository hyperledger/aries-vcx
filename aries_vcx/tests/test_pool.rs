#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

pub mod utils;

#[cfg(test)]
mod integration_tests {
    use aries_vcx::common::keys::{get_verkey_from_ledger, rotate_verkey};
    use aries_vcx::common::ledger::service_didsov::EndpointDidSov;
    use aries_vcx::common::ledger::transactions::{
        add_attr, add_new_did, clear_attr, get_attr, get_service, write_endpoint, write_endpoint_legacy,
    };
    use aries_vcx::common::test_utils::create_and_store_nonrevocable_credential_def;
    use aries_vcx::utils::constants::DEFAULT_SCHEMA_ATTRS;
    use aries_vcx::utils::devsetup::{SetupProfile, SetupWalletPool};
    use diddoc::aries::service::AriesService;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[tokio::test]
    #[ignore]
    async fn test_pool_open_close_pool() {
        SetupWalletPool::run(|setup| async move {
            assert!(setup.pool_handle > 0);
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_pool_get_credential_def() {
        // TODO - use SetupProfile::run after modular impls
        SetupProfile::run_indy(|setup| async move {
            let (_, _, cred_def_id, cred_def_json, _) = create_and_store_nonrevocable_credential_def(
                &setup.profile,
                &setup.institution_did,
                DEFAULT_SCHEMA_ATTRS,
            )
            .await;

            let ledger = Arc::clone(&setup.profile).inject_ledger();

            let r_cred_def_json = ledger.get_cred_def(&cred_def_id, None).await.unwrap();

            let def1: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
            let def2: serde_json::Value = serde_json::from_str(&r_cred_def_json).unwrap();
            assert_eq!(def1, def2);
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_pool_rotate_verkey() {
        SetupProfile::run(|setup| async move {
            let (did, verkey) = add_new_did(&setup.profile, &setup.institution_did, None).await.unwrap();
            rotate_verkey(&setup.profile, &did).await.unwrap();
            tokio::time::sleep(Duration::from_millis(1000)).await;
            let local_verkey = setup.profile.inject_wallet().key_for_local_did(&did).await.unwrap();

            let ledger_verkey = get_verkey_from_ledger(&setup.profile, &did).await.unwrap();
            assert_ne!(verkey, ledger_verkey);
            assert_eq!(local_verkey, ledger_verkey);
        })
        .await;
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
    #[ignore]
    async fn test_pool_add_get_service() {
        SetupProfile::run(|setup| async move {
            let did = setup.institution_did.clone();
            let expect_service = AriesService::default();
            write_endpoint_legacy(&setup.profile, &did, &expect_service)
                .await
                .unwrap();
            thread::sleep(Duration::from_millis(50));
            let service = get_service(&setup.profile, &did).await.unwrap();
            assert_eq!(expect_service, service);

            // clean up written legacy service
            clear_attr(&setup.profile, &setup.institution_did, "service")
                .await
                .unwrap();
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_pool_add_get_service_public() {
        SetupProfile::run(|setup| async move {
            let did = setup.institution_did.clone();
            let create_service = EndpointDidSov::create()
                .set_service_endpoint("https://example.org".parse().unwrap())
                .set_routing_keys(Some(vec!["did:sov:456".into()]));
            write_endpoint(&setup.profile, &did, &create_service).await.unwrap();
            thread::sleep(Duration::from_millis(50));
            let service = get_service(&setup.profile, &did).await.unwrap();
            let expect_recipient_key = get_verkey_from_ledger(&setup.profile, &setup.institution_did)
                .await
                .unwrap();
            let expect_service = AriesService::default()
                .set_service_endpoint("https://example.org".parse().unwrap())
                .set_recipient_keys(vec![expect_recipient_key])
                .set_routing_keys(vec!["did:sov:456".into()]);
            assert_eq!(expect_service, service);

            // clean up written endpoint
            clear_attr(&setup.profile, &setup.institution_did, "endpoint")
                .await
                .unwrap();
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_pool_add_get_service_public_none_routing_keys() {
        SetupProfile::run(|setup| async move {
            let did = setup.institution_did.clone();
            let create_service = EndpointDidSov::create()
                .set_service_endpoint("https://example.org".parse().unwrap())
                .set_routing_keys(None);
            write_endpoint(&setup.profile, &did, &create_service).await.unwrap();
            thread::sleep(Duration::from_millis(50));
            let service = get_service(&setup.profile, &did).await.unwrap();
            let expect_recipient_key = get_verkey_from_ledger(&setup.profile, &setup.institution_did)
                .await
                .unwrap();
            let expect_service = AriesService::default()
                .set_service_endpoint("https://example.org".parse().unwrap())
                .set_recipient_keys(vec![expect_recipient_key])
                .set_routing_keys(vec![]);
            assert_eq!(expect_service, service);

            // clean up written endpoint
            clear_attr(&setup.profile, &setup.institution_did, "endpoint")
                .await
                .unwrap();
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_pool_multiple_service_formats() {
        SetupProfile::run(|setup| async move {
            let did = setup.institution_did.clone();

            // Write legacy service format
            let service_1 = AriesService::create()
                .set_service_endpoint("https://example1.org".parse().unwrap())
                .set_recipient_keys(vec!["did:sov:123".into()])
                .set_routing_keys(vec!["did:sov:456".into()]);
            write_endpoint_legacy(&setup.profile, &did, &service_1).await.unwrap();

            // Get service and verify it is in the old format
            let service = get_service(&setup.profile, &did).await.unwrap();
            assert_eq!(service_1, service);

            // Write new service format
            let endpoint_url_2 = "https://example2.org";
            let routing_keys_2 = vec![];
            let service_2 = EndpointDidSov::create()
                .set_service_endpoint(endpoint_url_2.parse().unwrap())
                .set_routing_keys(Some(routing_keys_2.clone()));
            write_endpoint(&setup.profile, &did, &service_2).await.unwrap();

            thread::sleep(Duration::from_millis(50));

            // Get service and verify it is in the new format
            let service = get_service(&setup.profile, &did).await.unwrap();
            let expect_recipient_key = get_verkey_from_ledger(&setup.profile, &setup.institution_did)
                .await
                .unwrap();
            let expect_service = AriesService::default()
                .set_service_endpoint(endpoint_url_2.parse().unwrap())
                .set_recipient_keys(vec![expect_recipient_key])
                .set_routing_keys(routing_keys_2);
            assert_eq!(expect_service, service);

            // Clear up written endpoint
            clear_attr(&setup.profile, &setup.institution_did, "endpoint")
                .await
                .unwrap();

            thread::sleep(Duration::from_millis(50));

            // Get service and verify it is in the old format
            let service = get_service(&setup.profile, &did).await.unwrap();
            assert_eq!(service_1, service);
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_pool_add_get_attr() {
        SetupProfile::run(|setup| async move {
            let did = setup.institution_did.clone();
            let attr_json = json!({
                "attr_json": {
                    "attr_key_1": "attr_value_1",
                    "attr_key_2": "attr_value_2",
                }
            });
            add_attr(&setup.profile, &did, &attr_json.to_string()).await.unwrap();
            thread::sleep(Duration::from_millis(50));
            let attr = get_attr(&setup.profile, &did, "attr_json").await.unwrap();
            assert_eq!(attr, attr_json["attr_json"].to_string());

            clear_attr(&setup.profile, &did, "attr_json").await.unwrap();
            thread::sleep(Duration::from_millis(50));
            let attr = get_attr(&setup.profile, &did, "attr_json").await.unwrap();
            assert_eq!(attr, "");

            let attr = get_attr(&setup.profile, &did, "nonexistent").await.unwrap();
            assert_eq!(attr, "");
        })
        .await;
    }
}
