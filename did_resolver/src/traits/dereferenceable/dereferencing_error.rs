use std::{error::Error, fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DIDDereferencingError {
    InvalidDid,
    NotFound,
}

impl Display for DIDDereferencingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DIDDereferencingError::InvalidDid => write!(f, "invalidDid"),
            DIDDereferencingError::NotFound => write!(f, "notFound"),
        }
    }
}

impl Error for DIDDereferencingError {}
