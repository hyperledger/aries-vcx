pub mod scenarios;
pub mod test_agent;
use std::{path::Path, time::Duration};

use anoncreds_types::data_types::identifiers::{
    cred_def_id::CredentialDefinitionId, schema_id::SchemaId,
};
use aries_vcx::{
    common::{
        credentials::encoding::encode_attributes,
        primitives::{
            credential_definition::CredentialDef, credential_schema::Schema,
            revocation_registry::RevocationRegistry,
        },
    },
    global::settings,
};
use aries_vcx_anoncreds::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_ledger::ledger::{
    base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite},
    indy::pool::test_utils::get_temp_dir_path,
};
use aries_vcx_wallet::wallet::base_wallet::BaseWallet;
use did_parser_nom::Did;
use test_utils::{
    constants::TEST_TAILS_URL,
    random::{generate_random_schema_name, generate_random_schema_version},
};

pub async fn create_and_write_test_schema(
    wallet: &impl BaseWallet,
    anoncreds: &impl BaseAnonCreds,
    ledger_write: &impl AnoncredsLedgerWrite,
    submitter_did: &Did,
    attr_list: &str,
) -> Schema {
    let schema = Schema::create(
        anoncreds,
        "source_id",
        submitter_did,
        &generate_random_schema_name(),
        &generate_random_schema_version(),
        serde_json::from_str::<Vec<String>>(attr_list).unwrap(),
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
    issuer_did: &Did,
    schema_id: &SchemaId,
    revokable: bool,
) -> CredentialDef {
    CredentialDef::create(
        wallet,
        ledger_read,
        anoncreds,
        "1".to_string(),
        issuer_did.clone(),
        schema_id.clone(),
        "1".to_string(),
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
    issuer_did: &Did,
    cred_def_id: &CredentialDefinitionId,
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

#[allow(clippy::too_many_arguments)] // test util code
pub async fn create_and_write_credential(
    wallet_issuer: &impl BaseWallet,
    wallet_holder: &impl BaseWallet,
    anoncreds_issuer: &impl BaseAnonCreds,
    anoncreds_holder: &impl BaseAnonCreds,
    institution_did: &Did,
    schema: &Schema,
    cred_def: &CredentialDef,
    rev_reg: Option<&RevocationRegistry>,
) -> String {
    // TODO: Inject credential_data from caller
    let credential_data = r#"{"address1": ["123 Main St"], "address2": ["Suite 3"], "city": ["Draper"], "state": ["UT"], "zip": ["84000"]}"#;
    let encoded_attributes = encode_attributes(credential_data).unwrap();

    let offer = anoncreds_issuer
        .issuer_create_credential_offer(wallet_issuer, cred_def.get_cred_def_id())
        .await
        .unwrap();
    let (req, req_meta) = anoncreds_holder
        .prover_create_credential_req(
            wallet_holder,
            institution_did,
            serde_json::from_str(&serde_json::to_string(&offer).unwrap()).unwrap(),
            cred_def.get_cred_def_json().try_clone().unwrap(),
            &settings::DEFAULT_LINK_SECRET_ALIAS.to_string(),
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
    let (cred, _) = anoncreds_issuer
        .issuer_create_credential(
            wallet_issuer,
            offer,
            req,
            serde_json::from_str(&encoded_attributes).unwrap(),
            rev_reg_id
                .map(TryInto::try_into)
                .transpose()
                .unwrap()
                .as_ref(),
            tails_dir.as_deref().map(Path::new),
        )
        .await
        .unwrap();

    anoncreds_holder
        .prover_store_credential(
            wallet_holder,
            req_meta,
            cred,
            schema.schema_json.clone(),
            cred_def.get_cred_def_json().try_clone().unwrap(),
            rev_reg_def_json
                .as_deref()
                .map(serde_json::from_str)
                .transpose()
                .unwrap(),
        )
        .await
        .unwrap()
}
