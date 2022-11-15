#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

extern crate vdrtoolsrs as vdrtools;

pub mod utils;

#[cfg(test)]
#[cfg(feature = "agency_pool_tests")]
mod integration_tests {
    use crate::utils::force_debug_stack;
    use aries_vcx::messages::connection::did::Did;
    use aries_vcx::messages::did_doc::service_aries::AriesService;
    use aries_vcx::utils::constants::DEFAULT_SCHEMA_ATTRS;
    use aries_vcx::utils::devsetup::{SetupIndyWalletPool, SetupProfile};
    use aries_vcx::xyz::keys::{get_verkey_from_ledger, rotate_verkey};
    use aries_vcx::xyz::ledger::transactions::add_new_did;
    use aries_vcx::xyz::test_utils::create_and_store_nonrevocable_credential_def;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[tokio::test]
    async fn test_open_close_pool() {
        let setup = SetupIndyWalletPool::init().await;
        assert!(setup.pool_handle > 0);
    }

    #[tokio::test]
    async fn test_get_credential_def() {
        // FUTURE - use dynamic setupprofile (init()) after credx anoncreds impl
        let setup = SetupProfile::init_indy().await;
        let (_, _, cred_def_id, cred_def_json, _) =
            create_and_store_nonrevocable_credential_def(&setup.profile, &setup.institution_did, DEFAULT_SCHEMA_ATTRS)
                .await;

        let ledger = Arc::clone(&setup.profile).inject_ledger();

        let r_cred_def_json = ledger.get_cred_def(&cred_def_id, None).await.unwrap();

        let def1: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
        let def2: serde_json::Value = serde_json::from_str(&r_cred_def_json).unwrap();
        assert_eq!(def1, def2);
    }

    #[tokio::test]
    async fn test_rotate_verkey() {
        let setup = SetupProfile::init().await;
        let (did, verkey) = add_new_did(&setup.profile, &setup.institution_did, None).await.unwrap();
        rotate_verkey(&setup.profile, &did).await.unwrap();
        thread::sleep(Duration::from_millis(100));
        let local_verkey = setup.profile.inject_wallet().key_for_local_did(&did).await.unwrap();

        let ledger_verkey = get_verkey_from_ledger(&setup.profile, &did).await.unwrap();
        assert_ne!(verkey, ledger_verkey);
        assert_eq!(local_verkey, ledger_verkey);
    }

    // TODO - bring back after endorser methods added to baseledger
    // #[tokio::test]
    // async fn test_endorse_transaction() {
    //     let setup = SetupProfile::init().await;

    //     let (author_did, _) = add_new_did(&setup.profile, &setup.institution_did, None).await.unwrap();
    //     let (endorser_did, _) = add_new_did(&setup.profile, &setup.institution_did, Some("ENDORSER")).await.unwrap();

    //     let schema_request = libindy_build_schema_request(&author_did, SCHEMA_DATA).await.unwrap();
    //     let schema_request = append_request_endorser(&schema_request, &endorser_did).await.unwrap();
    //     let schema_request = multisign_request(setup.wallet_handle, &author_did, &schema_request)
    //         .await
    //         .unwrap();

    //     endorse_transaction(setup.wallet_handle, setup.pool_handle, &endorser_did, &schema_request).await.unwrap();
    // }

    #[tokio::test]
    async fn test_add_get_service() {
        let setup = SetupProfile::init().await;
        let ledger = Arc::clone(&setup.profile).inject_ledger();
        let did = setup.institution_did.clone();
        let expect_service = AriesService::default();

        ledger.add_service(&did, &expect_service).await.unwrap();
        thread::sleep(Duration::from_millis(50));
        let service = ledger.get_service(&Did::new(&did).unwrap()).await.unwrap();

        assert_eq!(expect_service, service)
    }
}
