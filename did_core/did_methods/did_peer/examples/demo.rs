use std::{collections::HashMap, error::Error};

use did_doc::schema::{
    did_doc::DidDocument,
    service::{typed::ServiceType, Service},
    types::uri::Uri,
    utils::OneOrList,
    verification_method::{VerificationMethod, VerificationMethodType},
};
use did_parser_nom::{Did, DidUrl};
use did_peer::{
    peer_did::{
        numalgos::{
            numalgo2::Numalgo2,
            numalgo3::Numalgo3,
            numalgo4::{encoded_document::DidPeer4EncodedDocumentBuilder, Numalgo4},
        },
        PeerDid,
    },
    resolver::{options::PublicKeyEncoding, PeerDidResolutionOptions, PeerDidResolver},
};
use did_resolver::traits::resolvable::{resolution_output::DidResolutionOutput, DidResolvable};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    demo().await
}

async fn demo_did_peer_2_and_3() -> Result<(), Box<dyn Error>> {
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
        .build();
    println!("Did document: \n{}", serde_json::to_string_pretty(&ddo)?);

    let peer_did_2 = PeerDid::<Numalgo2>::from_did_doc(ddo.clone())?;
    println!("as did:peer numalgo(2): {}", peer_did_2);

    let peer_did_3 = PeerDid::<Numalgo3>::from_did_doc(ddo)?;
    println!("as did:peer numalgo(3): {}", peer_did_3);

    let peer_did_3_v2 = peer_did_2.to_numalgo3()?;
    println!(
        "as did:peer numalgo(2) converted to numalgo(3): {}",
        peer_did_3_v2
    );

    let DidResolutionOutput { did_document, .. } = PeerDidResolver::new()
        .resolve(
            peer_did_2.did(),
            &PeerDidResolutionOptions {
                encoding: Some(PublicKeyEncoding::Base58),
            },
        )
        .await
        .unwrap();
    println!(
        "Decoded did document: \n{}",
        serde_json::to_string_pretty(&did_document)?
    );
    Ok(())
}

async fn demo_did_peer_4() -> Result<(), Box<dyn Error>> {
    let service = Service::new(
        Uri::new("#service-0").unwrap(),
        "https://example.com/endpoint".parse().unwrap(),
        OneOrList::One(ServiceType::DIDCommV2),
        HashMap::default(),
    );
    let encoded_document = DidPeer4EncodedDocumentBuilder::default()
        .service(vec![service])
        .build()
        .unwrap();
    println!(
        "Pseudo DidDocument, input for did:peer:4 construction: {}",
        serde_json::to_string_pretty(&encoded_document)?
    );

    let peer_did_4 = PeerDid::<Numalgo4>::new(encoded_document)?;
    println!("as did:peer numalgo(4): {}", peer_did_4);

    let DidResolutionOutput { did_document, .. } = PeerDidResolver::new()
        .resolve(
            peer_did_4.did(),
            &PeerDidResolutionOptions {
                encoding: Some(PublicKeyEncoding::Base58),
            },
        )
        .await
        .unwrap();
    println!(
        "Decoded did document: \n{}",
        serde_json::to_string_pretty(&did_document)?
    );
    Ok(())
}

async fn demo() -> Result<(), Box<dyn Error>> {
    let env = env_logger::Env::default().default_filter_or("info");
    env_logger::init_from_env(env);

    demo_did_peer_2_and_3().await?;
    demo_did_peer_4().await?;

    Ok(())
}

#[tokio::test]
async fn demo_test() -> Result<(), Box<dyn Error>> {
    demo().await
}
