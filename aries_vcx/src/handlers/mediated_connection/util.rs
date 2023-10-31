use aries_vcx_core::wallet::base_wallet::BaseWallet;
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use messages::AriesMessage;
use shared_vcx::http_client::post_message;

use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    utils::encryption_envelope::EncryptionEnvelope,
};

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

    post_message(
        envelope,
        did_doc.get_endpoint().ok_or_else(|| {
            AriesVcxError::from_msg(AriesVcxErrorKind::InvalidUrl, "No URL in DID Doc")
        })?,
    )
    .await?;

    Ok(())
}
