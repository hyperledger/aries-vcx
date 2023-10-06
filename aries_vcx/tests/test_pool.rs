#![allow(clippy::diverging_sub_expression)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

pub mod utils;
use std::{thread, time::Duration};

use aries_vcx::{
    common::{
        keys::{get_verkey_from_ledger, rotate_verkey},
        ledger::{
            service_didsov::EndpointDidSov,
            transactions::{
                add_attr, add_new_did, clear_attr, get_attr, get_service, write_endorser_did,
                write_endpoint, write_endpoint_legacy,
            },
        },
        primitives::{
            credential_definition::CredentialDef,
            credential_schema::Schema,
            revocation_registry::{generate_rev_reg, RevocationRegistry},
            revocation_registry_delta::RevocationRegistryDelta,
        },
        test_utils::{
            create_and_publish_test_rev_reg, create_and_write_test_cred_def,
            create_and_write_test_schema,
        },
    },
    core::profile::Profile,
    errors::error::AriesVcxErrorKind,
    run_setup,
    utils::{
        constants::{DEFAULT_SCHEMA_ATTRS, TEST_TAILS_URL},
        devsetup::SetupPoolDirectory,
    },
};
use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds,
    ledger::{
        base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite},
        indy::pool::test_utils::get_temp_file_path,
    },
    wallet::{base_wallet::BaseWallet, indy::wallet::get_verkey_from_wallet},
};
use diddoc_legacy::aries::service::AriesService;

use crate::utils::{
    scenarios::attr_names_address_list,
    test_agent::{create_test_agent, create_test_agent_trustee},
};

// TODO: Deduplicate with create_and_store_revocable_credential_def
async fn create_and_store_nonrevocable_credential_def(
    anoncreds: &impl BaseAnonCreds,
    ledger_read: &impl AnoncredsLedgerRead,
    ledger_write: &impl AnoncredsLedgerWrite,
    issuer_did: &str,
    attr_list: &str,
) -> (String, String, String, String, CredentialDef) {
    let schema = create_and_write_test_schema(anoncreds, ledger_write, issuer_did, attr_list).await;
    let cred_def = create_and_write_test_cred_def(
        anoncreds,
        ledger_read,
        ledger_write,
        issuer_did,
        &schema.schema_id,
        false,
    )
    .await;

    tokio::time::sleep(Duration::from_millis(1000)).await;
    let cred_def_id = cred_def.get_cred_def_id();
    let cred_def_json = ledger_read.get_cred_def(&cred_def_id, None).await.unwrap();
    (
        schema.schema_id,
        schema.schema_json,
        cred_def_id,
        cred_def_json,
        cred_def,
    )
}

// TODO: Deduplicate with create_and_store_nonrevocable_credential_def
async fn create_and_store_revocable_credential_def(
    anoncreds: &impl BaseAnonCreds,
    ledger_read: &impl AnoncredsLedgerRead,
    ledger_write: &impl AnoncredsLedgerWrite,
    issuer_did: &str,
    attr_list: &str,
) -> (Schema, CredentialDef, RevocationRegistry) {
    let schema = create_and_write_test_schema(anoncreds, ledger_write, issuer_did, attr_list).await;
    let cred_def = create_and_write_test_cred_def(
        anoncreds,
        ledger_read,
        ledger_write,
        issuer_did,
        &schema.schema_id,
        true,
    )
    .await;
    let rev_reg = create_and_publish_test_rev_reg(
        anoncreds,
        ledger_write,
        issuer_did,
        &cred_def.get_cred_def_id(),
    )
    .await;
    tokio::time::sleep(Duration::from_millis(1000)).await;

    (schema, cred_def, rev_reg)
}

#[tokio::test]
#[ignore]
async fn test_pool_rotate_verkey() {
    run_setup!(|setup| async move {
        let (did, verkey) = add_new_did(
            setup.profile.wallet(),
            setup.profile.ledger_write(),
            &setup.institution_did,
            None,
        )
        .await
        .unwrap();
        rotate_verkey(setup.profile.wallet(), setup.profile.ledger_write(), &did)
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(1000)).await;
        let local_verkey = setup
            .profile
            .wallet()
            .key_for_local_did(&did)
            .await
            .unwrap();

        let ledger_verkey = get_verkey_from_ledger(setup.profile.ledger_read(), &did)
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
    run_setup!(|setup| async move {
        let did = setup.institution_did.clone();
        let expect_service = AriesService::default();
        write_endpoint_legacy(setup.profile.ledger_write(), &did, &expect_service)
            .await
            .unwrap();
        thread::sleep(Duration::from_millis(50));
        let service = get_service(setup.profile.ledger_read(), &did)
            .await
            .unwrap();
        assert_eq!(expect_service, service);

        // clean up written legacy service
        clear_attr(
            setup.profile.ledger_write(),
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
        let acme_vk = get_verkey_from_wallet(
            acme.profile.wallet().get_wallet_handle(),
            &acme.institution_did,
        )
        .await
        .unwrap();

        let attrib_json = json!({ "attrib_name": "foo"}).to_string();
        assert!(add_attr(
            acme.profile.ledger_write(),
            &acme.institution_did,
            &attrib_json
        )
        .await
        .is_err());
        write_endorser_did(
            faber.profile.ledger_write(),
            &faber.institution_did,
            &acme.institution_did,
            &acme_vk,
            None,
        )
        .await
        .unwrap();
        thread::sleep(Duration::from_millis(50));
        add_attr(
            acme.profile.ledger_write(),
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
    run_setup!(|setup| async move {
        let did = setup.institution_did.clone();
        let create_service = EndpointDidSov::create()
            .set_service_endpoint("https://example.org".parse().unwrap())
            .set_routing_keys(Some(vec!["did:sov:456".into()]));
        write_endpoint(setup.profile.ledger_write(), &did, &create_service)
            .await
            .unwrap();
        thread::sleep(Duration::from_millis(50));
        let service = get_service(setup.profile.ledger_read(), &did)
            .await
            .unwrap();
        let expect_recipient_key =
            get_verkey_from_ledger(setup.profile.ledger_read(), &setup.institution_did)
                .await
                .unwrap();
        let expect_service = AriesService::default()
            .set_service_endpoint("https://example.org".parse().unwrap())
            .set_recipient_keys(vec![expect_recipient_key])
            .set_routing_keys(vec!["did:sov:456".into()]);
        assert_eq!(expect_service, service);

        // clean up written endpoint
        clear_attr(
            setup.profile.ledger_write(),
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
    run_setup!(|setup| async move {
        let did = setup.institution_did.clone();
        let create_service = EndpointDidSov::create()
            .set_service_endpoint("https://example.org".parse().unwrap())
            .set_routing_keys(None);
        write_endpoint(setup.profile.ledger_write(), &did, &create_service)
            .await
            .unwrap();
        thread::sleep(Duration::from_millis(50));
        let service = get_service(setup.profile.ledger_read(), &did)
            .await
            .unwrap();
        let expect_recipient_key =
            get_verkey_from_ledger(setup.profile.ledger_read(), &setup.institution_did)
                .await
                .unwrap();
        let expect_service = AriesService::default()
            .set_service_endpoint("https://example.org".parse().unwrap())
            .set_recipient_keys(vec![expect_recipient_key])
            .set_routing_keys(vec![]);
        assert_eq!(expect_service, service);

        // clean up written endpoint
        clear_attr(
            setup.profile.ledger_write(),
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
    run_setup!(|setup| async move {
        let did = setup.institution_did.clone();

        // Write legacy service format
        let service_1 = AriesService::create()
            .set_service_endpoint("https://example1.org".parse().unwrap())
            .set_recipient_keys(vec!["did:sov:123".into()])
            .set_routing_keys(vec!["did:sov:456".into()]);
        write_endpoint_legacy(setup.profile.ledger_write(), &did, &service_1)
            .await
            .unwrap();

        // Get service and verify it is in the old format
        let service = get_service(setup.profile.ledger_read(), &did)
            .await
            .unwrap();
        assert_eq!(service_1, service);

        // Write new service format
        let endpoint_url_2 = "https://example2.org";
        let routing_keys_2 = vec![];
        let service_2 = EndpointDidSov::create()
            .set_service_endpoint(endpoint_url_2.parse().unwrap())
            .set_routing_keys(Some(routing_keys_2.clone()));
        write_endpoint(setup.profile.ledger_write(), &did, &service_2)
            .await
            .unwrap();

        thread::sleep(Duration::from_millis(50));

        // Get service and verify it is in the new format
        let service = get_service(setup.profile.ledger_read(), &did)
            .await
            .unwrap();
        let expect_recipient_key =
            get_verkey_from_ledger(setup.profile.ledger_read(), &setup.institution_did)
                .await
                .unwrap();
        let expect_service = AriesService::default()
            .set_service_endpoint(endpoint_url_2.parse().unwrap())
            .set_recipient_keys(vec![expect_recipient_key])
            .set_routing_keys(routing_keys_2);
        assert_eq!(expect_service, service);

        // Clear up written endpoint
        clear_attr(
            setup.profile.ledger_write(),
            &setup.institution_did,
            "endpoint",
        )
        .await
        .unwrap();

        thread::sleep(Duration::from_millis(50));

        // Get service and verify it is in the old format
        let service = get_service(setup.profile.ledger_read(), &did)
            .await
            .unwrap();
        assert_eq!(service_1, service);
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_pool_add_get_attr() {
    run_setup!(|setup| async move {
        let did = setup.institution_did.clone();
        let attr_json = json!({
            "attr_json": {
                "attr_key_1": "attr_value_1",
                "attr_key_2": "attr_value_2",
            }
        });
        add_attr(setup.profile.ledger_write(), &did, &attr_json.to_string())
            .await
            .unwrap();
        thread::sleep(Duration::from_millis(50));
        let attr = get_attr(setup.profile.ledger_read(), &did, "attr_json")
            .await
            .unwrap();
        assert_eq!(attr, attr_json["attr_json"].to_string());

        clear_attr(setup.profile.ledger_write(), &did, "attr_json")
            .await
            .unwrap();
        thread::sleep(Duration::from_millis(50));
        let attr = get_attr(setup.profile.ledger_read(), &did, "attr_json")
            .await
            .unwrap();
        assert_eq!(attr, "");

        let attr = get_attr(setup.profile.ledger_read(), &did, "nonexistent")
            .await
            .unwrap();
        assert_eq!(attr, "");
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_get_credential_def() {
    run_setup!(|setup| async move {
        let (_, _, cred_def_id, cred_def_json, _) = create_and_store_nonrevocable_credential_def(
            setup.profile.anoncreds(),
            setup.profile.ledger_read(),
            setup.profile.ledger_write(),
            &setup.institution_did,
            DEFAULT_SCHEMA_ATTRS,
        )
        .await;

        let ledger = setup.profile.ledger_read();
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
    run_setup!(|setup| async move {
        // Cred def is created with support_revocation=false,
        // revoc_reg_def will fail in libindy because cred_Def doesn't have revocation keys
        let (_, _, cred_def_id, _, _) = create_and_store_nonrevocable_credential_def(
            setup.profile.anoncreds(),
            setup.profile.ledger_read(),
            setup.profile.ledger_write(),
            &setup.institution_did,
            DEFAULT_SCHEMA_ATTRS,
        )
        .await;

        let rc = generate_rev_reg(
            setup.profile.anoncreds(),
            &setup.institution_did,
            &cred_def_id,
            get_temp_file_path("path.txt").to_str().unwrap(),
            2,
            "tag1",
        )
        .await;

        #[cfg(feature = "credx")]
        assert_eq!(rc.unwrap_err().kind(), AriesVcxErrorKind::InvalidState);
        #[cfg(not(feature = "credx"))]
        assert_eq!(rc.unwrap_err().kind(), AriesVcxErrorKind::InvalidInput);
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_pool_get_rev_reg_def_json() {
    run_setup!(|setup| async move {
        let attrs = format!("{:?}", attr_names_address_list());
        let (_, _, rev_reg) = create_and_store_revocable_credential_def(
            setup.profile.anoncreds(),
            setup.profile.ledger_read(),
            setup.profile.ledger_write(),
            &setup.institution_did,
            &attrs,
        )
        .await;

        let ledger = setup.profile.ledger_read();
        let _json = ledger
            .get_rev_reg_def_json(&rev_reg.rev_reg_id)
            .await
            .unwrap();
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_pool_get_rev_reg_delta_json() {
    run_setup!(|setup| async move {
        let attrs = format!("{:?}", attr_names_address_list());
        let (_, _, rev_reg) = create_and_store_revocable_credential_def(
            setup.profile.anoncreds(),
            setup.profile.ledger_read(),
            setup.profile.ledger_write(),
            &setup.institution_did,
            &attrs,
        )
        .await;

        let ledger = setup.profile.ledger_read();
        let (id, _delta, _timestamp) = ledger
            .get_rev_reg_delta_json(&rev_reg.rev_reg_id, None, None)
            .await
            .unwrap();

        assert_eq!(id, rev_reg.rev_reg_id);
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_pool_get_rev_reg() {
    run_setup!(|setup| async move {
        let attrs = format!("{:?}", attr_names_address_list());
        let (_, _, rev_reg) = create_and_store_revocable_credential_def(
            setup.profile.anoncreds(),
            setup.profile.ledger_read(),
            setup.profile.ledger_write(),
            &setup.institution_did,
            &attrs,
        )
        .await;
        assert_eq!(
            TEST_TAILS_URL,
            rev_reg.get_rev_reg_def().value.tails_location
        );

        let ledger = setup.profile.ledger_read();
        let (id, _rev_reg, _timestamp) = ledger
            .get_rev_reg(
                &rev_reg.rev_reg_id,
                time::OffsetDateTime::now_utc().unix_timestamp() as u64,
            )
            .await
            .unwrap();

        assert_eq!(id, rev_reg.rev_reg_id);
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_pool_create_and_get_schema() {
    run_setup!(|setup| async move {
        let schema = create_and_write_test_schema(
            setup.profile.anoncreds(),
            setup.profile.ledger_write(),
            &setup.institution_did,
            DEFAULT_SCHEMA_ATTRS,
        )
        .await;

        let ledger = setup.profile.ledger_read();
        let rc = ledger.get_schema(&schema.schema_id, None).await;

        let retrieved_schema = rc.unwrap();
        assert!(retrieved_schema.contains(&schema.schema_id));
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_pool_create_rev_reg_delta_from_ledger() {
    run_setup!(|setup| async move {
        let attrs = format!("{:?}", attr_names_address_list());
        let (_, _, rev_reg) = create_and_store_revocable_credential_def(
            setup.profile.anoncreds(),
            setup.profile.ledger_read(),
            setup.profile.ledger_write(),
            &setup.institution_did,
            &attrs,
        )
        .await;

        let (_, rev_reg_delta_json, _) = setup
            .profile
            .ledger_read()
            .get_rev_reg_delta_json(&rev_reg.rev_reg_id, None, None)
            .await
            .unwrap();
        assert!(
            RevocationRegistryDelta::create_from_ledger(&rev_reg_delta_json)
                .await
                .is_ok()
        );
    })
    .await;
}
