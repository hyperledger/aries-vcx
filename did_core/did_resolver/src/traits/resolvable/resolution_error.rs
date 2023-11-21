use std::{error::Error, fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DidResolutionError {
    InvalidDid,
    NotFound,
    RepresentationNotSupported,
    MethodNotSupported,
    InternalError,
    InvalidPublicKey,
    InvalidPublicKeyLength,
    InvalidPublicKeyType,
    UnsupportedPublicKeyType,
    NotAllowedVerificationMethodType,
    NotAllowedKeyType,
    NotAllowedMethod,
    NotAllowedCertificate,
    NotAllowedLocalDuplicateKey,
    NotAllowedLocalDerivedKey,
    NotAllowedGlobalDuplicateKey,
}

impl Display for DidResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DidResolutionError::InvalidDid => write!(f, "invalidDid"),
            DidResolutionError::NotFound => write!(f, "notFound"),
            DidResolutionError::RepresentationNotSupported => {
                write!(f, "representationNotSupported")
            }
            DidResolutionError::MethodNotSupported => write!(f, "methodNotSupported"),
            DidResolutionError::InternalError => write!(f, "internalError"),
            DidResolutionError::InvalidPublicKey => write!(f, "invalidPublicKey"),
            DidResolutionError::InvalidPublicKeyLength => write!(f, "invalidPublicKeyLength"),
            DidResolutionError::InvalidPublicKeyType => write!(f, "invalidPublicKeyType"),
            DidResolutionError::UnsupportedPublicKeyType => {
                write!(f, "unsupportedPublicKeyType")
            }
            DidResolutionError::NotAllowedVerificationMethodType => {
                write!(f, "notAllowedVerificationMethodType")
            }
            DidResolutionError::NotAllowedKeyType => write!(f, "notAllowedKeyType"),
            DidResolutionError::NotAllowedMethod => write!(f, "notAllowedMethod"),
            DidResolutionError::NotAllowedCertificate => write!(f, "notAllowedCertificate"),
            DidResolutionError::NotAllowedLocalDuplicateKey => {
                write!(f, "notAllowedLocalDuplicateKey")
            }
            DidResolutionError::NotAllowedLocalDerivedKey => {
                write!(f, "notAllowedLocalDerivedKey")
            }
            DidResolutionError::NotAllowedGlobalDuplicateKey => {
                write!(f, "notAllowedGlobalDuplicateKey")
            }
        }
    }
}

impl Error for DidResolutionError {}
