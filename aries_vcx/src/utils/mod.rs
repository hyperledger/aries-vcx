use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use messages::did_doc::DidDoc;
use crate::error::VcxResult;
use messages::a2a::A2AMessage;
use crate::utils::encryption_envelope::EncryptionEnvelope;
use crate::plugins::wallet::base_wallet::BaseWallet;

#[macro_use]
pub mod version_constants;

#[macro_use]
#[cfg(feature = "test_utils")]
pub mod devsetup;

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! secret {
    ($val:expr) => {{
        $val
    }};
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! secret {
    ($val:expr) => {{
        "_"
    }};
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

pub mod author_agreement;
#[rustfmt::skip]
pub mod constants;
pub mod error;
pub mod file;
pub mod json;
pub mod mockdata;
pub mod openssl;
pub mod provision;
pub mod qualifier;
pub mod random;
pub mod uuid;
pub mod async_fn_iterator;

#[macro_use]
pub mod test_logger;
pub mod encryption_envelope;
pub mod filters;
pub mod serialization;
pub mod validation;

pub fn get_temp_dir_path(filename: &str) -> PathBuf {
    let mut path = env::temp_dir();
    path.push(filename);
    path
}

pub async fn send_message(
    wallet: Arc<dyn BaseWallet>,
    sender_verkey: String,
    did_doc: DidDoc,
    message: A2AMessage,
) -> VcxResult<()> {
    trace!("send_message >>> message: {:?}, did_doc: {:?}", message, &did_doc);
    let EncryptionEnvelope(envelope) = EncryptionEnvelope::create(
        &wallet, 
        &message, 
        Some(&sender_verkey), 
        &did_doc)
        .await?;
        
    agency_client::httpclient::post_message(envelope, &did_doc.get_endpoint()).await?;
    Ok(())
}

pub async fn send_message_anonymously(
    wallet: Arc<dyn BaseWallet>,
    did_doc: &DidDoc,
    message: &A2AMessage,
) -> VcxResult<()> {
    trace!(
        "send_message_anonymously >>> message: {:?}, did_doc: {:?}",
        message,
        &did_doc
    );
    let EncryptionEnvelope(envelope) = EncryptionEnvelope::create(
        &wallet, 
        &message, 
        None,
        &did_doc)
        .await?;
        
    agency_client::httpclient::post_message(envelope, &did_doc.get_endpoint()).await?;
    Ok(())
}
