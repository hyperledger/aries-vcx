use serde::{Deserialize, Serialize};
use url::Url as UrlDep;

use crate::error::DIDDocumentBuilderError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Url(UrlDep);

impl Url {
    pub fn new(url: String) -> Result<Self, DIDDocumentBuilderError> {
        Ok(Self(UrlDep::parse(&url).unwrap()))
    }

    pub fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}
