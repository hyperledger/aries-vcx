use super::super::crypto::{
    ED25519,
    SECP256K1,
};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum LedgerTypes {
    Indy,
    Cheqd,
}

impl LedgerTypes {
    pub(crate) fn signature_type(&self) -> &'static str {
        match self {
            LedgerTypes::Indy => ED25519,
            LedgerTypes::Cheqd => SECP256K1,
        }
    }
}