use regex::Regex;

use crate::error::prelude::*;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Did(String);

impl Did {
    pub fn new(did: &str) -> VcxResult<Self> {
        if Self::validate(did) {
            Ok(Self(did.to_string()))
        } else {
            Err(VcxError::from_msg(VcxErrorKind::InvalidDid, format!("{} is not a valid DID", did)))
        }
    }

    fn validate(did: &str) -> bool {
        lazy_static! {
            static ref REGEX_METHOD_NAME: Regex = Regex::new("^(did:sov:)?[1-9A-HJ-NP-Za-km-z]{21,22}$").unwrap();
        }
        REGEX_METHOD_NAME.is_match(did)
    }
}

impl std::string::ToString for Did {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

// TODO: THIS IS DANGEROUS! Done because we use builder method on structs
// where Did is used. We need to avoid requiring default for structs which
// implement the builder pattern.
impl Default for Did {
    fn default() -> Self {
        Self("2wJPyULfLLnYTEFYzByfUR".to_string())
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use super::*;

    #[test]
    fn test_did_validation() {
        assert!(Did::new("2hoqvcwupRTUNkXn6ArYzs").is_ok());
        assert!(Did::new("did:sov:2hoqvcwupRTUNkXn6ArYzs").is_ok());
        assert!(Did::new("df6Y3iUa6t").is_err());
    }
}
