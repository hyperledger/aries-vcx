#![allow(clippy::diverging_sub_expression)]

use aries_vcx::global::settings::DEFAULT_LINK_SECRET_ALIAS;
use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds,
    ledger::base_ledger::{
        AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite,
    },
    wallet::{base_wallet::BaseWallet, indy::IndySdkWallet},
};
use test_utils::{
    constants::TRUSTEE_SEED,
    devsetup::{dev_build_featured_components, dev_setup_wallet_indy},
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
    pub institution_did: String,
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
    let (institution_did, wallet_handle) = dev_setup_wallet_indy(seed).await;
    let wallet = IndySdkWallet::new(wallet_handle);
    let (ledger_read, ledger_write, anoncreds) =
        dev_build_featured_components(genesis_file_path.clone()).await;

    anoncreds
        .prover_create_link_secret(&wallet, DEFAULT_LINK_SECRET_ALIAS)
        .await
        .unwrap();

    TestAgent {
        genesis_file_path,
        institution_did,
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
