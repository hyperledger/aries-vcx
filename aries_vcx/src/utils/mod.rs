use aries_vcx_core::wallet::base_wallet::BaseWallet;
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use messages::AriesMessage;

use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    utils::encryption_envelope::EncryptionEnvelope,
};

#[macro_use]
#[cfg(feature = "vdrtools_wallet")]
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

#[rustfmt::skip]
pub mod constants;
pub mod file;
pub mod mockdata;
pub mod openssl;
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
    wallet: &impl BaseWallet,
    sender_verkey: String,
    did_doc: AriesDidDoc,
    message: AriesMessage,
) -> VcxResult<()> {
    trace!(
        "send_message >>> message: {:?}, did_doc: {:?}",
        message,
        &did_doc
    );

    let EncryptionEnvelope(envelope) = EncryptionEnvelope::create(
        wallet,
        json!(message).to_string().as_bytes(),
        Some(&sender_verkey),
        &did_doc,
    )
    .await?;

    // TODO: Extract from agency client
    agency_client::httpclient::post_message(
        envelope,
        did_doc.get_endpoint().ok_or_else(|| {
            AriesVcxError::from_msg(AriesVcxErrorKind::InvalidUrl, "No URL in DID Doc")
        })?,
    )
    .await?;
    Ok(())
}

pub async fn send_message_anonymously(
    wallet: &impl BaseWallet,
    did_doc: &AriesDidDoc,
    message: &AriesMessage,
) -> VcxResult<()> {
    trace!(
        "send_message_anonymously >>> message: {:?}, did_doc: {:?}",
        message,
        &did_doc
    );
    let EncryptionEnvelope(envelope) =
        EncryptionEnvelope::create(wallet, json!(message).to_string().as_bytes(), None, did_doc)
            .await?;

    agency_client::httpclient::post_message(
        envelope,
        did_doc.get_endpoint().ok_or_else(|| {
            AriesVcxError::from_msg(AriesVcxErrorKind::InvalidUrl, "No URL in DID Doc")
        })?,
    )
    .await?;
    Ok(())
}
