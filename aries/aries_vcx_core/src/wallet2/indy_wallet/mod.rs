use super::BaseWallet2;
use crate::wallet::indy::IndySdkWallet;

pub mod indy_did_wallet;
pub mod indy_record_wallet;

const WALLET_OPTIONS: &str =
    r#"{"retrieveType": true, "retrieveValue": true, "retrieveTags": true}"#;

const SEARCH_OPTIONS: &str = r#"{"retrieveType": true, "retrieveValue": true, "retrieveTags": true, "retrieveRecords": true}"#;

impl BaseWallet2 for IndySdkWallet {}
