use std::{collections::HashMap, sync::Arc};

use time::get_time;

use super::primitives::revocation_registry_delta::RevocationRegistryDelta;
use crate::{
    core::profile::profile::Profile,
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
};

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

pub async fn get_cred_rev_id(profile: &Arc<dyn Profile>, cred_id: &str) -> VcxResult<String> {
    let anoncreds = Arc::clone(profile).inject_anoncreds();
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

pub async fn is_cred_revoked(profile: &Arc<dyn Profile>, rev_reg_id: &str, rev_id: &str) -> VcxResult<bool> {
    let from = None;
    let to = Some(get_time().sec as u64 + 100);
    let rev_reg_delta = RevocationRegistryDelta::create_from_ledger(profile, rev_reg_id, from, to).await?;
    Ok(rev_reg_delta.revoked().iter().any(|s| s.to_string().eq(rev_id)))
}

#[cfg(test)]
#[cfg(feature = "pool_tests")]
mod integration_tests {
    use super::*;
    use crate::{
        common::test_utils::create_and_store_credential,
        utils::{
            constants::DEFAULT_SCHEMA_ATTRS,
            devsetup::{init_holder_setup_in_indy_context, SetupProfile},
        },
    };

    #[tokio::test]
    async fn test_prover_get_credential() {
        SetupProfile::run_indy(|setup| async move {
            let holder_setup = init_holder_setup_in_indy_context(&setup).await;

            let res = create_and_store_credential(
                &setup.profile,
                &holder_setup.profile,
                &setup.institution_did,
                DEFAULT_SCHEMA_ATTRS,
            )
            .await;
            let schema_id = res.0;
            let cred_def_id = res.2;
            let cred_id = res.7;
            let rev_reg_id = res.8;
            let cred_rev_id = res.9;

            let anoncreds = Arc::clone(&holder_setup.profile).inject_anoncreds();

            let cred_json = anoncreds.prover_get_credential(&cred_id).await.unwrap();
            let prover_cred = serde_json::from_str::<ProverCredential>(&cred_json).unwrap();

            assert_eq!(prover_cred.schema_id, schema_id);
            assert_eq!(prover_cred.cred_def_id, cred_def_id);
            assert_eq!(prover_cred.cred_rev_id.unwrap().to_string(), cred_rev_id);
            assert_eq!(prover_cred.rev_reg_id.unwrap(), rev_reg_id);
        })
        .await;
    }

    #[tokio::test]
    async fn test_get_cred_rev_id() {
        SetupProfile::run_indy(|setup| async move {
            let holder_setup = init_holder_setup_in_indy_context(&setup).await;

            let res = create_and_store_credential(
                &setup.profile,
                &holder_setup.profile,
                &setup.institution_did,
                DEFAULT_SCHEMA_ATTRS,
            )
            .await;
            let cred_id = res.7;
            let cred_rev_id = res.9;

            let cred_rev_id_ = get_cred_rev_id(&holder_setup.profile, &cred_id).await.unwrap();

            assert_eq!(cred_rev_id, cred_rev_id_.to_string());
        })
        .await;
    }

    #[tokio::test]
    async fn test_is_cred_revoked() {
        SetupProfile::run_indy(|setup| async move {
            let holder_setup = init_holder_setup_in_indy_context(&setup).await;

            let res = create_and_store_credential(
                &setup.profile,
                &holder_setup.profile,
                &setup.institution_did,
                DEFAULT_SCHEMA_ATTRS,
            )
            .await;
            let rev_reg_id = res.8;
            let cred_rev_id = res.9;
            let tails_file = res.10;

            assert!(!is_cred_revoked(&holder_setup.profile, &rev_reg_id, &cred_rev_id)
                .await
                .unwrap());

            let anoncreds = Arc::clone(&setup.profile).inject_anoncreds();

            anoncreds
                .revoke_credential_local(&tails_file, &rev_reg_id, &cred_rev_id)
                .await
                .unwrap();
            anoncreds
                .publish_local_revocations(&setup.institution_did, &rev_reg_id)
                .await
                .unwrap();

            std::thread::sleep(std::time::Duration::from_millis(500));

            assert!(is_cred_revoked(&holder_setup.profile, &rev_reg_id, &cred_rev_id)
                .await
                .unwrap());
        })
        .await;
    }
}
