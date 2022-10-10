#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

extern crate vdrtoolsrs as vdrtools;

pub mod utils;

#[cfg(test)]
#[cfg(feature = "pool_tests")]
mod integration_tests {
    use aries_vcx::messages::did_doc::service_aries::AriesService;
    use aries_vcx::indy::ledger::transactions::get_cred_def_json;
    use aries_vcx::indy::test_utils::create_and_store_nonrevocable_credential_def;
    use aries_vcx::indy::ledger::transactions::{
        add_new_did, add_service, endorse_transaction, get_service, libindy_build_schema_request, multisign_request,
    };
    use aries_vcx::indy::keys::{get_verkey_from_ledger, get_verkey_from_wallet, rotate_verkey};
    use aries_vcx::messages::connection::did::Did;
    use aries_vcx::utils::constants::{DEFAULT_SCHEMA_ATTRS, SCHEMA_DATA};
    use aries_vcx::utils::devsetup::SetupWalletPool;
    use vdrtools::ledger::append_request_endorser;
    use std::thread;
    use std::time::Duration;

    #[tokio::test]
    async fn test_open_close_pool() {
        let setup = SetupWalletPool::init().await;
        assert!(setup.pool_handle > 0);
    }

    #[tokio::test]
    async fn test_get_credential_def() {
        let setup = SetupWalletPool::init().await;
        let (_, _, cred_def_id, cred_def_json, _) =
            create_and_store_nonrevocable_credential_def(setup.wallet_handle, setup.pool_handle, &setup.institution_did, DEFAULT_SCHEMA_ATTRS).await;

        let (id, r_cred_def_json) = get_cred_def_json(setup.wallet_handle, setup.pool_handle, &cred_def_id).await.unwrap();

        assert_eq!(id, cred_def_id);
        let def1: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
        let def2: serde_json::Value = serde_json::from_str(&r_cred_def_json).unwrap();
        assert_eq!(def1, def2);
    }

    #[tokio::test]
    async fn test_rotate_verkey() {
        let setup = SetupWalletPool::init().await;
        let (did, verkey) = add_new_did(setup.wallet_handle, setup.pool_handle, &setup.institution_did, None).await;
        rotate_verkey(setup.wallet_handle, setup.pool_handle, &did).await.unwrap();
        thread::sleep(Duration::from_millis(100));
        let local_verkey = get_verkey_from_wallet(setup.wallet_handle, &did).await.unwrap();
        let ledger_verkey = get_verkey_from_ledger(setup.pool_handle, &did).await.unwrap();
        assert_ne!(verkey, ledger_verkey);
        assert_eq!(local_verkey, ledger_verkey);
    }

    #[tokio::test]
    async fn test_endorse_transaction() {
        let setup = SetupWalletPool::init().await;

        let (author_did, _) = add_new_did(setup.wallet_handle, setup.pool_handle, &setup.institution_did, None).await;
        let (endorser_did, _) = add_new_did(setup.wallet_handle, setup.pool_handle, &setup.institution_did, Some("ENDORSER")).await;

        let schema_request = libindy_build_schema_request(&author_did, SCHEMA_DATA).await.unwrap();
        let schema_request = append_request_endorser(&schema_request, &endorser_did).await.unwrap();
        let schema_request = multisign_request(setup.wallet_handle, &author_did, &schema_request)
            .await
            .unwrap();

        endorse_transaction(setup.wallet_handle, setup.pool_handle, &endorser_did, &schema_request).await.unwrap();
    }

    #[tokio::test]
    async fn test_add_get_service() {
        let setup = SetupWalletPool::init().await;

        let did = setup.institution_did.clone();
        let expect_service = AriesService::default();
        add_service(setup.wallet_handle, setup.pool_handle, &did, &expect_service).await.unwrap();
        thread::sleep(Duration::from_millis(50));
        let service = get_service(setup.pool_handle, &Did::new(&did).unwrap()).await.unwrap();

        assert_eq!(expect_service, service)
    }
}
