#[cfg(feature = "ffi_api")]
use super::super::crypto::{ED25519, SECP256K1};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum DidMethod {
    Indy,
    Cheqd,
}

impl ToString for DidMethod {
    fn to_string(&self) -> String {
        match self {
            DidMethod::Indy => "indy".to_owned(),
            DidMethod::Cheqd => "cheqd".to_owned(),
        }
    }
}

#[cfg(feature = "ffi_api")]
impl DidMethod {
    pub(crate) fn signature_type(&self) -> &'static str {
        match self {
            DidMethod::Indy => ED25519,
            DidMethod::Cheqd => SECP256K1,
        }
    }
}
