use super::*;
use crate::common::test_utils::{
    create_and_publish_test_rev_reg, create_and_write_credential, create_and_write_test_cred_def,
    create_and_write_test_schema,
};

#[tokio::test]
#[ignore]
async fn test_pool_prover_get_credential() {
    run_setup!(|setup| async move {
        let schema = create_and_write_test_schema(
            &setup.wallet,
            &setup.anoncreds,
            &setup.ledger_write,
            &setup.institution_did,
            crate::utils::constants::DEFAULT_SCHEMA_ATTRS,
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
            &cred_def.get_cred_def_id(),
        )
        .await;
        let cred_id = create_and_write_credential(
            &setup.wallet,
            &setup.wallet,
            &setup.anoncreds,
            &setup.anoncreds,
            &setup.institution_did,
            &cred_def,
            Some(&rev_reg),
        )
        .await;
        let cred_rev_id = get_cred_rev_id(&setup.wallet, &setup.anoncreds, &cred_id)
            .await
            .unwrap();

        let cred_json = setup
            .anoncreds
            .prover_get_credential(&setup.wallet, &cred_id)
            .await
            .unwrap();
        let prover_cred = serde_json::from_str::<ProverCredential>(&cred_json).unwrap();

        assert_eq!(prover_cred.schema_id, schema.schema_id);
        assert_eq!(prover_cred.cred_def_id, cred_def.get_cred_def_id());
        assert_eq!(prover_cred.cred_rev_id.unwrap(), cred_rev_id);
        assert_eq!(prover_cred.rev_reg_id.unwrap(), rev_reg.rev_reg_id);
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_pool_is_cred_revoked() {
    run_setup!(|setup| async move {
        let schema = create_and_write_test_schema(
            &setup.wallet,
            &setup.anoncreds,
            &setup.ledger_write,
            &setup.institution_did,
            crate::utils::constants::DEFAULT_SCHEMA_ATTRS,
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
            &cred_def.get_cred_def_id(),
        )
        .await;
        let cred_id = create_and_write_credential(
            &setup.wallet,
            &setup.wallet,
            &setup.anoncreds,
            &setup.anoncreds,
            &setup.institution_did,
            &cred_def,
            Some(&rev_reg),
        )
        .await;
        let cred_rev_id = get_cred_rev_id(&setup.wallet, &setup.anoncreds, &cred_id)
            .await
            .unwrap();

        assert!(
            !is_cred_revoked(&setup.ledger_read, &rev_reg.rev_reg_id, &cred_rev_id)
                .await
                .unwrap()
        );

        setup
            .anoncreds
            .revoke_credential_local(
                &setup.wallet,
                &rev_reg.get_tails_dir(),
                &rev_reg.rev_reg_id,
                &cred_rev_id,
            )
            .await
            .unwrap();
        rev_reg
            .publish_local_revocations(
                &setup.wallet,
                &setup.anoncreds,
                &setup.ledger_write,
                &setup.institution_did,
            )
            .await
            .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(500));

        assert!(
            is_cred_revoked(&setup.ledger_read, &rev_reg.rev_reg_id, &cred_rev_id)
                .await
                .unwrap()
        );
    })
    .await;
}
