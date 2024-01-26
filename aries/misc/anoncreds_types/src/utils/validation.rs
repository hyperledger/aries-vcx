use crate::error::ValidationError;
use once_cell::sync::Lazy;
use regex::Regex;

// TODO: stricten the URI regex.
// Right now everything after the first colon is allowed,
// we might want to restrict this
pub static URI_IDENTIFIER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9\+\-\.]+:.+$").unwrap());

/// base58 alpahet as defined in the [base58
/// specification](https://datatracker.ietf.org/doc/html/draft-msporny-base58#section-2) This is
/// used for legacy indy identifiers that we will keep supporting for backwards compatibility. This
/// might validate invalid identifiers if they happen to fall within the base58 alphabet, but there
/// is not much we can do about that.
pub static LEGACY_DID_IDENTIFIER: Lazy<Regex> =
    Lazy::new(|| Regex::new("^[1-9A-HJ-NP-Za-km-z]{21,22}$").unwrap());

pub static LEGACY_SCHEMA_IDENTIFIER: Lazy<Regex> =
    Lazy::new(|| Regex::new("^[1-9A-HJ-NP-Za-km-z]{21,22}:2:.+:[0-9.]+$").unwrap());

pub static LEGACY_CRED_DEF_IDENTIFIER: Lazy<Regex> = Lazy::new(|| {
    Regex::new("^[1-9A-HJ-NP-Za-km-z]{21,22}:3:CL:(([1-9][0-9]*)|([a-zA-Z0-9]{21,22}:2:.+:[0-9.]+)):(.+)?$").unwrap()
});

pub fn is_uri_identifier(id: &str) -> bool {
    URI_IDENTIFIER.captures(id).is_some()
}

/// Macro to return a new `ValidationError` with an optional message
#[macro_export]
macro_rules! invalid {
    () => { $crate::error::ValidationError::from(None) };
    ($($arg:tt)+) => {
        $crate::error::ValidationError::from(format!($($arg)+))
    };
}

/// Trait for data types which need validation after being loaded from external sources
/// TODO: this should not default to Ok(())
pub trait Validatable {
    fn validate(&self) -> Result<(), ValidationError> {
        Ok(())
    }
}

#[cfg(test)]
mod test_identifiers {
    use super::*;

    #[test]
    fn should_validate_valid_identifiers() {
        let valid_uri_identifier = "mock:uri";
        let valid_legacy_schema_identifier = "DXoTtQJNtXtiwWaZAK3rB1:2:example:1.0";
        let valid_legacy_cred_def_identifier = "DXoTtQJNtXtiwWaZAK3rB1:3:CL:98153:default";
        let valid_legacy_did_identifier = "DXoTtQJNtXtiwWaZAK3rB1";

        assert!(URI_IDENTIFIER.captures(valid_uri_identifier).is_some());
        assert!(LEGACY_SCHEMA_IDENTIFIER
            .captures(valid_legacy_schema_identifier)
            .is_some());
        assert!(LEGACY_CRED_DEF_IDENTIFIER
            .captures(valid_legacy_cred_def_identifier)
            .is_some());
        assert!(LEGACY_DID_IDENTIFIER
            .captures(valid_legacy_did_identifier)
            .is_some());
    }

    #[test]
    fn should_not_validate_invalid_identifiers() {
        let invalid_uri_identifier = "DXoTtQJNtXtiwWaZAK3rB1";
        let invalid_legacy_schema_identifier = "invalid:id";
        let invalid_legacy_cred_def_identifier = "invalid:id";
        let invalid_legacy_did_identifier = "invalid:id";

        assert!(URI_IDENTIFIER.captures(invalid_uri_identifier).is_none());
        assert!(LEGACY_DID_IDENTIFIER
            .captures(invalid_legacy_schema_identifier)
            .is_none());
        assert!(LEGACY_CRED_DEF_IDENTIFIER
            .captures(invalid_legacy_cred_def_identifier)
            .is_none());
        assert!(LEGACY_DID_IDENTIFIER
            .captures(invalid_legacy_did_identifier)
            .is_none());

        assert!(LEGACY_SCHEMA_IDENTIFIER
            .captures("DXoTtQJNtXtiwWaZAK3rB1:3:example:1.0")
            .is_none());
        assert!(LEGACY_CRED_DEF_IDENTIFIER
            .captures("DXoTtQJNtXtiwWaZAK3rB1:4:CL:98153:default")
            .is_none());
    }
}
