#![allow(clippy::diverging_sub_expression)]

use std::{error::Error, thread, time::Duration};

use anoncreds_types::data_types::identifiers::cred_def_id::CredentialDefinitionId;
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
        },
    },
    errors::error::AriesVcxErrorKind,
};
use aries_vcx_anoncreds::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_ledger::ledger::{
    base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerWrite},
    indy::pool::test_utils::get_temp_file_path,
};
use aries_vcx_wallet::wallet::base_wallet::{did_wallet::DidWallet, BaseWallet};
use did_parser_nom::Did;
use diddoc_legacy::aries::service::AriesService;
use serde_json::json;
use test_utils::{
    constants::{DEFAULT_SCHEMA_ATTRS, TEST_TAILS_URL},
    devsetup::{build_setup_profile, SetupPoolDirectory},
};

use crate::utils::{
    create_and_publish_test_rev_reg, create_and_write_test_cred_def, create_and_write_test_schema,
    scenarios::attr_names_address_list,
    test_agent::{create_test_agent, create_test_agent_endorser, create_test_agent_trustee},
};

pub mod utils;

// TODO: Deduplicate with create_and_store_revocable_credential_def
async fn create_and_store_nonrevocable_credential_def(
    wallet: &impl BaseWallet,
    anoncreds: &impl BaseAnonCreds,
    ledger_read: &impl AnoncredsLedgerRead,
    ledger_write: &impl AnoncredsLedgerWrite,
    issuer_did: &Did,
    attr_list: &str,
) -> Result<
    (
        String,
        String,
        CredentialDefinitionId,
        String,
        CredentialDef,
    ),
    Box<dyn Error>,
> {
    let schema =
        create_and_write_test_schema(wallet, anoncreds, ledger_write, issuer_did, attr_list).await;
    let cred_def = create_and_write_test_cred_def(
        wallet,
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
    let cred_def_json = ledger_read
        .get_cred_def(&cred_def_id.to_owned(), None)
        .await?;
    Ok((
        schema.schema_id.to_string(),
        serde_json::to_string(&schema.schema_json)?,
        cred_def_id.to_owned(),
        serde_json::to_string(&cred_def_json)?,
        cred_def,
    ))
}

// TODO: Deduplicate with create_and_store_nonrevocable_credential_def
async fn create_and_store_revocable_credential_def(
    wallet: &impl BaseWallet,
    anoncreds: &impl BaseAnonCreds,
    ledger_read: &impl AnoncredsLedgerRead,
    ledger_write: &impl AnoncredsLedgerWrite,
    issuer_did: &Did,
    attr_list: &str,
) -> Result<(Schema, CredentialDef, RevocationRegistry), Box<dyn Error>> {
    let schema =
        create_and_write_test_schema(wallet, anoncreds, ledger_write, issuer_did, attr_list).await;
    let cred_def = create_and_write_test_cred_def(
        wallet,
        anoncreds,
        ledger_read,
        ledger_write,
        issuer_did,
        &schema.schema_id,
        true,
    )
    .await;
    let rev_reg = create_and_publish_test_rev_reg(
        wallet,
        anoncreds,
        ledger_write,
        issuer_did,
        cred_def.get_cred_def_id(),
    )
    .await;
    tokio::time::sleep(Duration::from_millis(1000)).await;
    Ok((schema, cred_def, rev_reg))
}

#[tokio::test]
#[ignore]
async fn test_pool_rotate_verkey() -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let (did, verkey) = add_new_did(
        &setup.wallet,
        &setup.ledger_write,
        &setup.institution_did,
        None,
    )
    .await?;
    rotate_verkey(&setup.wallet, &setup.ledger_write, &did).await?;
    tokio::time::sleep(Duration::from_millis(1000)).await;
    let local_verkey = setup.wallet.key_for_did(&did.to_string()).await?;

    let ledger_verkey = get_verkey_from_ledger(&setup.ledger_read, &did).await?;
    assert_ne!(verkey, ledger_verkey);
    assert_eq!(local_verkey.base58(), ledger_verkey);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_pool_add_get_service() -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let endorser = create_test_agent_endorser(
        setup.ledger_write,
        setup.wallet,
        &setup.genesis_file_path,
        &setup.institution_did,
    )
    .await?;

    let expect_service = AriesService::default();
    write_endpoint_legacy(
        &endorser.wallet,
        &endorser.ledger_write,
        &endorser.institution_did,
        &expect_service,
    )
    .await?;
    thread::sleep(Duration::from_millis(50));
    let service = get_service(&endorser.ledger_read, &endorser.institution_did).await?;
    assert_eq!(expect_service, service);

    // clean up written legacy service
    clear_attr(
        &endorser.wallet,
        &endorser.ledger_write,
        &endorser.institution_did,
        "service",
    )
    .await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_pool_write_new_endorser_did() -> Result<(), Box<dyn Error>> {
    let setup = SetupPoolDirectory::init().await;
    let faber = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
    let acme = create_test_agent(setup.genesis_file_path.clone()).await;
    let acme_vk = acme
        .wallet
        .key_for_did(&acme.institution_did.to_string())
        .await?;

    let attrib_json = json!({ "attrib_name": "foo"}).to_string();
    assert!(add_attr(
        &acme.wallet,
        &acme.ledger_write,
        &acme.institution_did,
        &attrib_json,
    )
    .await
    .is_err());
    write_endorser_did(
        &faber.wallet,
        &faber.ledger_write,
        &faber.institution_did,
        &acme.institution_did,
        &acme_vk.base58(),
        None,
    )
    .await?;
    thread::sleep(Duration::from_millis(50));
    add_attr(
        &acme.wallet,
        &acme.ledger_write,
        &acme.institution_did,
        &attrib_json,
    )
    .await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_pool_add_get_service_public() -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let endorser = create_test_agent_endorser(
        setup.ledger_write,
        setup.wallet,
        &setup.genesis_file_path,
        &setup.institution_did,
    )
    .await?;

    let create_service = EndpointDidSov::create()
        .set_service_endpoint("https://example.org".parse()?)
        .set_routing_keys(Some(vec!["did:sov:456".into()]));
    write_endpoint(
        &endorser.wallet,
        &endorser.ledger_write,
        &endorser.institution_did,
        &create_service,
    )
    .await?;
    thread::sleep(Duration::from_millis(50));
    let service = get_service(&endorser.ledger_read, &endorser.institution_did).await?;
    let expect_recipient_key =
        get_verkey_from_ledger(&endorser.ledger_read, &endorser.institution_did).await?;
    let expect_service = AriesService::default()
        .set_service_endpoint("https://example.org".parse()?)
        .set_recipient_keys(vec![expect_recipient_key])
        .set_routing_keys(vec!["did:sov:456".into()]);
    assert_eq!(expect_service, service);

    // clean up written endpoint
    clear_attr(
        &endorser.wallet,
        &endorser.ledger_write,
        &endorser.institution_did,
        "endpoint",
    )
    .await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_pool_add_get_service_public_none_routing_keys() -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let did = setup.institution_did.clone();
    let create_service = EndpointDidSov::create()
        .set_service_endpoint("https://example.org".parse()?)
        .set_routing_keys(None);
    write_endpoint(&setup.wallet, &setup.ledger_write, &did, &create_service).await?;
    thread::sleep(Duration::from_millis(50));
    let service = get_service(&setup.ledger_read, &did).await?;
    let expect_recipient_key =
        get_verkey_from_ledger(&setup.ledger_read, &setup.institution_did).await?;
    let expect_service = AriesService::default()
        .set_service_endpoint("https://example.org".parse()?)
        .set_recipient_keys(vec![expect_recipient_key])
        .set_routing_keys(vec![]);
    assert_eq!(expect_service, service);

    // clean up written endpoint
    clear_attr(
        &setup.wallet,
        &setup.ledger_write,
        &setup.institution_did,
        "endpoint",
    )
    .await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_pool_multiple_service_formats() -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let did = setup.institution_did.clone();

    // clear all
    let c = json!({ "service": serde_json::Value::Null }).to_string();
    setup.ledger_write.add_attr(&setup.wallet, &did, &c).await?;
    let c = json!({ "endpoint": serde_json::Value::Null }).to_string();
    setup.ledger_write.add_attr(&setup.wallet, &did, &c).await?;

    // Write legacy service format
    let service_1 = AriesService::create()
        .set_service_endpoint("https://example1.org".parse()?)
        .set_recipient_keys(vec!["did:sov:123".into()])
        .set_routing_keys(vec!["did:sov:456".into()]);
    write_endpoint_legacy(&setup.wallet, &setup.ledger_write, &did, &service_1).await?;
    thread::sleep(Duration::from_millis(50));

    // Get service and verify it is in the old format
    let service = get_service(&setup.ledger_read, &did).await?;
    assert_eq!(service_1, service);

    // Write new service format
    let endpoint_url_2 = "https://example2.org";
    let routing_keys_2 = vec![];
    let service_2 = EndpointDidSov::create()
        .set_service_endpoint(endpoint_url_2.parse()?)
        .set_routing_keys(Some(routing_keys_2.clone()));
    write_endpoint(&setup.wallet, &setup.ledger_write, &did, &service_2).await?;
    thread::sleep(Duration::from_millis(50));

    // Get service and verify it is in the new format
    let service = get_service(&setup.ledger_read, &did).await?;
    let expect_recipient_key =
        get_verkey_from_ledger(&setup.ledger_read, &setup.institution_did).await?;
    let expect_service = AriesService::default()
        .set_service_endpoint(endpoint_url_2.parse()?)
        .set_recipient_keys(vec![expect_recipient_key])
        .set_routing_keys(routing_keys_2);
    assert_eq!(expect_service, service);

    // Clear up written endpoint
    clear_attr(
        &setup.wallet,
        &setup.ledger_write,
        &setup.institution_did,
        "endpoint",
    )
    .await?;

    thread::sleep(Duration::from_millis(50));

    // Get service and verify it is in the old format
    let service = get_service(&setup.ledger_read, &did).await?;
    assert_eq!(service_1, service);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_pool_add_get_attr() -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let did = setup.institution_did.clone();
    let attr_json = json!({
        "attr_json": {
            "attr_key_1": "attr_value_1",
            "attr_key_2": "attr_value_2",
        }
    });
    add_attr(
        &setup.wallet,
        &setup.ledger_write,
        &did,
        &attr_json.to_string(),
    )
    .await?;
    thread::sleep(Duration::from_millis(50));
    let attr = get_attr(&setup.ledger_read, &did, "attr_json").await?;
    assert_eq!(attr, attr_json["attr_json"].to_string());

    clear_attr(&setup.wallet, &setup.ledger_write, &did, "attr_json").await?;
    thread::sleep(Duration::from_millis(50));
    let attr = get_attr(&setup.ledger_read, &did, "attr_json").await?;
    assert_eq!(attr, "");

    let attr = get_attr(&setup.ledger_read, &did, "nonexistent").await?;
    assert_eq!(attr, "");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_get_credential_def() -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let (_, _, cred_def_id, cred_def_json, _) = create_and_store_nonrevocable_credential_def(
        &setup.wallet,
        &setup.anoncreds,
        &setup.ledger_read,
        &setup.ledger_write,
        &setup.institution_did,
        DEFAULT_SCHEMA_ATTRS,
    )
    .await?;

    let ledger = &setup.ledger_read;
    let r_cred_def_json = ledger.get_cred_def(&cred_def_id, None).await?;

    let def1: serde_json::Value = serde_json::from_str(&cred_def_json)?;
    let def2 = serde_json::to_value(&r_cred_def_json)?;
    assert_eq!(def1, def2);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_pool_rev_reg_def_fails_for_cred_def_created_without_revocation(
) -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    // Cred def is created with support_revocation=false,
    // revoc_reg_def will fail in libindy because cred_Def doesn't have revocation keys
    let (_, _, cred_def_id, _, _) = create_and_store_nonrevocable_credential_def(
        &setup.wallet,
        &setup.anoncreds,
        &setup.ledger_read,
        &setup.ledger_write,
        &setup.institution_did,
        DEFAULT_SCHEMA_ATTRS,
    )
    .await?;

    let rc = generate_rev_reg(
        &setup.wallet,
        &setup.anoncreds,
        &setup.institution_did,
        &cred_def_id,
        get_temp_file_path("path.txt").to_str().unwrap(),
        2,
        "tag1",
    )
    .await;

    assert_eq!(rc.unwrap_err().kind(), AriesVcxErrorKind::InvalidState);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_pool_get_rev_reg_def_json() -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let attrs = format!("{:?}", attr_names_address_list());
    let (_, _, rev_reg) = create_and_store_revocable_credential_def(
        &setup.wallet,
        &setup.anoncreds,
        &setup.ledger_read,
        &setup.ledger_write,
        &setup.institution_did,
        &attrs,
    )
    .await?;

    let ledger = &setup.ledger_read;
    let _json = ledger
        .get_rev_reg_def_json(&rev_reg.rev_reg_id.try_into()?)
        .await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_pool_get_rev_reg_delta_json() -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let attrs = format!("{:?}", attr_names_address_list());
    let (_, _, rev_reg) = create_and_store_revocable_credential_def(
        &setup.wallet,
        &setup.anoncreds,
        &setup.ledger_read,
        &setup.ledger_write,
        &setup.institution_did,
        &attrs,
    )
    .await?;

    let ledger = &setup.ledger_read;
    #[allow(deprecated)] // TODO - https://github.com/hyperledger/aries-vcx/issues/1309
    let (_delta, _timestamp) = ledger
        .get_rev_reg_delta_json(&rev_reg.rev_reg_id.to_owned().try_into()?, None, None)
        .await?;

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_pool_get_rev_reg() -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let attrs = format!("{:?}", attr_names_address_list());
    let (_, _, rev_reg) = create_and_store_revocable_credential_def(
        &setup.wallet,
        &setup.anoncreds,
        &setup.ledger_read,
        &setup.ledger_write,
        &setup.institution_did,
        &attrs,
    )
    .await?;
    assert_eq!(
        TEST_TAILS_URL,
        rev_reg.get_rev_reg_def().value.tails_location
    );

    let ledger = &setup.ledger_read;
    let (_rev_reg, _timestamp) = ledger
        .get_rev_reg(
            &rev_reg.rev_reg_id.to_owned().try_into()?,
            time::OffsetDateTime::now_utc().unix_timestamp() as u64,
        )
        .await?;

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_pool_create_and_get_schema() -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let schema = create_and_write_test_schema(
        &setup.wallet,
        &setup.anoncreds,
        &setup.ledger_write,
        &setup.institution_did,
        DEFAULT_SCHEMA_ATTRS,
    )
    .await;

    let ledger = &setup.ledger_read;
    let retrieved_schema =
        serde_json::to_string(&ledger.get_schema(&schema.schema_id, None).await?).unwrap();
    assert!(retrieved_schema.contains(&schema.schema_id.to_string()));
    Ok(())
}
