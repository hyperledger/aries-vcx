use aries_vcx::{
    did_doc_sov::{service::ServiceSov, DidDocumentSov},
    messages::AriesMessage,
    utils::{encryption_envelope::EncryptionEnvelope, from_did_doc_sov_to_legacy},
};
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use serde_json::json;
use url::Url;

use crate::{AgentError, AgentErrorKind, AgentResult};

pub fn get_their_endpoint(did_document: &DidDocumentSov) -> AgentResult<Url> {
    let service = did_document.service().first().ok_or(AgentError::from_msg(
        AgentErrorKind::InvalidState,
        "No service found",
    ))?;
    // todo: will get cleaned up after service is de-generified
    let url: String = match service {
        ServiceSov::Legacy(d) => d.service_endpoint().to_string(),
        ServiceSov::AIP1(d) => d.service_endpoint().to_string(),
        ServiceSov::DIDCommV1(d) => d.service_endpoint().to_string(),
        ServiceSov::DIDCommV2(d) => d.service_endpoint().to_string(),
    };
    Url::parse(&url).map_err(|err| {
        AgentError::from_msg(
            AgentErrorKind::InvalidState,
            &format!("Failed to parse url found in did document due: {:?}", err),
        )
    })
}

pub async fn pairwise_encrypt(
    our_did_doc: &DidDocumentSov,
    their_did_doc: &DidDocumentSov,
    wallet: &impl BaseWallet,
    message: &AriesMessage,
) -> AgentResult<EncryptionEnvelope> {
    let sender_verkey = our_did_doc
        .resolved_key_agreement()
        .next()
        .ok_or_else(|| {
            AgentError::from_msg(
                AgentErrorKind::InvalidState,
                "No key agreement method found in our did document",
            )
        })?
        .public_key()?
        .base58();
    EncryptionEnvelope::create(
        wallet,
        json!(message).to_string().as_bytes(),
        Some(&sender_verkey),
        &from_did_doc_sov_to_legacy(their_did_doc.clone())?,
    )
    .await
    .map_err(|err| err.into())
}
