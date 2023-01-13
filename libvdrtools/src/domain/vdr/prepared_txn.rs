use super::super::crypto::CryptoTypes;
use super::ledger_types::DidMethod;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct SignatureSpec {
    pub signature_type: CryptoTypes,
    pub ledger_type: DidMethod,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum EndorsementSpec {
    Indy(IndyEndorsementSpec),
    Cheqd(CheqdEndorsementSpec),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct IndyEndorsementSpec {
    pub endorser_did: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct CheqdEndorsementSpec {}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct IndyEndorsementData {
    pub did: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct CheqdEndorsementData {
    pub txn_author_did: String,
    pub key_alias: String,
    pub chain_id: String,
    pub account_number: u64,
    pub sequence_number: u64,
    pub max_gas: u64,
    pub max_coin_amount: u64,
    pub max_coin_denom: String,
    pub timeout_height: u64,
    pub memo: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum Endorsement {
    Indy(IndyEndorsement),
    Cheqd(CheqdEndorsement),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct IndyEndorsement {
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct CheqdEndorsement {
    pub chain_id: String,
    pub txn_author_did: String,
    pub public_key: String,
    pub account_id: String,
    pub account_number: u64,
    pub sequence_number: u64,
    pub max_gas: u64,
    pub max_coin_amount: u64,
    pub max_coin_denom: String,
    pub timeout_height: u64,
    pub memo: String,
    pub signature: String,
}
