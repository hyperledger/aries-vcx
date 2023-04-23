use std::{error::Error, fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DIDResolutionError {
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

impl Display for DIDResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DIDResolutionError::InvalidDid => write!(f, "invalidDid"),
            DIDResolutionError::NotFound => write!(f, "notFound"),
            DIDResolutionError::RepresentationNotSupported => {
                write!(f, "representationNotSupported")
            }
            DIDResolutionError::MethodNotSupported => write!(f, "methodNotSupported"),
            DIDResolutionError::InternalError => write!(f, "internalError"),
            DIDResolutionError::InvalidPublicKey => write!(f, "invalidPublicKey"),
            DIDResolutionError::InvalidPublicKeyLength => write!(f, "invalidPublicKeyLength"),
            DIDResolutionError::InvalidPublicKeyType => write!(f, "invalidPublicKeyType"),
            DIDResolutionError::UnsupportedPublicKeyType => {
                write!(f, "unsupportedPublicKeyType")
            }
            DIDResolutionError::NotAllowedVerificationMethodType => {
                write!(f, "notAllowedVerificationMethodType")
            }
            DIDResolutionError::NotAllowedKeyType => write!(f, "notAllowedKeyType"),
            DIDResolutionError::NotAllowedMethod => write!(f, "notAllowedMethod"),
            DIDResolutionError::NotAllowedCertificate => write!(f, "notAllowedCertificate"),
            DIDResolutionError::NotAllowedLocalDuplicateKey => {
                write!(f, "notAllowedLocalDuplicateKey")
            }
            DIDResolutionError::NotAllowedLocalDerivedKey => {
                write!(f, "notAllowedLocalDerivedKey")
            }
            DIDResolutionError::NotAllowedGlobalDuplicateKey => {
                write!(f, "notAllowedGlobalDuplicateKey")
            }
        }
    }
}

impl Error for DIDResolutionError {}
