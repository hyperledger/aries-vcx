use std::collections::HashMap;

use aries_vcx_core::wallet::base_wallet::BaseWallet;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::Utc;
use did_doc::schema::{
    did_doc::DidDocument,
    service::{service_key_kind::ServiceKeyKind, typed::didcommv1::ServiceDidCommV1, Service},
    types::uri::Uri,
    verification_method::{VerificationMethod, VerificationMethodType},
};
use did_key::DidKey;
use did_parser_nom::{Did, DidUrl};
use did_peer::peer_did::{numalgos::numalgo2::Numalgo2, PeerDid};
use messages::{
    decorators::{
        attachment::{Attachment, AttachmentData, AttachmentType},
        thread::Thread,
        timing::Timing,
    },
    msg_fields::protocols::did_exchange::response::{
        Response, ResponseContent, ResponseDecorators,
    },
};
use public_key::{Key, KeyType};
use serde_json::Value;
use url::Url;
use uuid::Uuid;

use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    protocols::{
        did_exchange::transition::transition_error::TransitionError,
        mediated_connection::pairwise_info::PairwiseInfo,
    },
};

pub fn construct_response(
    request_id: String,
    our_did_document: &DidDocument,
    signed_attach: Attachment,
) -> Response {
    let content = ResponseContent::builder()
        .did(our_did_document.id().to_string())
        .did_doc(Some(signed_attach))
        .build();
    let decorators = ResponseDecorators::builder()
        .thread(Thread::builder().thid(request_id).build())
        .timing(Timing::builder().out_time(Utc::now()).build())
        .build();
    Response::builder()
        .id(Uuid::new_v4().to_string())
        .content(content)
        .decorators(decorators)
        .build()
}

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
    let key_enc = generate_keypair(wallet, KeyType::Ed25519).await?;

    let service: Service = ServiceDidCommV1::new(
        Uri::new("#0")?,
        service_endpoint,
        0,
        vec![],
        routing_keys
            .into_iter()
            .map(ServiceKeyKind::Value)
            .collect(),
    )
    .try_into()?;

    info!("Prepared service for peer:did:2 generation: {} ", service);
    let mut did_document = did_doc_from_keys(Default::default(), key_enc.clone(), service)?;
    info!(
        "Created did document for peer:did:2 generation: {} ",
        did_document
    );
    let peer_did = PeerDid::<Numalgo2>::from_did_doc(did_document.clone())?;
    did_document.set_id(peer_did.did().clone());
    Ok((did_document, key_enc))
}

fn did_doc_from_keys(
    did: Did,
    key_enc: Key,
    service: Service,
) -> Result<DidDocument, AriesVcxError> {
    let vm_ka_id = DidUrl::from_fragment(key_enc.short_prefixless_fingerprint())?;
    let vm_ka = VerificationMethod::builder(
        vm_ka_id,
        did.clone(),
        VerificationMethodType::Ed25519VerificationKey2020,
    )
    .add_public_key_base58(key_enc.base58())
    .build();
    Ok(DidDocument::builder(did)
        .add_service(service)
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
        let signed = wallet.sign(&verkey, &sign_input).await?;
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

pub fn attachment_to_diddoc(attachment: Attachment) -> Result<DidDocument, AriesVcxError> {
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
            serde_json::from_slice::<DidDocument>(&bytes).map_err(Into::into)
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

#[cfg(test)]
mod tests {
    use did_doc::schema::{
        service::typed::ServiceType, utils::OneOrList, verification_method::VerificationMethodKind,
    };

    use super::*;

    #[tokio::test]
    async fn test_did_doc_from_keys() {
        let key_enc = Key::new(
            "tyntrez7bCthPqvZUDGwhYB1bSe9HzpLdSeHFpuSwst".into(),
            KeyType::Ed25519,
        )
        .unwrap();

        let service_endpoint = Url::parse("http://example.com").unwrap();
        let routing_keys = vec![
            ServiceKeyKind::Value("routing_key1".into()),
            ServiceKeyKind::Value("routing_key2".into()),
        ];
        let service: Service = ServiceDidCommV1::new(
            Uri::new("#service-0").unwrap(),
            service_endpoint.clone(),
            0,
            vec![],
            routing_keys,
        )
        .try_into()
        .unwrap();

        let did = Did::default();

        let result = did_doc_from_keys(did, key_enc.clone(), service);

        assert!(result.is_ok());
        let did_doc = result.unwrap();

        assert_eq!(did_doc.service().len(), 1);
        let ddo_service = did_doc.service().first().unwrap();
        assert_eq!(&ddo_service.id().to_string(), "#service-0");
        assert_eq!(
            ddo_service.service_type(),
            &OneOrList::One(ServiceType::DIDCommV1)
        );
        assert_eq!(ddo_service.service_endpoint(), &service_endpoint);
        assert_eq!(
            ddo_service.extra_field_routing_keys().unwrap(),
            vec![
                ServiceKeyKind::Value("routing_key1".into()),
                ServiceKeyKind::Value("routing_key2".into())
            ]
        );

        assert_eq!(did_doc.key_agreement().len(), 1);
        match did_doc.key_agreement().first().unwrap() {
            VerificationMethodKind::Resolved(key_agreement) => {
                assert_eq!(key_agreement.public_key().unwrap(), key_enc);
            }
            VerificationMethodKind::Resolvable(_) => {
                panic!("Key agreement was expected to have embedded key");
            }
        }
    }
}
