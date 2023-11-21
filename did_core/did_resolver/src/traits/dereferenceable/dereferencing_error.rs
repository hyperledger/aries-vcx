use std::{error::Error, fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DidDereferencingError {
    InvalidDid,
    NotFound,
}

impl Display for DidDereferencingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DidDereferencingError::InvalidDid => write!(f, "invalidDid"),
            DidDereferencingError::NotFound => write!(f, "notFound"),
        }
    }
}

impl Error for DidDereferencingError {}
