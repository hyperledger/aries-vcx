use std::sync::Arc;

use aries_vcx_core::{ledger::base_ledger::IndyLedgerRead, wallet::base_wallet::BaseWallet};
use did_parser::Did;
use messages::msg_fields::protocols::out_of_band::invitation::Invitation as OobInvitation;
use url::Url;

pub struct PairwiseConstructRequestConfig {
    pub ledger: Arc<dyn IndyLedgerRead>,
    pub wallet: Arc<dyn BaseWallet>,
    pub invitation: OobInvitation,
    pub service_endpoint: Url,
    pub routing_keys: Vec<String>,
}

pub struct PublicConstructRequestConfig {
    pub ledger: Arc<dyn IndyLedgerRead>,
    pub wallet: Arc<dyn BaseWallet>,
    pub their_did: Did,
    pub our_did: Did,
}

pub enum ConstructRequestConfig {
    Pairwise(PairwiseConstructRequestConfig),
    Public(PublicConstructRequestConfig),
}
