use std::error::Error;

use anoncreds_types::{
    data_types::ledger::cred_def::CredentialDefinition, utils::validation::Validatable,
};
use aries_vcx::common::primitives::{
    credential_definition::generate_cred_def, revocation_registry::generate_rev_reg,
};
use aries_vcx_ledger::ledger::{
    base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite},
    indy::pool::test_utils::get_temp_dir_path,
};
use test_utils::{constants::DEFAULT_SCHEMA_ATTRS, devsetup::build_setup_profile};

use crate::utils::create_and_write_test_schema;

pub mod utils;

#[tokio::test]
#[ignore]
async fn test_pool_create_cred_def_real() -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let schema = create_and_write_test_schema(
        &setup.wallet,
        &setup.anoncreds,
        &setup.ledger_write,
        &setup.institution_did,
        DEFAULT_SCHEMA_ATTRS,
    )
    .await;

    let ledger_read = setup.ledger_read;
    let ledger_write = &setup.ledger_write;
    let schema_json = ledger_read.get_schema(&schema.schema_id, None).await?;

    let cred_def = generate_cred_def(
        &setup.wallet,
        &setup.anoncreds,
        &setup.institution_did,
        &schema.schema_id,
        schema_json,
        "tag_1",
        None,
        Some(true),
    )
    .await?;

    ledger_write
        .publish_cred_def(&setup.wallet, cred_def.try_clone()?, &setup.institution_did)
        .await?;

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let cred_def_json_ledger = ledger_read
        .get_cred_def(&cred_def.id, Some(&setup.institution_did))
        .await?;
    cred_def_json_ledger.validate()?;

    // same as the original generated cred def, but schema ID corrected to the qualified version
    let cred_def_corrected_schema_id = CredentialDefinition {
        schema_id: schema.schema_id,
        ..cred_def.try_clone().unwrap()
    };

    // check cred def matches originally, but with corected schema ID.
    assert_eq!(
        serde_json::to_value(cred_def_json_ledger)?,
        serde_json::to_value(cred_def_corrected_schema_id)?
    );
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_pool_create_rev_reg_def() -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let schema = create_and_write_test_schema(
        &setup.wallet,
        &setup.anoncreds,
        &setup.ledger_write,
        &setup.institution_did,
        DEFAULT_SCHEMA_ATTRS,
    )
    .await;
    let ledger_read = &setup.ledger_read;
    let ledger_write = &setup.ledger_write;
    let schema_json = ledger_read.get_schema(&schema.schema_id, None).await?;

    let cred_def = generate_cred_def(
        &setup.wallet,
        &setup.anoncreds,
        &setup.institution_did,
        &schema.schema_id,
        schema_json,
        "tag_1",
        None,
        Some(true),
    )
    .await?;
    ledger_write
        .publish_cred_def(&setup.wallet, cred_def.try_clone()?, &setup.institution_did)
        .await?;

    let path = get_temp_dir_path();

    let (rev_reg_def_id, rev_reg_def_json, rev_reg_entry_json) = generate_rev_reg(
        &setup.wallet,
        &setup.anoncreds,
        &setup.institution_did,
        &cred_def.id,
        path.to_str().unwrap(),
        2,
        "tag1",
    )
    .await?;
    ledger_write
        .publish_rev_reg_def(
            &setup.wallet,
            serde_json::from_str(&serde_json::to_string(&rev_reg_def_json)?)?,
            &setup.institution_did,
        )
        .await?;
    ledger_write
        .publish_rev_reg_delta(
            &setup.wallet,
            &rev_reg_def_id.try_into()?,
            serde_json::from_str(&rev_reg_entry_json)?,
            &setup.institution_did,
        )
        .await?;
    Ok(())
}
