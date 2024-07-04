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
    let content_b64 = base64::engine::Engine::encode(&URL_SAFE_LENIENT, did.did());
    Attachment::builder()
        .data(
            AttachmentData::builder()
                .content(AttachmentType::Base64(content_b64))
                .build(),
        )
        .build()
}

// TODO: if this becomes a common method, move to a shared location.
/// Creates a JWS signature of the attachment with the provided verkey. The created JWS
/// signature is appended to the attachment, in alignment with Aries RFC 0017:
/// https://hyperledger.github.io/aries-rfcs/latest/concepts/0017-attachments/#signing-attachments.
pub(crate) async fn jws_sign_attach(
    mut attach: Attachment,
    verkey: Key,
    wallet: &impl BaseWallet,
) -> Result<Attachment, AriesVcxError> {
    let AttachmentType::Base64(attach_base64) = &attach.data.content else {
        return Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidInput,
            "Cannot sign non-base64-encoded attachment",
        ));
    };
    if verkey.key_type() != &KeyType::Ed25519 {
        return Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidVerkey,
            "Only JWS signatures with Ed25519 based keys are currently supported.",
        ));
    }

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
        "kid": did_key.to_string(),
    });
    let b64_protected =
        base64::engine::Engine::encode(&URL_SAFE_LENIENT, protected_header.to_string());
    let sign_input = format!("{}.{}", b64_protected, attach_base64).into_bytes();
    let signed: Vec<u8> = wallet.sign(&verkey, &sign_input).await?;
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
}

/// Verifies that the given has a JWS signature attached, which is a valid signature given
/// the expected signer key.
// NOTE: Does not handle attachments with multiple signatures.
// NOTE: this is the specific use case where the signer is known by the function caller. Therefore
// we do not need to attempt to decode key within the protected nor unprotected header.
pub(crate) async fn jws_verify_attachment(
    attach: &Attachment,
    expected_signer: &Key,
    wallet: &impl BaseWallet,
) -> Result<bool, AriesVcxError> {
    let AttachmentType::Base64(attach_base64) = &attach.data.content else {
        return Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidInput,
            "Cannot verify JWS of a non-base64-encoded attachment",
        ));
    };
    // aries attachments do not REQUIRE that the attachment has no padding,
    // but JWS does, so remove it; just incase.
    let attach_base64 = attach_base64.replace('=', "");

    let Some(ref jws) = attach.data.jws else {
        return Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidInput,
            "Attachment has no JWS signature attached. Cannot verify.",
        ));
    };

    let (Some(b64_protected), Some(b64_signature)) = (
        jws.get("protected").and_then(|s| s.as_str()),
        jws.get("signature").and_then(|s| s.as_str()),
    ) else {
        return Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidInput,
            "Attachment has an invalid JWS with missing fields. Cannot verify.",
        ));
    };

    let sign_input = format!("{}.{}", b64_protected, attach_base64).into_bytes();
    let signature =
        base64::engine::Engine::decode(&URL_SAFE_LENIENT, b64_signature).map_err(|_| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::EncodeError,
                "Attachment JWS signature was not correctly base64Url encoded.",
            )
        })?;

    let res = wallet
        .verify(expected_signer, &sign_input, &signature)
        .await?;

    Ok(res)
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

#[cfg(test)]
mod tests {
    use std::error::Error;

    use aries_vcx_wallet::wallet::base_wallet::did_wallet::DidWallet;
    use messages::decorators::attachment::{Attachment, AttachmentData, AttachmentType};
    use public_key::Key;
    use test_utils::devsetup::build_setup_profile;

    use crate::{
        protocols::did_exchange::state_machine::helpers::{jws_sign_attach, jws_verify_attachment},
        utils::base64::URL_SAFE_LENIENT,
    };

    // assert self fulfilling
    #[tokio::test]
    async fn test_jws_sign_and_verify_attachment() -> Result<(), Box<dyn Error>> {
        let setup = build_setup_profile().await;
        let wallet = &setup.wallet;
        let signer_did = wallet.create_and_store_my_did(None, None).await?;
        let signer = signer_did.verkey();

        let content_b64 = base64::engine::Engine::encode(&URL_SAFE_LENIENT, "hello world");
        let attach = Attachment::builder()
            .data(
                AttachmentData::builder()
                    .content(AttachmentType::Base64(content_b64))
                    .build(),
            )
            .build();

        let signed_attach = jws_sign_attach(attach, signer.clone(), wallet).await?;

        // should contain signed JWS
        assert_eq!(signed_attach.data.jws.as_ref().unwrap().len(), 3);

        // verify
        assert!(jws_verify_attachment(&signed_attach, signer, wallet).await?);

        // verify with wrong key should be false
        let wrong_did = wallet.create_and_store_my_did(None, None).await?;
        let wrong_signer = wrong_did.verkey();
        assert!(!jws_verify_attachment(&signed_attach, wrong_signer, wallet).await?);

        Ok(())
    }

    // test vector taken from an ACApy 0.12.1 DIDExchange response
    #[tokio::test]
    async fn test_jws_verify_attachment_with_acapy_test_vector() -> Result<(), Box<dyn Error>> {
        let setup = build_setup_profile().await;
        let wallet = &setup.wallet;

        let json = json!({
          "@id": "18bec73c-c621-4ef2-b3d8-085c59ac9e2b",
          "mime-type": "text/string",
          "data": {
            "jws": {
              "signature": "QxC2oLxAYav-fPOvjkn4OpMLng9qOo2fjsy0MoQotDgyVM_PRjYlatsrw6_rADpRpWR_GMpBVlBskuKxpsJIBQ",
              "header": {
                "kid": "did:key:z6MkpNusbzt7HSBwrBiRpZmbyLiBEsNGs2fotoYhykU8Muaz"
              },
              "protected": "eyJhbGciOiAiRWREU0EiLCAiandrIjogeyJrdHkiOiAiT0tQIiwgImNydiI6ICJFZDI1NTE5IiwgIngiOiAiazNlOHZRTHpSZlFhZFhzVDBMUkMxMWhpX09LUlR6VFphd29ocmxhaW1ETSIsICJraWQiOiAiZGlkOmtleTp6Nk1rcE51c2J6dDdIU0J3ckJpUnBabWJ5TGlCRXNOR3MyZm90b1loeWtVOE11YXoifX0"
            },
            // NOTE: includes b64 padding, but not recommended
            "base64": "ZGlkOnBlZXI6NHpRbVhza2o1Sjc3NXRyWUpkaVVFZVlaUU5mYXZZQUREb25YMzJUOHF4VHJiU05oOno2MmY5VlFROER0N1VWRXJXcmp6YTd4MUVKOG50NWVxOWlaZk1BUGoyYnpyeGJycGY4VXdUTEpXVUJTV2U4dHNoRFl4ZDhlcmVSclRhOHRqVlhKNmNEOTV0Qml5dVdRVll6QzNtZWtUckJ4MzNjeXFCb2g0c3JGamdXZm1lcE5yOEZpRFI5aEoySExxMlM3VGZNWXIxNVN4UG52OExRR2lIV24zODhzVlF3ODRURVJFaTg4OXlUejZzeVVmRXhEaXdxWHZOTk05akt1eHc4NERvbmtVUDRHYkh0Q3B4R2hKYVBKWnlUWmJVaFF2SHBENGc2YzYyWTN5ZGQ0V1BQdXBYQVFISzJScFZod2hQWlVnQWQzN1lrcW1jb3FiWGFZTWFnekZZY3kxTEJ6NkdYekV5NjRrOGQ4WGhlem5vUkpIV3F4RTV1am5LYkpOM0pRR241UzREaEtRaXJTbUZINUJOYUNvRTZqaFlWc3gzWlpEM1ZWZVVxUW9ZMmVHMkNRVVRRak1zY0ozOEdqeDFiaVVlRkhZVVRrejRRVDJFWXpXRlVEbW1URHExVmVoZExtelJDWnNQUjJKR1VpVExUVkNzdUNzZ21jd1FqWHY4WmN6ejRaZUo0ODc4S3hBRm5mam1ibk1EejV5NVJOMnZtRGtkaE42dFFMZjJEWVJuSm1vSjJ5VTNheXczU2NjV0VMVzNpWEN6UFROV1F3WmFEb2d5UFVXZFBobkw0OEVpMjI2cnRBcWoySGQxcTRua1Fwb0ZWQ1B3aXJGUmtub05Zc2NGV1dxN1JEVGVMcmlKcENrUVVFblh4WVBpU1F5S0RxbVpFN0FRVjI="
          }
        });
        let mut attach: Attachment = serde_json::from_value(json)?;
        let signer = Key::from_fingerprint("z6MkpNusbzt7HSBwrBiRpZmbyLiBEsNGs2fotoYhykU8Muaz")?;

        // should verify with correct signer
        assert!(jws_verify_attachment(&attach, &signer, wallet).await?);

        // should not verify with wrong signer
        let wrong_signer =
            Key::from_fingerprint("z6Mkva1JM9mM3SMuLCtVDAXzAQTwkdtfzHXSYMKtfXK2cPye")?;
        assert!(!jws_verify_attachment(&attach, &wrong_signer, wallet).await?);

        // should not verify if wrong signature
        attach.data.content = AttachmentType::Base64(String::from("d3JvbmcgZGF0YQ=="));
        assert!(!jws_verify_attachment(&attach, &signer, wallet).await?);

        Ok(())
    }
}
