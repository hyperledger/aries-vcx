use std::error::Error;

use aries_vcx::common::credentials::{get_cred_rev_id, is_cred_revoked};
use aries_vcx_anoncreds::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_ledger::ledger::base_ledger::AnoncredsLedgerRead;
use test_utils::{constants::DEFAULT_SCHEMA_ATTRS, devsetup::build_setup_profile};

use crate::utils::{
    create_and_publish_test_rev_reg, create_and_write_credential, create_and_write_test_cred_def,
    create_and_write_test_schema,
};

pub mod utils;

#[tokio::test]
#[ignore]
async fn test_pool_prover_get_credential() -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let schema = create_and_write_test_schema(
        &setup.wallet,
        &setup.anoncreds,
        &setup.ledger_write,
        &setup.institution_did,
        DEFAULT_SCHEMA_ATTRS,
    )
    .await;
    let cred_def = create_and_write_test_cred_def(
        &setup.wallet,
        &setup.anoncreds,
        &setup.ledger_read,
        &setup.ledger_write,
        &setup.institution_did,
        &schema.schema_id,
        true,
    )
    .await;
    let rev_reg = create_and_publish_test_rev_reg(
        &setup.wallet,
        &setup.anoncreds,
        &setup.ledger_write,
        &setup.institution_did,
        cred_def.get_cred_def_id(),
    )
    .await;
    let cred_id = create_and_write_credential(
        &setup.wallet,
        &setup.wallet,
        &setup.anoncreds,
        &setup.anoncreds,
        &setup.institution_did,
        &schema,
        &cred_def,
        Some(&rev_reg),
    )
    .await;
    let cred_rev_id = get_cred_rev_id(&setup.wallet, &setup.anoncreds, &cred_id).await?;

    let prover_cred = setup
        .anoncreds
        .prover_get_credential(&setup.wallet, &cred_id)
        .await?;

    assert_eq!(prover_cred.schema_id, schema.schema_id);
    assert_eq!(&prover_cred.cred_def_id, cred_def.get_cred_def_id());
    assert_eq!(prover_cred.cred_rev_id.unwrap(), cred_rev_id);
    assert_eq!(prover_cred.rev_reg_id.unwrap(), rev_reg.rev_reg_id);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_pool_is_cred_revoked() -> Result<(), Box<dyn Error>> {
    let setup = build_setup_profile().await;
    let schema = create_and_write_test_schema(
        &setup.wallet,
        &setup.anoncreds,
        &setup.ledger_write,
        &setup.institution_did,
        DEFAULT_SCHEMA_ATTRS,
    )
    .await;
    let cred_def = create_and_write_test_cred_def(
        &setup.wallet,
        &setup.anoncreds,
        &setup.ledger_read,
        &setup.ledger_write,
        &setup.institution_did,
        &schema.schema_id,
        true,
    )
    .await;
    let rev_reg = create_and_publish_test_rev_reg(
        &setup.wallet,
        &setup.anoncreds,
        &setup.ledger_write,
        &setup.institution_did,
        cred_def.get_cred_def_id(),
    )
    .await;
    let cred_id = create_and_write_credential(
        &setup.wallet,
        &setup.wallet,
        &setup.anoncreds,
        &setup.anoncreds,
        &setup.institution_did,
        &schema,
        &cred_def,
        Some(&rev_reg),
    )
    .await;
    let cred_rev_id = get_cred_rev_id(&setup.wallet, &setup.anoncreds, &cred_id).await?;

    assert!(!is_cred_revoked(&setup.ledger_read, &rev_reg.rev_reg_id, cred_rev_id).await?);

    #[allow(deprecated)] // TODO - https://github.com/hyperledger/aries-vcx/issues/1309
    let rev_reg_delta_json = setup
        .ledger_read
        .get_rev_reg_delta_json(&rev_reg.rev_reg_id.to_owned().try_into()?, None, None)
        .await?
        .0;
    setup
        .anoncreds
        .revoke_credential_local(
            &setup.wallet,
            &rev_reg.rev_reg_id.to_owned().try_into()?,
            cred_rev_id,
            rev_reg_delta_json,
        )
        .await?;
    rev_reg
        .publish_local_revocations(
            &setup.wallet,
            &setup.anoncreds,
            &setup.ledger_write,
            &setup.institution_did,
        )
        .await?;

    std::thread::sleep(std::time::Duration::from_millis(500));

    assert!(is_cred_revoked(&setup.ledger_read, &rev_reg.rev_reg_id, cred_rev_id).await?);
    Ok(())
}
