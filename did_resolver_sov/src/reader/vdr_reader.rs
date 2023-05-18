use std::sync::Arc;

use crate::error::DidSovError;
use aries_vcx_core::{
    ledger::{
        indy_vdr_ledger::{IndyVdrLedger, IndyVdrLedgerConfig},
        request_signer::base_wallet::BaseWalletRequestSigner,
        request_submitter::vdr_ledger::{IndyVdrLedgerPool, IndyVdrSubmitter, LedgerPoolConfig},
    },
    wallet::{base_wallet::BaseWallet, indy_wallet::IndySdkWallet},
    ResponseParser, INVALID_WALLET_HANDLE,
};

use super::ConcreteAttrReader;

impl TryFrom<LedgerPoolConfig> for ConcreteAttrReader {
    type Error = DidSovError;

    fn try_from(pool_config: LedgerPoolConfig) -> Result<Self, Self::Error> {
        let wallet = Arc::new(IndySdkWallet::new(INVALID_WALLET_HANDLE)) as Arc<dyn BaseWallet>;
        let ledger_pool = Arc::new(IndyVdrLedgerPool::new(pool_config)?);
        let request_submitter = Arc::new(IndyVdrSubmitter::new(ledger_pool));
        let request_signer = Arc::new(BaseWalletRequestSigner::new(wallet.clone()));
        let response_parser = Arc::new(ResponseParser::new());
        let config = IndyVdrLedgerConfig {
            request_signer,
            request_submitter,
            response_parser,
        };
        let ledger = Arc::new(IndyVdrLedger::new(config));
        Ok(Self { ledger })
    }
}
