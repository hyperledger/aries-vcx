use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use aries_vcx_core::wallet::base_wallet::BaseWallet;
use diddoc_legacy::aries::diddoc::AriesDidDoc;

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};
use crate::utils::encryption_envelope::EncryptionEnvelope;
use messages::AriesMessage;

#[macro_use]
#[cfg(feature = "vdrtools")]
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

#[rustfmt::skip]
pub mod constants;
pub mod file;
pub mod mockdata;
pub mod openssl;
#[cfg(feature = "vdrtools")]
pub mod provision;
pub mod qualifier;
pub mod random;
pub mod uuid;

#[macro_use]
pub mod test_logger;
pub mod encryption_envelope;
pub mod filters;
pub mod serialization;
pub mod validation;

pub async fn send_message(
    wallet: Arc<dyn BaseWallet>,
    sender_verkey: String,
    did_doc: AriesDidDoc,
    message: AriesMessage,
) -> VcxResult<()> {
    trace!("send_message >>> message: {:?}, did_doc: {:?}", message, &did_doc);
    let EncryptionEnvelope(envelope) =
        EncryptionEnvelope::create(&wallet, &message, Some(&sender_verkey), &did_doc).await?;

    // TODO: Extract from agency client
    agency_client::httpclient::post_message(
        envelope,
        did_doc
            .get_endpoint()
            .ok_or_else(|| AriesVcxError::from_msg(AriesVcxErrorKind::InvalidUrl, "No URL in DID Doc"))?,
    )
    .await?;
    Ok(())
}

pub async fn send_message_anonymously(
    wallet: Arc<dyn BaseWallet>,
    did_doc: &AriesDidDoc,
    message: &AriesMessage,
) -> VcxResult<()> {
    trace!(
        "send_message_anonymously >>> message: {:?}, did_doc: {:?}",
        message,
        &did_doc
    );
    let EncryptionEnvelope(envelope) = EncryptionEnvelope::create(&wallet, message, None, did_doc).await?;

    agency_client::httpclient::post_message(
        envelope,
        did_doc
            .get_endpoint()
            .ok_or_else(|| AriesVcxError::from_msg(AriesVcxErrorKind::InvalidUrl, "No URL in DID Doc"))?,
    )
    .await?;
    Ok(())
}
