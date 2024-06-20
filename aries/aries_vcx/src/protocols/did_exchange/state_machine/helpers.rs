use std::collections::HashMap;

use aries_vcx_wallet::wallet::base_wallet::BaseWallet;
use chrono::Utc;
use did_doc::schema::{
    did_doc::DidDocument,
    service::{service_key_kind::ServiceKeyKind, typed::didcommv1::ServiceDidCommV1, Service},
    types::uri::Uri,
    verification_method::{PublicKeyField, VerificationMethodType},
};
use did_key::DidKey;
use did_parser_nom::{Did, DidUrl};
use did_peer::peer_did::{
    numalgos::numalgo4::{
        construction_did_doc::{DidPeer4ConstructionDidDocument, DidPeer4VerificationMethod},
        Numalgo4,
    },
    PeerDid,
};
use messages::{
    decorators::{
        attachment::{Attachment, AttachmentData, AttachmentType},
        thread::Thread,
        timing::Timing,
    },
    msg_fields::protocols::did_exchange::{
        v1_0::response::{Response as ResponseV1_0, ResponseContent as ResponseV1_0Content},
        v1_1::response::{Response as ResponseV1_1, ResponseContent as ResponseV1_1Content},
        v1_x::response::ResponseDecorators,
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
    utils::base64::URL_SAFE_LENIENT,
};

pub(crate) fn construct_response_v1_0(
    // pthid inclusion is overkill in practice, but needed. see: https://github.com/hyperledger/aries-rfcs/issues/817
    request_pthid: Option<String>,
    request_id: String,
    did: &Did,
    signed_diddoc_attach: Attachment,
) -> ResponseV1_0 {
    let thread = match request_pthid {
        Some(request_pthid) => Thread::builder()
            .thid(request_id)
            .pthid(request_pthid)
            .build(),
        None => Thread::builder().thid(request_id).build(),
    };

    let content = ResponseV1_0Content::builder()
        .did(did.to_string())
        .did_doc(Some(signed_diddoc_attach))
        .build();
    let decorators = ResponseDecorators::builder()
        .thread(thread)
        .timing(Timing::builder().out_time(Utc::now()).build())
        .build();
    ResponseV1_0::builder()
        .id(Uuid::new_v4().to_string())
        .content(content)
        .decorators(decorators)
        .build()
}

pub(crate) fn construct_response_v1_1(
    // pthid inclusion is overkill in practice, but needed. see: https://github.com/hyperledger/aries-rfcs/issues/817
    request_pthid: Option<String>,
    request_id: String,
    did: &Did,
    signed_didrotate_attach: Attachment,
) -> ResponseV1_1 {
    let thread = match request_pthid {
        Some(request_pthid) => Thread::builder()
            .thid(request_id)
            .pthid(request_pthid)
            .build(),
        None => Thread::builder().thid(request_id).build(),
    };

    let content = ResponseV1_1Content::builder()
        .did(did.to_string())
        .did_rotate(signed_didrotate_attach)
        .build();
    let decorators = ResponseDecorators::builder()
        .thread(thread)
        .timing(Timing::builder().out_time(Utc::now()).build())
        .build();
    ResponseV1_1::builder()
        .id(Uuid::new_v4().to_string())
        .content(content)
        .decorators(decorators)
        .build()
}

async fn generate_keypair(
    wallet: &impl BaseWallet,
    key_type: KeyType,
) -> Result<Key, AriesVcxError> {
    let pairwise_info = PairwiseInfo::create(wallet).await?;
    Ok(Key::from_base58(&pairwise_info.pw_vk, key_type)?)
}

pub async fn create_peer_did_4(
    wallet: &impl BaseWallet,
    service_endpoint: Url,
    routing_keys: Vec<String>,
) -> Result<(PeerDid<Numalgo4>, Key), AriesVcxError> {
    let key_enc = generate_keypair(wallet, KeyType::Ed25519).await?;

    let vm_ka_id = DidUrl::from_fragment("key1".to_string())?;

    let service: Service = ServiceDidCommV1::new(
        Uri::new("#0")?,
        service_endpoint,
        0,
        vec![ServiceKeyKind::Reference(vm_ka_id.clone())],
        routing_keys
            .into_iter()
            .map(ServiceKeyKind::Value)
            .collect(),
    )
    .try_into()?;

    info!("Prepared service for peer:did:4 generation: {} ", service);
    let vm_ka = DidPeer4VerificationMethod::builder()
        .id(vm_ka_id.clone())
        .verification_method_type(VerificationMethodType::Ed25519VerificationKey2020)
        .public_key(PublicKeyField::Multibase {
            public_key_multibase: key_enc.fingerprint(),
        })
        .build();
    let mut construction_did_doc = DidPeer4ConstructionDidDocument::new();
    construction_did_doc.add_verification_method(vm_ka);
    construction_did_doc.add_key_agreement_ref(vm_ka_id);

    construction_did_doc.add_service(service);

    info!(
        "Created did document for peer:did:4 generation: {} ",
        construction_did_doc
    );
    let peer_did = PeerDid::<Numalgo4>::new(construction_did_doc)?;
    info!("Created peer did: {peer_did}");

    Ok((peer_did, key_enc))
}

pub(crate) fn ddo_to_attach(ddo: DidDocument) -> Result<Attachment, AriesVcxError> {
    // Interop note: acapy accepts unsigned when using peer dids?
    let content_b64 =
        base64::engine::Engine::encode(&URL_SAFE_LENIENT, serde_json::to_string(&ddo)?);
    Ok(Attachment::builder()
        .data(
            AttachmentData::builder()
                .content(AttachmentType::Base64(content_b64))
                .build(),
        )
        .build())
}

pub(crate) fn assemble_did_rotate_attachment(did: &Did) -> Attachment {
    let content_b64 = base64::engine::Engine::encode(&URL_SAFE_LENIENT, did.id());
    Attachment::builder()
        .data(
            AttachmentData::builder()
                .content(AttachmentType::Base64(content_b64))
                .build(),
        )
        .build()
}

// TODO: Obviously, extract attachment signing
// TODO: JWS verification
pub(crate) async fn jws_sign_attach(
    mut attach: Attachment,
    verkey: Key,
    wallet: &impl BaseWallet,
) -> Result<Attachment, AriesVcxError> {
    if let AttachmentType::Base64(attach_base64) = &attach.data.content {
        let did_key: DidKey = verkey.clone().try_into()?;
        let verkey_b64 = base64::engine::Engine::encode(&URL_SAFE_LENIENT, verkey.key());

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
            base64::engine::Engine::encode(&URL_SAFE_LENIENT, protected_header.to_string());
        let sign_input = format!("{}.{}", b64_protected, attach_base64).into_bytes();
        let signed = wallet.sign(&verkey, &sign_input).await?;
        let signature_base64 = base64::engine::Engine::encode(&URL_SAFE_LENIENT, signed);

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

// TODO - ideally this should be resilient to the case where the attachment is a legacy aries DIDDoc
// structure (i.e. [diddoc_legacy::aries::diddoc::AriesDidDoc]). It should be converted to a
// spec-compliant [DidDocument]. ACA-py handles this case here: https://github.com/hyperledger/aries-cloudagent-python/blob/5ad52c15d2f4f62db1678b22a7470776d78b36f5/aries_cloudagent/resolver/default/legacy_peer.py#L27
// https://github.com/hyperledger/aries-vcx/issues/1227
pub(crate) fn attachment_to_diddoc(attachment: Attachment) -> Result<DidDocument, AriesVcxError> {
    match attachment.data.content {
        AttachmentType::Json(value) => serde_json::from_value(value).map_err(Into::into),
        AttachmentType::Base64(ref value) => {
            let bytes = base64::Engine::decode(&URL_SAFE_LENIENT, value).map_err(|err| {
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

pub(crate) fn to_transition_error<S, T>(state: S) -> impl FnOnce(T) -> TransitionError<S>
where
    T: Into<AriesVcxError>,
{
    move |error| TransitionError {
        error: error.into(),
        state,
    }
}
