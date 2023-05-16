use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::error::DidDocumentBuilderError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Uri(uniresid::Uri);

impl Uri {
    pub fn new(uri: &str) -> Result<Self, DidDocumentBuilderError> {
        Ok(Self(uniresid::Uri::try_from(uri).map_err(|e| {
            DidDocumentBuilderError::InvalidInput(format!("Invalid URI: {}", e))
        })?))
    }
}

impl FromStr for Uri {
    type Err = DidDocumentBuilderError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl Display for Uri {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<uniresid::Uri> for Uri {
    fn as_ref(&self) -> &uniresid::Uri {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uri_new_valid() {
        let uri = Uri::new("http://example.com");
        assert!(uri.is_ok());
    }

    #[test]
    fn test_uri_new_invalid() {
        let uri = Uri::new(r"http:\\example.com\index.html");
        assert!(uri.is_err());
    }

    #[test]
    fn test_uri_from_str_valid() {
        let uri = Uri::from_str("http://example.com");
        assert!(uri.is_ok());
    }

    #[test]
    fn test_uri_from_str_invalid() {
        let uri = Uri::from_str(
            r"http:\\example.com\index.html
",
        );
        assert!(uri.is_err());
    }

    #[test]
    fn test_uri_clone() {
        let uri_str = "http://example.com";
        let uri = Uri::from_str(uri_str).unwrap();
        let uri_clone = uri.clone();
        assert_eq!(uri, uri_clone);
    }

    #[test]
    fn test_uri_partial_eq() {
        let uri1 = Uri::from_str("http://example.com").unwrap();
        let uri2 = Uri::from_str("http://example.com").unwrap();
        let uri3 = Uri::from_str("http://different.com").unwrap();

        assert_eq!(uri1, uri2);
        assert_ne!(uri1, uri3);
    }
}
