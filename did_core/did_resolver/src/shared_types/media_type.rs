use std::fmt::{self, Display, Formatter};

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub enum MediaType {
    DidJson,
    DidLdJson,
}

impl Display for MediaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MediaType::DidJson => write!(f, "application/did+json"),
            MediaType::DidLdJson => write!(f, "application/did+ld+json"),
        }
    }
}
