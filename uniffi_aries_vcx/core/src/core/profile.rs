use std::sync::Arc;

use aries_vcx::{
    aries_vcx_core::{
        anoncreds::credx_anoncreds::IndyCredxAnonCreds,
        ledger::{
            base_ledger::TxnAuthrAgrmtOptions,
            indy_vdr_ledger::{indyvdr_build_ledger_read, IndyVdrLedgerRead},
            request_submitter::vdr_ledger::{IndyVdrLedgerPool, IndyVdrSubmitter},
            response_cacher::in_memory::{InMemoryResponseCacher, InMemoryResponseCacherConfig},
        },
        wallet::indy::{wallet::create_and_open_wallet, IndySdkWallet, WalletConfig},
        PoolConfig,
    },
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
};

use crate::{errors::error::VcxUniFFIResult, runtime::block_on};

#[derive(Debug)]
pub struct UniffiProfile {
    wallet: IndySdkWallet,
    anoncreds: IndyCredxAnonCreds,
    ledger_read: IndyVdrLedgerRead<IndyVdrSubmitter, InMemoryResponseCacher>,
}

impl UniffiProfile {
    pub fn ledger_read(&self) -> &IndyVdrLedgerRead<IndyVdrSubmitter, InMemoryResponseCacher> {
        &self.ledger_read
    }

    pub fn anoncreds(&self) -> &IndyCredxAnonCreds {
        &self.anoncreds
    }

    pub fn wallet(&self) -> &IndySdkWallet {
        &self.wallet
    }

    pub fn update_taa_configuration(&self, _taa_options: TxnAuthrAgrmtOptions) -> VcxResult<()> {
        Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::ActionNotSupported,
            "update_taa_configuration no implemented for VdrtoolsProfile",
        ))
    }
}

pub struct ProfileHolder {
    pub(crate) inner: UniffiProfile,
}

pub fn new_indy_profile(
    wallet_config: WalletConfig,
    genesis_file_path: String,
) -> VcxUniFFIResult<Arc<ProfileHolder>> {
    block_on(async {
        let wh = create_and_open_wallet(&wallet_config).await?;
        let indy_vdr_config = PoolConfig::default();
        let cache_config = InMemoryResponseCacherConfig::builder()
            .ttl(std::time::Duration::from_secs(60))
            .capacity(1000)?
            .build();
        let ledger_pool = IndyVdrLedgerPool::new(genesis_file_path, indy_vdr_config, vec![])?;
        let request_submitter = IndyVdrSubmitter::new(ledger_pool);
        let ledger_read = indyvdr_build_ledger_read(request_submitter.clone(), cache_config)?;
        let wallet = IndySdkWallet::new(wh);
        let profile = UniffiProfile {
            anoncreds: IndyCredxAnonCreds,
            wallet,
            ledger_read,
        };

        Ok(Arc::new(ProfileHolder { inner: profile }))
    })
}
