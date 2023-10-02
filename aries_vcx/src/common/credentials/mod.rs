use std::collections::HashMap;

use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds, ledger::base_ledger::AnoncredsLedgerRead,
};
use time::OffsetDateTime;

use super::primitives::revocation_registry_delta::RevocationRegistryDelta;
use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

pub mod encoding;

#[derive(Serialize, Deserialize)]
struct ProverCredential {
    referent: String,
    attrs: HashMap<String, String>,
    schema_id: String,
    cred_def_id: String,
    rev_reg_id: Option<String>,
    cred_rev_id: Option<String>,
}

pub async fn get_cred_rev_id(anoncreds: &impl BaseAnonCreds, cred_id: &str) -> VcxResult<String> {
    let cred_json = anoncreds.prover_get_credential(cred_id).await?;
    let prover_cred = serde_json::from_str::<ProverCredential>(&cred_json).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!("Failed to deserialize anoncreds credential: {}", err),
        )
    })?;
    prover_cred.cred_rev_id.ok_or(AriesVcxError::from_msg(
        AriesVcxErrorKind::InvalidRevocationDetails,
        "Credenial revocation id missing on credential - is this credential revokable?",
    ))
}

pub async fn is_cred_revoked(
    ledger: &impl AnoncredsLedgerRead,
    rev_reg_id: &str,
    rev_id: &str,
) -> VcxResult<bool> {
    let to = Some(OffsetDateTime::now_utc().unix_timestamp() as u64 + 100);
    let (_, rev_reg_delta_json, _) = ledger.get_rev_reg_delta_json(rev_reg_id, None, to).await?;
    let rev_reg_delta = RevocationRegistryDelta::create_from_ledger(&rev_reg_delta_json).await?;
    Ok(rev_reg_delta
        .revoked()
        .iter()
        .any(|s| s.to_string().eq(rev_id)))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod integration_tests {
    use super::*;
    use crate::{
        common::test_utils::{
            create_and_write_credential, create_and_write_test_cred_def,
            create_and_write_test_rev_reg, create_and_write_test_schema,
        },
        utils::devsetup::SetupProfile,
    };

    #[tokio::test]
    #[ignore]
    async fn test_pool_prover_get_credential() {
        run_setup!(|setup| async move {
            let schema = create_and_write_test_schema(
                setup.profile.anoncreds(),
                setup.profile.ledger_write(),
                &setup.institution_did,
                crate::utils::constants::DEFAULT_SCHEMA_ATTRS,
            )
            .await;
            let cred_def = create_and_write_test_cred_def(
                setup.profile.anoncreds(),
                setup.profile.ledger_read(),
                setup.profile.ledger_write(),
                &setup.institution_did,
                &schema.schema_id,
                true,
            )
            .await;
            let rev_reg = create_and_write_test_rev_reg(
                setup.profile.anoncreds(),
                setup.profile.ledger_write(),
                &setup.institution_did,
                &cred_def.get_cred_def_id(),
            )
            .await;
            let cred_id = create_and_write_credential(
                setup.profile.anoncreds(),
                setup.profile.anoncreds(),
                &setup.institution_did,
                &cred_def,
                Some(&rev_reg),
            )
            .await;
            let cred_rev_id = get_cred_rev_id(setup.profile.anoncreds(), &cred_id)
                .await
                .unwrap();

            let cred_json = setup
                .profile
                .anoncreds()
                .prover_get_credential(&cred_id)
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
                setup.profile.anoncreds(),
                setup.profile.ledger_write(),
                &setup.institution_did,
                crate::utils::constants::DEFAULT_SCHEMA_ATTRS,
            )
            .await;
            let cred_def = create_and_write_test_cred_def(
                setup.profile.anoncreds(),
                setup.profile.ledger_read(),
                setup.profile.ledger_write(),
                &setup.institution_did,
                &schema.schema_id,
                true,
            )
            .await;
            let rev_reg = create_and_write_test_rev_reg(
                setup.profile.anoncreds(),
                setup.profile.ledger_write(),
                &setup.institution_did,
                &cred_def.get_cred_def_id(),
            )
            .await;
            let cred_id = create_and_write_credential(
                setup.profile.anoncreds(),
                setup.profile.anoncreds(),
                &setup.institution_did,
                &cred_def,
                Some(&rev_reg),
            )
            .await;
            let cred_rev_id = get_cred_rev_id(setup.profile.anoncreds(), &cred_id)
                .await
                .unwrap();

            assert!(!is_cred_revoked(
                setup.profile.ledger_read(),
                &rev_reg.rev_reg_id,
                &cred_rev_id
            )
            .await
            .unwrap());

            setup
                .profile
                .anoncreds()
                .revoke_credential_local(
                    &rev_reg.get_tails_dir(),
                    &rev_reg.rev_reg_id,
                    &cred_rev_id,
                )
                .await
                .unwrap();
            rev_reg
                .publish_local_revocations(
                    setup.profile.anoncreds(),
                    setup.profile.ledger_write(),
                    &setup.institution_did,
                )
                .await
                .unwrap();

            std::thread::sleep(std::time::Duration::from_millis(500));

            assert!(is_cred_revoked(
                setup.profile.ledger_read(),
                &rev_reg.rev_reg_id,
                &cred_rev_id
            )
            .await
            .unwrap());
        })
        .await;
    }
}
