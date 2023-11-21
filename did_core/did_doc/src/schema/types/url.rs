use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use url::Url as UrlDep;

use crate::error::DidDocumentBuilderError;

// TODO: This was bad idea, get rid of it
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Url(UrlDep);

impl Url {
    pub fn new(url: &str) -> Result<Self, DidDocumentBuilderError> {
        Ok(Self(UrlDep::parse(url)?))
    }
}

impl TryFrom<&str> for Url {
    type Error = DidDocumentBuilderError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self(UrlDep::parse(value)?))
    }
}

impl FromStr for Url {
    type Err = DidDocumentBuilderError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(UrlDep::parse(s)?))
    }
}

impl From<UrlDep> for Url {
    fn from(url: UrlDep) -> Self {
        Self(url)
    }
}

impl From<Url> for UrlDep {
    fn from(url: Url) -> Self {
        url.0
    }
}

impl AsRef<str> for Url {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.as_str().fmt(f)
    }
}
