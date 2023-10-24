#![allow(clippy::unwrap_used)]

use std::time::Duration;

use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds,
    ledger::{
        base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite},
        indy::pool::test_utils::get_temp_dir_path,
    },
    wallet::base_wallet::BaseWallet,
};

use super::primitives::credential_schema::Schema;
use crate::{
    common::{
        credentials::encoding::encode_attributes,
        primitives::{
            credential_definition::{CredentialDef, CredentialDefConfigBuilder},
            revocation_registry::RevocationRegistry,
        },
    },
    global::settings,
    utils::{
        constants::TEST_TAILS_URL,
        random::{generate_random_schema_name, generate_random_schema_version},
    },
};

pub async fn create_and_write_test_schema(
    wallet: &impl BaseWallet,
    anoncreds: &impl BaseAnonCreds,
    ledger_write: &impl AnoncredsLedgerWrite,
    submitter_did: &str,
    attr_list: &str,
) -> Schema {
    let schema = Schema::create(
        anoncreds,
        "source_id",
        submitter_did,
        &generate_random_schema_name(),
        &generate_random_schema_version(),
        &serde_json::from_str::<Vec<String>>(attr_list).unwrap(),
    )
    .await
    .unwrap();
    let schema = schema.publish(wallet, ledger_write).await.unwrap();
    std::thread::sleep(Duration::from_millis(500));
    schema
}

pub async fn create_and_write_test_cred_def(
    wallet: &impl BaseWallet,
    anoncreds: &impl BaseAnonCreds,
    ledger_read: &impl AnoncredsLedgerRead,
    ledger_write: &impl AnoncredsLedgerWrite,
    issuer_did: &str,
    schema_id: &str,
    revokable: bool,
) -> CredentialDef {
    CredentialDef::create(
        wallet,
        ledger_read,
        anoncreds,
        "1".to_string(),
        CredentialDefConfigBuilder::default()
            .issuer_did(issuer_did)
            .schema_id(schema_id)
            .tag("1")
            .build()
            .unwrap(),
        revokable,
    )
    .await
    .unwrap()
    .publish_cred_def(wallet, ledger_read, ledger_write)
    .await
    .unwrap()
}

pub async fn create_and_publish_test_rev_reg(
    wallet: &impl BaseWallet,
    anoncreds: &impl BaseAnonCreds,
    ledger_write: &impl AnoncredsLedgerWrite,
    issuer_did: &str,
    cred_def_id: &str,
) -> RevocationRegistry {
    let tails_dir = get_temp_dir_path().as_path().to_str().unwrap().to_string();
    let mut rev_reg = RevocationRegistry::create(
        wallet,
        anoncreds,
        issuer_did,
        cred_def_id,
        &tails_dir,
        10,
        1,
    )
    .await
    .unwrap();
    rev_reg
        .publish_revocation_primitives(wallet, ledger_write, TEST_TAILS_URL)
        .await
        .unwrap();
    rev_reg
}

pub async fn create_and_write_credential(
    wallet_issuer: &impl BaseWallet,
    wallet_holder: &impl BaseWallet,
    anoncreds_issuer: &impl BaseAnonCreds,
    anoncreds_holder: &impl BaseAnonCreds,
    institution_did: &str,
    cred_def: &CredentialDef,
    rev_reg: Option<&RevocationRegistry>,
) -> String {
    // TODO: Inject credential_data from caller
    let credential_data = r#"{"address1": ["123 Main St"], "address2": ["Suite 3"], "city": ["Draper"], "state": ["UT"], "zip": ["84000"]}"#;
    let encoded_attributes = encode_attributes(credential_data).unwrap();

    let offer = anoncreds_issuer
        .issuer_create_credential_offer(wallet_issuer, &cred_def.get_cred_def_id())
        .await
        .unwrap();
    let (req, req_meta) = anoncreds_holder
        .prover_create_credential_req(
            wallet_holder,
            institution_did,
            &offer,
            cred_def.get_cred_def_json(),
            settings::DEFAULT_LINK_SECRET_ALIAS,
        )
        .await
        .unwrap();

    let (rev_reg_def_json, rev_reg_id, tails_dir) = if let Some(rev_reg) = rev_reg {
        (
            Some(serde_json::to_string(&rev_reg.get_rev_reg_def()).unwrap()),
            Some(rev_reg.rev_reg_id.clone()),
            Some(rev_reg.tails_dir.clone()),
        )
    } else {
        (None, None, None)
    };
    println!("rev_reg_def_json: {:?}", rev_reg_def_json);
    let (cred, _, _) = anoncreds_issuer
        .issuer_create_credential(
            wallet_issuer,
            &offer,
            &req,
            &encoded_attributes,
            rev_reg_id,
            tails_dir,
        )
        .await
        .unwrap();

    anoncreds_holder
        .prover_store_credential(
            wallet_holder,
            None,
            &req_meta,
            &cred,
            cred_def.get_cred_def_json(),
            rev_reg_def_json.as_deref(),
        )
        .await
        .unwrap()
}
