use std::env;
use std::path::PathBuf;

use crate::error::VcxResult;
use crate::messages::a2a::A2AMessage;
use crate::messages::connection::did_doc::DidDoc;
use crate::utils::encryption_envelope::EncryptionEnvelope;

#[macro_use]
pub mod version_constants;

#[macro_use]
#[cfg(feature = "test_utils")]
pub mod devsetup;

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! secret {
    ($val:expr) => {{ $val }};
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! secret {
    ($val:expr) => {{ "_" }};
}

#[cfg(test)]
macro_rules! map (
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
     };
);

pub mod error;
pub mod constants;
pub mod openssl;
pub mod json;
pub mod uuid;
pub mod author_agreement;
pub mod qualifier;
pub mod file;
pub mod mockdata;
pub mod provision;
pub mod random;

pub mod plugins;

#[macro_use]
pub mod test_logger;
pub mod validation;
pub mod serialization;
pub mod encryption_envelope;
pub mod filters;

pub fn get_temp_dir_path(filename: &str) -> PathBuf {
    let mut path = env::temp_dir();
    path.push(filename);
    path
}

pub fn send_message(sender_verkey: &str, did_doc: &DidDoc, message: &A2AMessage) -> VcxResult<()> {
    trace!("send_message >>> message: {:?}, did_doc: {:?}", message, &did_doc);
    let envelope = EncryptionEnvelope::create(&message, Some(sender_verkey), &did_doc)?;
    agency_client::httpclient::post_message(&envelope.0, &did_doc.get_endpoint())?;
    Ok(())
}

pub fn send_message_anonymously(did_doc: &DidDoc, message: &A2AMessage) -> VcxResult<()> {
    trace!("send_message_anonymously >>> message: {:?}, did_doc: {:?}", message, &did_doc);
    let envelope = EncryptionEnvelope::create(&message, None, &did_doc)?;
    agency_client::httpclient::post_message(&envelope.0, &did_doc.get_endpoint())?;
    Ok(())
}
