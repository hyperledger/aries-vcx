use crate::{
    error::Error,
    utils::validation::{Validatable, LEGACY_DID_IDENTIFIER, URI_IDENTIFIER},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, Default)]
pub struct IssuerId(pub String);

impl IssuerId {
    pub fn new_unchecked(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn new(s: impl Into<String>) -> Result<Self, Error> {
        let s = Self(s.into());
        Validatable::validate(&s)?;
        Ok(s)
    }

    pub fn is_legacy(&self) -> bool {
        LEGACY_DID_IDENTIFIER.captures(&self.0).is_some()
    }

    pub fn is_uri(&self) -> bool {
        URI_IDENTIFIER.captures(&self.0).is_some()
    }
}

impl Validatable for IssuerId {
    fn validate(&self) -> Result<(), Error> {
        if crate::utils::validation::URI_IDENTIFIER
            .captures(&self.0)
            .is_some()
        {
            return Ok(());
        }

        if LEGACY_DID_IDENTIFIER.captures(&self.0).is_some() {
            return Ok(());
        }

        Err(crate::Error::from_msg(
            crate::ErrorKind::ConversionError,
            format!(
                "type: {}, identifier: {} is invalid. It MUST be a URI or legacy identifier.",
                "IssuerId", self.0
            ),
        ))
    }
}

impl From<IssuerId> for String {
    fn from(i: IssuerId) -> Self {
        i.0
    }
}

impl TryFrom<String> for IssuerId {
    type Error = Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        IssuerId::new(value)
    }
}

impl TryFrom<&str> for IssuerId {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        IssuerId::new(value.to_owned())
    }
}

impl std::fmt::Display for IssuerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod test_issuer_identifiers {
    use super::*;

    #[test]
    fn should_validate_new_and_legacy_identifiers() {
        let valid_uri_identifier_1 = "did:uri:new";
        let valid_uri_identifier_2 = "did:indy:idunion:test:2MZYuPv2Km7Q1eD4GCsSb6";
        let valid_uri_identifier_3 = "did:indy:sovrin:staging:6cgbu8ZPoWTnR5Rv5JcSMB";
        let valid_uri_identifier_4 = "did:indy:sovrin:7Tqg6BwSSWapxgUDm9KKgg";
        let valid_uri_identifier_5 = "did:web:example.com#controller";
        let valid_uri_identifier_6 = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";

        let invalid_uri_identifier = "::::";

        let valid_legacy_identifier_1 = "NcYxiDXkpYi6ov5FcYDi1e";
        let valid_legacy_identifier_2 = "VsKV7grR1BUE29mG2Fm2kX";

        let too_short_legacy_identifier = "abc";
        let illegal_base58_legacy_identifier_zero = "0000000000000000000000";
        let illegal_base58_legacy_identifier_captial_o = "OOOOOOOOOOOOOOOOOOOOOO";
        let illegal_base58_legacy_identifier_captial_i = "IIIIIIIIIIIIIIIIIIIIII";
        let illegal_base58_legacy_identifier_lower_l = "llllllllllllllllllllll";

        // Instantiating a new IssuerId validates it
        assert!(IssuerId::new(valid_uri_identifier_1).is_ok());
        assert!(IssuerId::new(valid_uri_identifier_2).is_ok());
        assert!(IssuerId::new(valid_uri_identifier_3).is_ok());
        assert!(IssuerId::new(valid_uri_identifier_4).is_ok());
        assert!(IssuerId::new(valid_uri_identifier_5).is_ok());
        assert!(IssuerId::new(valid_uri_identifier_6).is_ok());

        assert!(IssuerId::new(invalid_uri_identifier).is_err());

        assert!(IssuerId::new(valid_legacy_identifier_1).is_ok());
        assert!(IssuerId::new(valid_legacy_identifier_2).is_ok());

        assert!(IssuerId::new(too_short_legacy_identifier).is_err());
        assert!(IssuerId::new(illegal_base58_legacy_identifier_zero).is_err());
        assert!(IssuerId::new(illegal_base58_legacy_identifier_captial_o).is_err());
        assert!(IssuerId::new(illegal_base58_legacy_identifier_captial_i).is_err());
        assert!(IssuerId::new(illegal_base58_legacy_identifier_lower_l).is_err());
    }
}
