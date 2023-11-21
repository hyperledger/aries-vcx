use std::error::Error;

use did_doc::schema::{
    did_doc::DidDocument,
    service::ServiceBuilder,
    types::{uri::Uri, url::Url},
    verification_method::{VerificationMethod, VerificationMethodType},
};
use did_doc_sov::extra_fields::{didcommv1::ExtraFieldsDidCommV1, ExtraFieldsSov, KeyKind};
use did_parser::{Did, DidUrl};
use did_peer::peer_did::{
    numalgos::{numalgo2::Numalgo2, numalgo3::Numalgo3},
    PeerDid,
};

#[test]
fn demo() -> Result<(), Box<dyn Error>> {
    let recipient_key = KeyKind::Value("foo".to_string());
    let sov_service_extra = ExtraFieldsSov::DIDCommV1(
        ExtraFieldsDidCommV1::builder()
            .set_recipient_keys(vec![recipient_key])
            .build(),
    );
    let service = ServiceBuilder::<ExtraFieldsSov>::new(
        Uri::new("xyz://example.org")?,
        Url::new("http://example.org")?,
        sov_service_extra,
    )
    .add_service_type("DIDCommMessaging".to_string())?
    .build();

    let did_url = DidUrl::parse("did:foo:bar#key-1".into())?;
    let did = Did::parse("did:foo:bar".into())?;
    let verification_method = VerificationMethod::builder(
        did_url,
        did.clone(),
        VerificationMethodType::Ed25519VerificationKey2018,
    )
    .add_public_key_base64("Zm9vYmFyCg".to_string())
    .build();

    let ddo = DidDocument::builder(did)
        .add_verification_method(verification_method)
        .add_service(service)
        .build();
    println!("diddoc: {}", ddo);

    let peer_did_2 = PeerDid::<Numalgo2>::from_did_doc(ddo.clone())?;
    println!("PeerDid numalgo(2): {}", peer_did_2);
    let peer_did_3 = PeerDid::<Numalgo3>::from_did_doc(ddo)?;
    println!("PeerDid numalgo(3): {}", peer_did_3);
    let peer_did_3_v2 = peer_did_2.to_numalgo3()?;
    println!("Converted PeerDid numalgo(3): {}", peer_did_3_v2);

    Ok(())
}
