pub mod utils;

use aries_vcx::common::primitives::{
    credential_definition::generate_cred_def, revocation_registry::generate_rev_reg,
};
use aries_vcx_core::ledger::{
    base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite},
    indy::pool::test_utils::get_temp_dir_path,
};
use serde_json::json;
use test_utils::{constants::DEFAULT_SCHEMA_ATTRS, run_setup_test};

use crate::utils::create_and_write_test_schema;

#[tokio::test]
#[ignore]
async fn test_pool_create_cred_def_real() {
    run_setup_test!(|setup| async move {
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
        let schema_json = ledger_read
            .get_schema(&schema.schema_id, None)
            .await
            .unwrap();

        let (cred_def_id, cred_def_json_local) = generate_cred_def(
            &setup.wallet,
            &setup.anoncreds,
            &setup.institution_did,
            &schema_json,
            "tag_1",
            None,
            Some(true),
        )
        .await
        .unwrap();

        ledger_write
            .publish_cred_def(&setup.wallet, &cred_def_json_local, &setup.institution_did)
            .await
            .unwrap();

        std::thread::sleep(std::time::Duration::from_secs(2));

        let cred_def_json_ledger = ledger_read
            .get_cred_def(&cred_def_id, Some(&setup.institution_did))
            .await
            .unwrap();

        assert!(cred_def_json_local.contains(&cred_def_id));
        assert!(cred_def_json_ledger.contains(&cred_def_id));
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_pool_create_rev_reg_def() {
    run_setup_test!(|setup| async move {
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
        let schema_json = ledger_read
            .get_schema(&schema.schema_id, None)
            .await
            .unwrap();

        let (cred_def_id, cred_def_json) = generate_cred_def(
            &setup.wallet,
            &setup.anoncreds,
            &setup.institution_did,
            &schema_json,
            "tag_1",
            None,
            Some(true),
        )
        .await
        .unwrap();
        ledger_write
            .publish_cred_def(&setup.wallet, &cred_def_json, &setup.institution_did)
            .await
            .unwrap();

        let path = get_temp_dir_path();

        let (rev_reg_def_id, rev_reg_def_json, rev_reg_entry_json) = generate_rev_reg(
            &setup.wallet,
            &setup.anoncreds,
            &setup.institution_did,
            &cred_def_id,
            path.to_str().unwrap(),
            2,
            "tag1",
        )
        .await
        .unwrap();
        ledger_write
            .publish_rev_reg_def(
                &setup.wallet,
                &json!(rev_reg_def_json).to_string(),
                &setup.institution_did,
            )
            .await
            .unwrap();
        ledger_write
            .publish_rev_reg_delta(
                &setup.wallet,
                &rev_reg_def_id,
                &rev_reg_entry_json,
                &setup.institution_did,
            )
            .await
            .unwrap();
    })
    .await;
}
