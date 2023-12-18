use std::error::Error;

use did_doc::schema::{
    did_doc::DidDocument,
    verification_method::{VerificationMethod, VerificationMethodType},
};
use did_parser::{Did, DidUrl};
use did_peer::peer_did::{
    numalgos::{numalgo2::Numalgo2, numalgo3::Numalgo3},
    PeerDid,
};

fn main() -> Result<(), Box<dyn Error>> {
    demo()
}

fn demo() -> Result<(), Box<dyn Error>> {
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

    let decoded_did_doc = peer_did_2
        .to_did_doc_builder(did_peer::resolver::options::PublicKeyEncoding::Base58)?
        .build();
    println!(
        "Decoded did document: \n{}",
        serde_json::to_string_pretty(&decoded_did_doc)?
    );

    Ok(())
}

#[test]
fn demo_test() -> Result<(), Box<dyn Error>> {
    demo()
}
