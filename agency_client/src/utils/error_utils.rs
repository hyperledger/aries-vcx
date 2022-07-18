use std::collections::HashMap;
use std::ffi::CString;
use std::fmt;
use crate::AgencyClientErrorKind;

pub fn kind_to_error_message(kind: &AgencyClientErrorKind) -> String {
    match kind {
        AgencyClientErrorKind::InvalidState => "Object is in invalid state for requested operation".into(),
        AgencyClientErrorKind::InvalidConfiguration => "Invalid Configuration".into(),
        AgencyClientErrorKind::InvalidJson => "Invalid JSON string".into(),
        AgencyClientErrorKind::InvalidOption => "Invalid Option".into(),
        AgencyClientErrorKind::InvalidMessagePack => "Invalid MessagePack".into(),
        AgencyClientErrorKind::IOError => "IO Error".into(),
        AgencyClientErrorKind::LibindyInvalidStructure => "Object (json, config, key, credential and etc...) passed to libindy has invalid structure".into(),
        AgencyClientErrorKind::TimeoutLibindy => "Waiting for libindy callback timed out".into(),
        AgencyClientErrorKind::InvalidLibindyParam => "Parameter passed to libindy was invalid".into(),
        AgencyClientErrorKind::PostMessageFailed => "Message failed in post".into(),
        AgencyClientErrorKind::InvalidWalletHandle => "Invalid Wallet or Search Handle".into(),
        AgencyClientErrorKind::DuplicationWallet => "Indy wallet already exists".into(),
        AgencyClientErrorKind::WalletRecordNotFound => "Wallet record not found".into(),
        AgencyClientErrorKind::DuplicationWalletRecord => "Record already exists in the wallet".into(),
        AgencyClientErrorKind::WalletNotFound => "Wallet Not Found".into(),
        AgencyClientErrorKind::WalletAlreadyOpen => "Indy wallet already open".into(),
        AgencyClientErrorKind::MissingWalletKey => "Configuration is missing wallet key".into(),
        AgencyClientErrorKind::DuplicationMasterSecret => "Attempted to add a Master Secret that already existed in wallet".into(),
        AgencyClientErrorKind::DuplicationDid => "Attempted to add a DID to wallet when that DID already exists in wallet".into(),
        AgencyClientErrorKind::UnknownError => "Unknown Error".into(),
        AgencyClientErrorKind::InvalidDid => "Invalid DID".into(),
        AgencyClientErrorKind::InvalidVerkey => "Invalid Verkey".into(),
        AgencyClientErrorKind::InvalidUrl => "Invalid URL".into(),
        AgencyClientErrorKind::SerializationError => "Unable to serialize".into(),
        AgencyClientErrorKind::NotBase58 => "Value needs to be base58".into(),
        AgencyClientErrorKind::InvalidHttpResponse => "Invalid HTTP response.".into(),
        AgencyClientErrorKind::CreateAgent => "Failed to create agency client".into(),
        AgencyClientErrorKind::UnknownLibndyError => "Unknown libindy error".into(),
        AgencyClientErrorKind::LibndyError(e) => format!("Libindy error with code {}", e).into(),
    }
}
