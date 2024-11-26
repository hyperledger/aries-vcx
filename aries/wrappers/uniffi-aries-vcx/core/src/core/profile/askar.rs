use std::sync::Arc;

use aries_vcx::{
    aries_vcx_anoncreds::anoncreds::base_anoncreds::BaseAnonCreds,
    aries_vcx_wallet::wallet::{
        askar::{askar_wallet_config::AskarWalletConfig, AskarWallet},
        base_wallet::ManageWallet,
    },
};
use aries_vcx_anoncreds::anoncreds::anoncreds::Anoncreds;
use aries_vcx_ledger::ledger::{
    indy_vdr_ledger::{indyvdr_build_ledger_read, IndyVdrLedgerRead},
    request_submitter::vdr_ledger::{IndyVdrLedgerPool, IndyVdrSubmitter},
    response_cacher::in_memory::{InMemoryResponseCacher, InMemoryResponseCacherConfig},
};
use indy_vdr::config::PoolConfig;

use crate::{
    core::logging::enable_logging, errors::error::VcxUniFFIResult, runtime::block_on, ProfileHolder,
};

#[derive(Debug)]
pub struct UniffiProfile {
    pub wallet: AskarWallet,
    pub anoncreds: Anoncreds,
    pub ledger_read: IndyVdrLedgerRead<IndyVdrSubmitter, InMemoryResponseCacher>,
}

pub fn new_indy_profile(
    wallet_config: AskarWalletConfig,
    genesis_file_path: String,
) -> VcxUniFFIResult<Arc<ProfileHolder>> {
    // Enable android logging
    enable_logging();

    block_on(async {
        let wallet = wallet_config.create_wallet().await?;

        let anoncreds = Anoncreds;

        anoncreds
            .prover_create_link_secret(&wallet, &"main".to_string())
            .await
            .ok();

        let indy_vdr_config = PoolConfig::default();
        let cache_config = InMemoryResponseCacherConfig::builder()
            .ttl(std::time::Duration::from_secs(60))
            .capacity(1000)?
            .build();
        let ledger_pool = IndyVdrLedgerPool::new(genesis_file_path, indy_vdr_config, vec![])?;
        let request_submitter = IndyVdrSubmitter::new(ledger_pool);
        let ledger_read = indyvdr_build_ledger_read(request_submitter, cache_config)?;
        let profile = UniffiProfile {
            anoncreds: Anoncreds,
            wallet,
            ledger_read,
        };

        Ok(Arc::new(ProfileHolder { inner: profile }))
    })
}
