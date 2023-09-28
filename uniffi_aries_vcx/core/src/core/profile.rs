use std::sync::Arc;

use aries_vcx::{
    aries_vcx_core::{
        anoncreds::indy_anoncreds::IndySdkAnonCreds,
        ledger::base_ledger::TxnAuthrAgrmtOptions,
        wallet::indy::{wallet::create_and_open_wallet, IndySdkWallet, WalletConfig},
    },
    core::profile::profile::Profile,
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    utils::mockdata::profile::mock_ledger::MockLedger,
};
use async_trait::async_trait;

use crate::{errors::error::VcxUniFFIResult, runtime::block_on};

#[derive(Debug)]
pub struct UniffiProfile {
    wallet: IndySdkWallet,
    anoncreds: IndySdkAnonCreds,
    ledger_read: MockLedger,
    ledger_write: MockLedger,
}

#[async_trait]
impl Profile for UniffiProfile {
    type LedgerRead = MockLedger;
    type LedgerWrite = MockLedger;
    type Anoncreds = IndySdkAnonCreds;
    type Wallet = IndySdkWallet;

    fn ledger_read(&self) -> &Self::LedgerRead {
        &self.ledger_read
    }

    fn ledger_write(&self) -> &Self::LedgerWrite {
        &self.ledger_write
    }

    fn anoncreds(&self) -> &Self::Anoncreds {
        &self.anoncreds
    }

    fn wallet(&self) -> &Self::Wallet {
        &self.wallet
    }

    fn update_taa_configuration(&self, _taa_options: TxnAuthrAgrmtOptions) -> VcxResult<()> {
        Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::ActionNotSupported,
            "update_taa_configuration no implemented for VdrtoolsProfile",
        ))
    }
}

pub struct ProfileHolder {
    pub(crate) inner: UniffiProfile,
}

pub fn new_indy_profile(wallet_config: WalletConfig) -> VcxUniFFIResult<Arc<ProfileHolder>> {
    block_on(async {
        let wh = create_and_open_wallet(&wallet_config).await?;

        let wallet = IndySdkWallet::new(wh);
        let profile = UniffiProfile {
            wallet,
            anoncreds: IndySdkAnonCreds::new(wh),
            ledger_read: MockLedger,
            ledger_write: MockLedger,
        };

        Ok(Arc::new(ProfileHolder { inner: profile }))
    })
}
