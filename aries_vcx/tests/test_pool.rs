#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

pub mod utils;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[cfg(feature = "migration")]
use crate::utils::migration::Migratable;
use crate::utils::test_agent::{create_test_agent, create_test_agent_trustee};
use aries_vcx::common::keys::{get_verkey_from_ledger, rotate_verkey};
use aries_vcx::common::ledger::service_didsov::EndpointDidSov;
use aries_vcx::common::ledger::transactions::{
    add_attr, add_new_did, clear_attr, get_attr, get_service, write_endorser_did, write_endpoint, write_endpoint_legacy,
};
use aries_vcx::common::primitives::credential_definition::{CredentialDef, CredentialDefConfigBuilder};
use aries_vcx::common::primitives::revocation_registry::generate_rev_reg;
use aries_vcx::common::test_utils::{create_and_store_credential_def_and_rev_reg, create_and_write_test_schema};
use aries_vcx::errors::error::AriesVcxErrorKind;
use aries_vcx::utils::constants::DEFAULT_SCHEMA_ATTRS;
use aries_vcx::utils::devsetup::{SetupPoolDirectory, SetupProfile};
use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::ledger::base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite};
use aries_vcx_core::ledger::indy::pool::test_utils::get_temp_file_path;
use aries_vcx_core::wallet::indy::wallet::get_verkey_from_wallet;
use diddoc_legacy::aries::service::AriesService;

// TODO: Deduplicate this with create_and_store_credential_def_and_rev_reg
async fn create_and_store_nonrevocable_credential_def(
    anoncreds: &Arc<dyn BaseAnonCreds>,
    ledger_read: &Arc<dyn AnoncredsLedgerRead>,
    ledger_write: &Arc<dyn AnoncredsLedgerWrite>,
    issuer_did: &str,
    attr_list: &str,
) -> (String, String, String, String, CredentialDef) {
    let (schema_id, schema_json) = create_and_write_test_schema(anoncreds, ledger_write, issuer_did, attr_list).await;
    let config = CredentialDefConfigBuilder::default()
        .issuer_did(issuer_did)
        .schema_id(&schema_id)
        .tag("1")
        .build()
        .unwrap();
    let cred_def = CredentialDef::create(ledger_read, anoncreds, "1".to_string(), config, false)
        .await
        .unwrap()
        .publish_cred_def(ledger_read, ledger_write)
        .await
        .unwrap();
    let cred_def_id = cred_def.get_cred_def_id();
    tokio::time::sleep(Duration::from_millis(1000)).await;

    let cred_def_json = ledger_read.get_cred_def(&cred_def_id, None).await.unwrap();
    (schema_id, schema_json, cred_def_id, cred_def_json, cred_def)
}

#[tokio::test]
#[ignore]
async fn test_pool_rotate_verkey() {
    SetupProfile::run(|setup| async move {
        let (did, verkey) = add_new_did(
            &setup.profile.inject_wallet(),
            &setup.profile.inject_indy_ledger_write(),
            &setup.institution_did,
            None,
        )
        .await
        .unwrap();
        rotate_verkey(
            &setup.profile.inject_wallet(),
            &setup.profile.inject_indy_ledger_write(),
            &did,
        )
        .await
        .unwrap();
        tokio::time::sleep(Duration::from_millis(1000)).await;
        let local_verkey = setup.profile.inject_wallet().key_for_local_did(&did).await.unwrap();

        let ledger_verkey = get_verkey_from_ledger(&setup.profile.inject_indy_ledger_read(), &did)
            .await
            .unwrap();
        assert_ne!(verkey, ledger_verkey);
        assert_eq!(local_verkey, ledger_verkey);
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_pool_add_get_service() {
    SetupProfile::run(|setup| async move {
        let did = setup.institution_did.clone();
        let expect_service = AriesService::default();
        write_endpoint_legacy(&setup.profile.inject_indy_ledger_write(), &did, &expect_service)
            .await
            .unwrap();
        thread::sleep(Duration::from_millis(50));
        let service = get_service(&setup.profile.inject_indy_ledger_read(), &did)
            .await
            .unwrap();
        assert_eq!(expect_service, service);

        // clean up written legacy service
        clear_attr(
            &setup.profile.inject_indy_ledger_write(),
            &setup.institution_did,
            "service",
        )
        .await
        .unwrap();
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_pool_write_new_endorser_did() {
    SetupPoolDirectory::run(|setup| async move {
        let faber = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let acme = create_test_agent(setup.genesis_file_path.clone()).await;
        let acme_vk = get_verkey_from_wallet(acme.profile.inject_wallet().get_wallet_handle(), &acme.institution_did)
            .await
            .unwrap();

        let attrib_json = json!({ "attrib_name": "foo"}).to_string();
        assert!(add_attr(
            &acme.profile.inject_indy_ledger_write(),
            &acme.institution_did,
            &attrib_json
        )
        .await
        .is_err());
        write_endorser_did(
            &faber.profile.inject_indy_ledger_write(),
            &faber.institution_did,
            &acme.institution_did,
            &acme_vk,
            None,
        )
        .await
        .unwrap();
        thread::sleep(Duration::from_millis(50));
        add_attr(
            &acme.profile.inject_indy_ledger_write(),
            &acme.institution_did,
            &attrib_json,
        )
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
        write_endpoint(&setup.profile.inject_indy_ledger_write(), &did, &create_service)
            .await
            .unwrap();
        thread::sleep(Duration::from_millis(50));
        let service = get_service(&setup.profile.inject_indy_ledger_read(), &did)
            .await
            .unwrap();
        let expect_recipient_key =
            get_verkey_from_ledger(&setup.profile.inject_indy_ledger_read(), &setup.institution_did)
                .await
                .unwrap();
        let expect_service = AriesService::default()
            .set_service_endpoint("https://example.org".parse().unwrap())
            .set_recipient_keys(vec![expect_recipient_key])
            .set_routing_keys(vec!["did:sov:456".into()]);
        assert_eq!(expect_service, service);

        // clean up written endpoint
        clear_attr(
            &setup.profile.inject_indy_ledger_write(),
            &setup.institution_did,
            "endpoint",
        )
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
        write_endpoint(&setup.profile.inject_indy_ledger_write(), &did, &create_service)
            .await
            .unwrap();
        thread::sleep(Duration::from_millis(50));
        let service = get_service(&setup.profile.inject_indy_ledger_read(), &did)
            .await
            .unwrap();
        let expect_recipient_key =
            get_verkey_from_ledger(&setup.profile.inject_indy_ledger_read(), &setup.institution_did)
                .await
                .unwrap();
        let expect_service = AriesService::default()
            .set_service_endpoint("https://example.org".parse().unwrap())
            .set_recipient_keys(vec![expect_recipient_key])
            .set_routing_keys(vec![]);
        assert_eq!(expect_service, service);

        // clean up written endpoint
        clear_attr(
            &setup.profile.inject_indy_ledger_write(),
            &setup.institution_did,
            "endpoint",
        )
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
        write_endpoint_legacy(&setup.profile.inject_indy_ledger_write(), &did, &service_1)
            .await
            .unwrap();

        // Get service and verify it is in the old format
        let service = get_service(&setup.profile.inject_indy_ledger_read(), &did)
            .await
            .unwrap();
        assert_eq!(service_1, service);

        // Write new service format
        let endpoint_url_2 = "https://example2.org";
        let routing_keys_2 = vec![];
        let service_2 = EndpointDidSov::create()
            .set_service_endpoint(endpoint_url_2.parse().unwrap())
            .set_routing_keys(Some(routing_keys_2.clone()));
        write_endpoint(&setup.profile.inject_indy_ledger_write(), &did, &service_2)
            .await
            .unwrap();

        thread::sleep(Duration::from_millis(50));

        // Get service and verify it is in the new format
        let service = get_service(&setup.profile.inject_indy_ledger_read(), &did)
            .await
            .unwrap();
        let expect_recipient_key =
            get_verkey_from_ledger(&setup.profile.inject_indy_ledger_read(), &setup.institution_did)
                .await
                .unwrap();
        let expect_service = AriesService::default()
            .set_service_endpoint(endpoint_url_2.parse().unwrap())
            .set_recipient_keys(vec![expect_recipient_key])
            .set_routing_keys(routing_keys_2);
        assert_eq!(expect_service, service);

        // Clear up written endpoint
        clear_attr(
            &setup.profile.inject_indy_ledger_write(),
            &setup.institution_did,
            "endpoint",
        )
        .await
        .unwrap();

        thread::sleep(Duration::from_millis(50));

        // Get service and verify it is in the old format
        let service = get_service(&setup.profile.inject_indy_ledger_read(), &did)
            .await
            .unwrap();
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
        add_attr(&setup.profile.inject_indy_ledger_write(), &did, &attr_json.to_string())
            .await
            .unwrap();
        thread::sleep(Duration::from_millis(50));
        let attr = get_attr(&setup.profile.inject_indy_ledger_read(), &did, "attr_json")
            .await
            .unwrap();
        assert_eq!(attr, attr_json["attr_json"].to_string());

        clear_attr(&setup.profile.inject_indy_ledger_write(), &did, "attr_json")
            .await
            .unwrap();
        thread::sleep(Duration::from_millis(50));
        let attr = get_attr(&setup.profile.inject_indy_ledger_read(), &did, "attr_json")
            .await
            .unwrap();
        assert_eq!(attr, "");

        let attr = get_attr(&setup.profile.inject_indy_ledger_read(), &did, "nonexistent")
            .await
            .unwrap();
        assert_eq!(attr, "");
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_get_credential_def() {
    SetupProfile::run(|mut setup| async move {
        let (_, _, cred_def_id, cred_def_json, _) = create_and_store_nonrevocable_credential_def(
            &setup.profile.inject_anoncreds(),
            &setup.profile.inject_anoncreds_ledger_read(),
            &setup.profile.inject_anoncreds_ledger_write(),
            &setup.institution_did,
            DEFAULT_SCHEMA_ATTRS,
        )
        .await;

        #[cfg(feature = "migration")]
        setup.migrate().await;

        let ledger = Arc::clone(&setup.profile).inject_anoncreds_ledger_read();
        let r_cred_def_json = ledger.get_cred_def(&cred_def_id, None).await.unwrap();

        let def1: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
        let def2: serde_json::Value = serde_json::from_str(&r_cred_def_json).unwrap();
        assert_eq!(def1, def2);
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_pool_rev_reg_def_fails_for_cred_def_created_without_revocation() {
    SetupProfile::run(|setup| async move {
        // Cred def is created with support_revocation=false,
        // revoc_reg_def will fail in libindy because cred_Def doesn't have revocation keys
        let (_, _, cred_def_id, _, _) = create_and_store_nonrevocable_credential_def(
            &setup.profile.inject_anoncreds(),
            &setup.profile.inject_anoncreds_ledger_read(),
            &setup.profile.inject_anoncreds_ledger_write(),
            &setup.institution_did,
            DEFAULT_SCHEMA_ATTRS,
        )
        .await;

        let rc = generate_rev_reg(
            &setup.profile.inject_anoncreds(),
            &setup.institution_did,
            &cred_def_id,
            get_temp_file_path("path.txt").to_str().unwrap(),
            2,
            "tag1",
        )
        .await;

        #[cfg(feature = "modular_libs")]
        assert_eq!(rc.unwrap_err().kind(), AriesVcxErrorKind::InvalidState);
        #[cfg(not(feature = "modular_libs"))]
        assert_eq!(rc.unwrap_err().kind(), AriesVcxErrorKind::InvalidInput);
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_pool_get_rev_reg_def_json() {
    SetupProfile::run(|setup| async move {
        let attrs = r#"["address1","address2","city","state","zip"]"#;
        let (_, _, _, _, rev_reg_id, _, _, _) = create_and_store_credential_def_and_rev_reg(
            &setup.profile.inject_anoncreds(),
            &setup.profile.inject_anoncreds_ledger_read(),
            &setup.profile.inject_anoncreds_ledger_write(),
            &setup.institution_did,
            attrs,
        )
        .await;

        let ledger = Arc::clone(&setup.profile).inject_anoncreds_ledger_read();
        let _json = ledger.get_rev_reg_def_json(&rev_reg_id).await.unwrap();
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_pool_get_rev_reg_delta_json() {
    SetupProfile::run(|setup| async move {
        let attrs = r#"["address1","address2","city","state","zip"]"#;
        let (_, _, _, _, rev_reg_id, _, _, _) = create_and_store_credential_def_and_rev_reg(
            &setup.profile.inject_anoncreds(),
            &setup.profile.inject_anoncreds_ledger_read(),
            &setup.profile.inject_anoncreds_ledger_write(),
            &setup.institution_did,
            attrs,
        )
        .await;

        let ledger = Arc::clone(&setup.profile).inject_anoncreds_ledger_read();
        let (id, _delta, _timestamp) = ledger.get_rev_reg_delta_json(&rev_reg_id, None, None).await.unwrap();

        assert_eq!(id, rev_reg_id);
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_pool_get_rev_reg() {
    SetupProfile::run(|setup| async move {
        let attrs = r#"["address1","address2","city","state","zip"]"#;
        let (_, _, _, _, rev_reg_id, _, _, _) = create_and_store_credential_def_and_rev_reg(
            &setup.profile.inject_anoncreds(),
            &setup.profile.inject_anoncreds_ledger_read(),
            &setup.profile.inject_anoncreds_ledger_write(),
            &setup.institution_did,
            attrs,
        )
        .await;

        let ledger = Arc::clone(&setup.profile).inject_anoncreds_ledger_read();
        let (id, _rev_reg, _timestamp) = ledger
            .get_rev_reg(&rev_reg_id, time::OffsetDateTime::now_utc().unix_timestamp() as u64)
            .await
            .unwrap();

        assert_eq!(id, rev_reg_id);
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_pool_create_and_get_schema() {
    SetupProfile::run(|setup| async move {
        let (schema_id, _schema_json) = create_and_write_test_schema(
            &setup.profile.inject_anoncreds(),
            &setup.profile.inject_anoncreds_ledger_write(),
            &setup.institution_did,
            DEFAULT_SCHEMA_ATTRS,
        )
        .await;

        let ledger = Arc::clone(&setup.profile).inject_anoncreds_ledger_read();
        let rc = ledger.get_schema(&schema_id, None).await;

        let retrieved_schema = rc.unwrap();
        assert!(retrieved_schema.contains(&schema_id));
    })
    .await;
}
