#![allow(clippy::diverging_sub_expression)]

use aries_vcx::{
    common::ledger::transactions::write_endorser_did, global::settings::DEFAULT_LINK_SECRET_ALIAS,
};
use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds,
    ledger::base_ledger::{
        AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite,
    },
    wallet::base_wallet::{BaseWallet, DidWallet},
};
use did_parser::Did;
use test_utils::{
    constants::TRUSTEE_SEED,
    devsetup::{
        dev_build_featured_anoncreds, dev_build_featured_indy_ledger, dev_build_featured_wallet,
    },
    random::generate_random_seed,
};

pub struct TestAgent<LR, LW, A, W>
where
    LR: IndyLedgerRead + AnoncredsLedgerRead,
    LW: IndyLedgerWrite + AnoncredsLedgerWrite,
    A: BaseAnonCreds,
    W: BaseWallet,
{
    pub ledger_read: LR,
    pub ledger_write: LW,
    pub anoncreds: A,
    pub wallet: W,
    pub institution_did: Did,
    pub genesis_file_path: String,
}

async fn create_test_agent_from_seed(
    seed: &str,
    genesis_file_path: String,
) -> TestAgent<
    impl IndyLedgerRead + AnoncredsLedgerRead,
    impl IndyLedgerWrite + AnoncredsLedgerWrite,
    impl BaseAnonCreds,
    impl BaseWallet,
> {
    let (institution_did, wallet) = dev_build_featured_wallet(seed).await;
    let (ledger_read, ledger_write) =
        dev_build_featured_indy_ledger(genesis_file_path.clone()).await;
    let anoncreds = dev_build_featured_anoncreds().await;

    anoncreds
        .prover_create_link_secret(&wallet, DEFAULT_LINK_SECRET_ALIAS)
        .await
        .unwrap();

    TestAgent {
        genesis_file_path,
        institution_did: Did::parse(institution_did).unwrap(),
        wallet,
        ledger_read,
        ledger_write,
        anoncreds,
    }
}

pub async fn create_test_agent_trustee(
    genesis_file_path: String,
) -> TestAgent<
    impl IndyLedgerRead + AnoncredsLedgerRead,
    impl IndyLedgerWrite + AnoncredsLedgerWrite,
    impl BaseAnonCreds,
    impl BaseWallet,
> {
    create_test_agent_from_seed(TRUSTEE_SEED, genesis_file_path).await
}

pub async fn create_test_agent(
    genesis_file_path: String,
) -> TestAgent<
    impl IndyLedgerRead + AnoncredsLedgerRead,
    impl IndyLedgerWrite + AnoncredsLedgerWrite,
    impl BaseAnonCreds,
    impl BaseWallet,
> {
    create_test_agent_from_seed(&generate_random_seed(), genesis_file_path).await
}

pub async fn create_test_agent_endorser<LW, W>(
    ledger_write: LW,
    trustee_wallet: W,
    genesis_file_path: &str,
    trustee_did: &Did,
) -> Result<
    TestAgent<
        impl IndyLedgerRead + AnoncredsLedgerRead,
        impl IndyLedgerWrite + AnoncredsLedgerWrite,
        impl BaseAnonCreds,
        impl BaseWallet,
    >,
    Box<dyn std::error::Error>,
>
where
    LW: IndyLedgerWrite + AnoncredsLedgerWrite,
    W: BaseWallet,
{
    let acme = create_test_agent(genesis_file_path.to_string()).await;
    let acme_vk = acme.wallet.key_for_did(&acme.institution_did.to_string()).await?;

    write_endorser_did(
        &trustee_wallet,
        &ledger_write,
        trustee_did,
        &acme.institution_did,
        &acme_vk.base58(),
        None,
    )
    .await?;

    Ok(acme)
}
