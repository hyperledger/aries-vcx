use std::sync::Arc;

use aries_vcx::{
    aries_vcx_core::{
        anoncreds::credx_anoncreds::IndyCredxAnonCreds,
        ledger::base_ledger::TxnAuthrAgrmtOptions,
        wallet::indy::{wallet::create_and_open_wallet, IndySdkWallet, WalletConfig},
    },
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    utils::mockdata::profile::mock_ledger::MockLedger,
};

use crate::{errors::error::VcxUniFFIResult, runtime::block_on};

#[derive(Debug)]
pub struct UniffiProfile {
    wallet: IndySdkWallet,
    anoncreds: IndyCredxAnonCreds,
    ledger_read: MockLedger,
    ledger_write: MockLedger,
}

impl UniffiProfile {
    pub fn ledger_read(&self) -> &MockLedger {
        &self.ledger_read
    }

    pub fn ledger_write(&self) -> &MockLedger {
        &self.ledger_write
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

pub fn new_indy_profile(wallet_config: WalletConfig) -> VcxUniFFIResult<Arc<ProfileHolder>> {
    block_on(async {
        let wh = create_and_open_wallet(&wallet_config).await?;

        let wallet = IndySdkWallet::new(wh);
        let profile = UniffiProfile {
            anoncreds: IndyCredxAnonCreds,
            wallet,
            ledger_read: MockLedger,
            ledger_write: MockLedger,
        };

        Ok(Arc::new(ProfileHolder { inner: profile }))
    })
}
