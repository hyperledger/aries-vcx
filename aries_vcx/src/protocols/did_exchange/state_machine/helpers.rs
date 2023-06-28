use std::{collections::HashMap, sync::Arc};

use aries_vcx_core::wallet::base_wallet::BaseWallet;
use did_doc::schema::{
    types::uri::Uri,
    verification_method::{VerificationMethod, VerificationMethodKind, VerificationMethodType},
};
use did_doc_sov::{
    extra_fields::{didcommv1::ExtraFieldsDidCommV1, KeyKind},
    service::{didcommv1::ServiceDidCommV1, ServiceSov},
    DidDocumentSov,
};
use did_key::DidKey;
use did_parser::{Did, DidUrl};
use did_peer::peer_did::generate::generate_numalgo2;
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use messages::decorators::attachment::{Attachment, AttachmentData, AttachmentType};
use public_key::{Key, KeyType};
use serde_json::Value;
use url::Url;

use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    protocols::{
        did_exchange::transition::transition_error::TransitionError, mediated_connection::pairwise_info::PairwiseInfo,
    },
    utils::from_legacy_did_doc_to_sov,
};

pub async fn generate_keypair(wallet: &Arc<dyn BaseWallet>, key_type: KeyType) -> Result<Key, AriesVcxError> {
    let pairwise_info = PairwiseInfo::create(wallet).await?;
    Ok(Key::from_base58(&pairwise_info.pw_vk, key_type)?)
}

pub fn construct_service(
    routing_keys: Vec<KeyKind>,
    recipient_keys: Vec<KeyKind>,
    service_endpoint: Url,
) -> Result<ServiceSov, AriesVcxError> {
    let extra = ExtraFieldsDidCommV1::builder()
        .set_routing_keys(routing_keys)
        .set_recipient_keys(recipient_keys)
        .build();
    let service = ServiceSov::DIDCommV1(ServiceDidCommV1::new(Uri::new("#0")?, service_endpoint.into(), extra)?);
    Ok(service)
}

pub async fn create_our_did_document(
    wallet: &Arc<dyn BaseWallet>,
    service_endpoint: Url,
    routing_keys: Vec<String>,
) -> Result<(DidDocumentSov, Key), AriesVcxError> {
    let key_ver = generate_keypair(wallet, KeyType::Ed25519).await?;
    let key_enc = generate_keypair(wallet, KeyType::X25519).await?;
    let service = construct_service(
        routing_keys.into_iter().map(KeyKind::Value).collect(),
        vec![KeyKind::DidKey(key_enc.clone().try_into()?)],
        service_endpoint,
    )?;

    // TODO: Make it easier to generate peer did from keys and service, and generate DDO from it
    let did_document_temp = did_doc_from_keys(Default::default(), key_ver.clone(), key_enc.clone(), service.clone())?;
    let peer_did = generate_numalgo2(did_document_temp.into())?;

    Ok((
        did_doc_from_keys(peer_did.clone().into(), key_ver, key_enc.clone(), service)?,
        key_enc,
    ))
}

fn did_doc_from_keys(
    did: Did,
    key_ver: Key,
    key_enc: Key,
    service: ServiceSov,
) -> Result<DidDocumentSov, AriesVcxError> {
    let vm_ver_id = DidUrl::from_fragment(key_ver.short_prefixless_fingerprint())?;
    let vm_ka_id = DidUrl::from_fragment(key_enc.short_prefixless_fingerprint())?;
    let vm_ver = VerificationMethod::builder(
        vm_ver_id,
        did.clone(),
        VerificationMethodType::Ed25519VerificationKey2020,
    )
    .add_public_key_base58(key_ver.base58())
    .build();
    let vm_ka = VerificationMethod::builder(vm_ka_id, did.clone(), VerificationMethodType::X25519KeyAgreementKey2020)
        .add_public_key_base58(key_enc.base58())
        .build();
    Ok(DidDocumentSov::builder(did)
        .add_service(service)
        .add_verification_method(vm_ver.clone())
        // TODO: Include just reference
        .add_key_agreement(VerificationMethodKind::Resolved(vm_ka))
        .build())
}

pub fn ddo_sov_to_attach(ddo: DidDocumentSov) -> Result<Attachment, AriesVcxError> {
    // Interop note: acapy accepts unsigned when using peer dids?
    Ok(Attachment::new(AttachmentData::new(AttachmentType::Base64(
        base64::encode_config(&serde_json::to_string(&ddo)?, base64::URL_SAFE_NO_PAD),
    ))))
}

// TODO: Obviously, extract attachment signing
// TODO: JWS verification
pub async fn jws_sign_attach(
    mut attach: Attachment,
    verkey: Key,
    wallet: &Arc<dyn BaseWallet>,
) -> Result<Attachment, AriesVcxError> {
    if let AttachmentType::Base64(attach_base64) = &attach.data.content {
        let did_key: DidKey = verkey.clone().try_into()?;
        let verkey_b64 = base64::encode_config(verkey.key(), base64::URL_SAFE_NO_PAD);
        let protected_header = json!({
            "alg": "EdDSA",
            "jwk": {
                "kty": "OKP",
                "kid": did_key.to_string(),
                "crv": "Ed25519",
                "x": verkey_b64
            }
        });
        let unprotected_header = json!({
            // TODO: Needs to be both protected and unprotected, does it make sense?
            "kid": did_key.to_string(),
        });
        let b64_protected = base64::encode_config(&protected_header.to_string(), base64::URL_SAFE_NO_PAD);
        let sign_input = format!("{}.{}", b64_protected, attach_base64).into_bytes();
        let signed = wallet.sign(&verkey.base58(), &sign_input).await?;
        let signature_base64 = base64::encode_config(&signed, base64::URL_SAFE_NO_PAD);

        let jws = {
            let mut jws = HashMap::new();
            jws.insert("header".to_string(), Value::from(unprotected_header));
            jws.insert("protected".to_string(), Value::String(b64_protected));
            jws.insert("signature".to_string(), Value::String(signature_base64));
            jws
        };
        attach.data.jws = Some(jws);
        Ok(attach)
    } else {
        Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidState,
            "Cannot sign non-base64-encoded attachment",
        ))
    }
}

pub fn attach_to_ddo_sov(attachment: Attachment) -> Result<DidDocumentSov, AriesVcxError> {
    match attachment.data.content {
        AttachmentType::Json(value) => serde_json::from_value(value).map_err(Into::into),
        AttachmentType::Base64(ref value) => {
            let bytes = base64::decode(&value).map_err(|err| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::SerializationError,
                    format!("Attachment base 64 decoding failed; attach: {attachment:?}, err: {err}"),
                )
            })?;
            serde_json::from_slice::<DidDocumentSov>(&bytes).map_err(Into::into)
        }
        _ => Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            "Attachment is not a JSON or Base64",
        )),
    }
}

pub fn to_transition_error<S, T>(state: S) -> impl FnOnce(T) -> TransitionError<S>
where
    T: Into<AriesVcxError>,
{
    move |error| TransitionError {
        error: error.into(),
        state,
    }
}
