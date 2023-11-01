use std::collections::HashMap;

use aries_vcx_core::wallet::base_wallet::BaseWallet;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use did_doc::schema::{
    types::uri::Uri,
    verification_method::{VerificationMethod, VerificationMethodKind, VerificationMethodType},
};
use did_doc_sov::{
    extra_fields::{didcommv1::ExtraFieldsDidCommV1, SovKeyKind},
    service::{didcommv1::ServiceDidCommV1},
};
use did_key::DidKey;
use did_parser::{Did, DidUrl};
use did_peer::peer_did::{numalgos::numalgo2::Numalgo2, PeerDid};
use messages::decorators::attachment::{Attachment, AttachmentData, AttachmentType};
use public_key::{Key, KeyType};
use serde_json::Value;
use url::Url;
use did_doc::schema::did_doc::DidDocument;
use did_doc::schema::service::Service;

use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    protocols::{
        did_exchange::transition::transition_error::TransitionError,
        mediated_connection::pairwise_info::PairwiseInfo,
    },
};

pub async fn generate_keypair(
    wallet: &impl BaseWallet,
    key_type: KeyType,
) -> Result<Key, AriesVcxError> {
    let pairwise_info = PairwiseInfo::create(wallet).await?;
    Ok(Key::from_base58(&pairwise_info.pw_vk, key_type)?)
}

pub async fn create_our_did_document(
    wallet: &impl BaseWallet,
    service_endpoint: Url,
    routing_keys: Vec<String>,
) -> Result<(DidDocument, Key), AriesVcxError> {
    let key_ver = generate_keypair(wallet, KeyType::Ed25519).await?;
    let key_enc = generate_keypair(wallet, KeyType::Ed25519).await?;

    let service: Service = ServiceDidCommV1::new(
        Uri::new("#0")?,
        service_endpoint.into(),
        ExtraFieldsDidCommV1::builder()
            .set_routing_keys(
                routing_keys.into_iter().map(SovKeyKind::Value).collect()
            )
            .set_recipient_keys(
                vec![SovKeyKind::DidKey(key_enc.clone().try_into()?)]
            )
            .build(),
    ).into();

    let mut did_document = did_doc_from_keys(
        Default::default(),
        key_ver.clone(),
        key_enc.clone(),
        service.clone(),
    )?;
    let peer_did = PeerDid::<Numalgo2>::from_did_doc(did_document.into())?;
    Ok((
        did_document.set_id(peer_did.did().clone())?,
        key_enc,
    ))
}

// todo: add unit tests
fn did_doc_from_keys(
    did: Did,
    key_ver: Key,
    key_enc: Key,
    service: Service,
) -> Result<DidDocument, AriesVcxError> {
    let vm_ver_id = DidUrl::from_fragment(key_ver.short_prefixless_fingerprint())?;
    let vm_ka_id = DidUrl::from_fragment(key_enc.short_prefixless_fingerprint())?;
    let vm_ver = VerificationMethod::builder(
        vm_ver_id,
        did.clone(),
        VerificationMethodType::Ed25519VerificationKey2020,
    )
    .add_public_key_base58(key_ver.base58())
    .build();
    let vm_ka = VerificationMethod::builder(
        vm_ka_id,
        did.clone(),
        VerificationMethodType::X25519KeyAgreementKey2020,
    )
    .add_public_key_base58(key_enc.base58())
    .build();
    Ok(DidDocument::builder(did)
        .add_service(service)
        .add_verification_method(vm_ver)
        // TODO: Include just reference
        .add_key_agreement(vm_ka)
        .build())
}

pub fn ddo_to_attach(ddo: DidDocument) -> Result<Attachment, AriesVcxError> {
    // Interop note: acapy accepts unsigned when using peer dids?
    let content_b64 =
        base64::engine::Engine::encode(&URL_SAFE_NO_PAD, serde_json::to_string(&ddo)?);
    Ok(Attachment::builder()
        .data(
            AttachmentData::builder()
                .content(AttachmentType::Base64(content_b64))
                .build(),
        )
        .build())
}

// TODO: Obviously, extract attachment signing
// TODO: JWS verification
pub async fn jws_sign_attach(
    mut attach: Attachment,
    verkey: Key,
    wallet: &impl BaseWallet,
) -> Result<Attachment, AriesVcxError> {
    if let AttachmentType::Base64(attach_base64) = &attach.data.content {
        let did_key: DidKey = verkey.clone().try_into()?;
        let verkey_b64 = base64::engine::Engine::encode(&URL_SAFE_NO_PAD, verkey.key());

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
        let b64_protected =
            base64::engine::Engine::encode(&URL_SAFE_NO_PAD, protected_header.to_string());
        let sign_input = format!("{}.{}", b64_protected, attach_base64).into_bytes();
        let signed = wallet.sign(&verkey.base58(), &sign_input).await?;
        let signature_base64 = base64::engine::Engine::encode(&URL_SAFE_NO_PAD, signed);

        let jws = {
            let mut jws = HashMap::new();
            jws.insert("header".to_string(), unprotected_header);
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
            let bytes = base64::Engine::decode(&URL_SAFE_NO_PAD, value).map_err(|err| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::SerializationError,
                    format!(
                        "Attachment base 64 decoding failed; attach: {attachment:?}, err: {err}"
                    ),
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


// todo: this is new test, yet to run it after we get stuff compiling
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test] // Use tokio::test if your generate_keypair function is async
    async fn test_did_doc_from_keys() {
        // Assuming generate_keypair and other functions are available in your context
        let key_ver = Key::new("7MV7mTpzQekW39mXdPXKnRJn79kkzMvmtaSHZWUSbvt5".into(), KeyType::Ed25519).unwrap();
        let key_enc = Key::new("tyntrez7bCthPqvZUDGwhYB1bSe9HzpLdSeHFpuSwst".into(), KeyType::Ed25519).unwrap();

        let service_endpoint = Url::parse("http://example.com").unwrap();
        let routing_keys = vec!["routing_key1".to_string(), "routing_key2".to_string()];
        let service: Service = ServiceDidCommV1::new(
            Uri::new("#0")?,
            service_endpoint.into(),
            ExtraFieldsDidCommV1::builder()
                .set_routing_keys(
                    routing_keys.into_iter().map(SovKeyKind::Value).collect()
                )
                .set_recipient_keys(
                    vec![SovKeyKind::DidKey(key_enc.clone().try_into()?)]
                )
                .build(),
        ).try_into().unwrap();

        let did = Did::default();

        let result = did_doc_from_keys(did.clone(), key_ver.clone(), key_enc.clone(), service.clone());

        assert!(result.is_ok());
        let did_doc = result.unwrap();

        assert_eq!(did_doc.services().len(), 1);
        assert_eq!(did_doc.verification_methods().len(), 1);
        assert_eq!(did_doc.key_agreements().len(), 1);
        // ...other assertions as needed
    }
}
