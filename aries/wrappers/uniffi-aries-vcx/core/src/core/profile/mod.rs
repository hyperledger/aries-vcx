use aries_vcx::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};
use aries_vcx_anoncreds::anoncreds::credx_anoncreds::IndyCredxAnonCreds;
use aries_vcx_ledger::ledger::{
    base_ledger::TxnAuthrAgrmtOptions, indy_vdr_ledger::IndyVdrLedgerRead,
    request_submitter::vdr_ledger::IndyVdrSubmitter,
    response_cacher::in_memory::InMemoryResponseCacher,
};

#[cfg(feature = "vdrtools_wallet")]
pub mod indy;
#[cfg(feature = "vdrtools_wallet")]
use aries_vcx::aries_vcx_wallet::wallet::indy::IndySdkWallet;
#[cfg(feature = "vdrtools_wallet")]
pub use indy as profile;

#[cfg(feature = "askar_wallet")]
pub mod askar;
#[cfg(feature = "askar_wallet")]
use aries_vcx::aries_vcx_wallet::wallet::askar::AskarWallet;
#[cfg(feature = "askar_wallet")]
pub use askar as profile;

use crate::profile::UniffiProfile;

impl UniffiProfile {
    pub fn ledger_read(&self) -> &IndyVdrLedgerRead<IndyVdrSubmitter, InMemoryResponseCacher> {
        &self.ledger_read
    }

    pub fn anoncreds(&self) -> &IndyCredxAnonCreds {
        &self.anoncreds
    }

    #[cfg(feature = "vdrtools_wallet")]
    pub fn wallet(&self) -> &IndySdkWallet {
        &self.wallet
    }

    #[cfg(feature = "askar_wallet")]
    pub fn wallet(&self) -> &AskarWallet {
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
