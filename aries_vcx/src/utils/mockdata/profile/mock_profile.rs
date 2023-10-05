use aries_vcx_core::{ledger::base_ledger::TxnAuthrAgrmtOptions, wallet::mock_wallet::MockWallet};
use async_trait::async_trait;

use super::{mock_anoncreds::MockAnoncreds, mock_ledger::MockLedger};
use crate::{core::profile::Profile, errors::error::VcxResult};

/// Implementation of a [Profile] which uses [MockLedger], [MockAnoncreds] and [MockWallet] to
/// return mock data for all Profile methods. Only for unit testing purposes
#[derive(Debug)]
pub struct MockProfile;

#[async_trait]
impl Profile for MockProfile {
    type LedgerRead = MockLedger;
    type LedgerWrite = MockLedger;
    type Anoncreds = MockAnoncreds;
    type Wallet = MockWallet;

    fn ledger_read(&self) -> &Self::LedgerRead {
        &MockLedger
    }

    fn ledger_write(&self) -> &Self::LedgerWrite {
        &MockLedger
    }

    fn anoncreds(&self) -> &Self::Anoncreds {
        &MockAnoncreds
    }

    fn wallet(&self) -> &Self::Wallet {
        &MockWallet
    }

    fn update_taa_configuration(&self, _taa_options: TxnAuthrAgrmtOptions) -> VcxResult<()> {
        error!("update_taa_configuration not implemented for MockProfile");
        Ok(())
    }
}
