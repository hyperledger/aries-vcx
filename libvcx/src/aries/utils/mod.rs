use crate::aries::messages::connection::did_doc::DidDoc;
use crate::aries::messages::a2a::A2AMessage;
use crate::aries::utils::encryption_envelope::EncryptionEnvelope;
use crate::error::VcxResult;

pub mod encryption_envelope;


/**
Sends authenticated message to connection counterparty
*/
pub fn send_message(sender_verkey: &str, did_doc: &DidDoc, message: &A2AMessage) -> VcxResult<()> {
    trace!("DidDoc::send_message >>> message: {:?}, did_doc: {:?}", message, &did_doc);
    let envelope = EncryptionEnvelope::create(&message, Some(sender_verkey), &did_doc)?;
    agency_client::httpclient::post_message(&envelope.0, &did_doc.get_endpoint())?;
    Ok(())
}

/**
Sends anonymous message to connection counterparty
 */
pub fn send_message_anonymously(did_doc: &DidDoc, message: &A2AMessage) -> VcxResult<()> {
    trace!("DidDoc::send_message_anonymously >>> message: {:?}, did_doc: {:?}", message, &did_doc);
    let envelope = EncryptionEnvelope::create(&message, None, &did_doc)?;
    agency_client::httpclient::post_message(&envelope.0, &did_doc.get_endpoint())?;
    Ok(())
}