extern crate log;

use std::sync::Arc;
use log::info;

use aries_vcx::{
    handlers::out_of_band::sender::OutOfBandSender,
    protocols::did_exchange::{
        resolve_key_from_invitation,
        state_machine::{
            generate_keypair, requester::DidExchangeRequester, responder::DidExchangeResponder,
        },
        states::{requester::request_sent::RequestSent, responder::response_sent::ResponseSent},
        transition::transition_result::TransitionResult,
    },
};
use did_doc_sov::{
    extra_fields::{didcommv1::ExtraFieldsDidCommV1, SovKeyKind},
    service::{didcommv1::ServiceDidCommV1},
};
use did_peer::resolver::PeerDidResolver;
use did_resolver_registry::ResolverRegistry;
use messages::{
    msg_fields::protocols::out_of_band::invitation::OobService,
    msg_types::{
        protocols::did_exchange::{DidExchangeType, DidExchangeTypeV1},
        Protocol,
    },
};
use public_key::KeyType;
use test_utils::devsetup::SetupPoolDirectory;
use url::Url;
use uuid::Uuid;
use did_doc::schema::did_doc::DidDocument;
use did_doc::schema::verification_method::{VerificationMethod, VerificationMethodType};
use did_parser::Did;
use did_peer::peer_did::numalgos::numalgo2::Numalgo2;
use did_peer::peer_did::PeerDid;
use messages::msg_fields::protocols::out_of_band::invitation::{Invitation, InvitationContent};

use crate::utils::test_agent::{create_test_agent, create_test_agent_trustee};

pub mod utils;

fn prepare_peer_did() {
    let did = Did::parse("did:peer:abc0123456789".into())?;
    let verification_method = VerificationMethod::builder(
        did.into(),
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

}

#[tokio::test]
#[ignore]
async fn did_exchange_test() {
    let dummy_url: Url = "http://dummyurl.org".parse().unwrap();
    let setup = SetupPoolDirectory::init().await;
    let institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
    let consumer = create_test_agent(setup.genesis_file_path).await;

    let did_peer_resolver = PeerDidResolver::new();
    let resolver_registry = Arc::new(
        ResolverRegistry::new()
            .register_resolver::<PeerDidResolver>("peer".into(), did_peer_resolver),
    );

    let invitation = Invitation::builder()
        .id("test_invite_id".to_owned())
        .content(InvitationContent::builder()
            .service(OobService::Did(format!("did:sov:{}", institution.institution_did)))
            .build()
        ).build();
    info!("Institution prepares invitation: {:?} and passes to invitee", invitation);


    let TransitionResult {
        state: requester,
        output: request,
    } = DidExchangeRequester::<RequestSent>::construct_request(
        resolver_registry.clone(),
        their_did.clone(),
        our_did.clone()
    )
    .await
    .unwrap();
    info!("Invitee processes invitation, builds up request {}", &request);

    let invitation_key = resolve_key_from_invitation(&invitation, &resolver_registry)
        .await
        .unwrap();

    //
    // let TransitionResult {
    //     state: responder,
    //     output: response,
    // } = DidExchangeResponder::<ResponseSent>::receive_request(
    //     &institution.wallet,
    //     resolver_registry,
    //     request,
    //     url.clone(),
    //     vec![],
    //     invitation_id,
    //     invitation_key,
    // )
    // .await
    // .unwrap();
    //
    // let TransitionResult {
    //     state: requester,
    //     output: complete,
    // } = requester.receive_response(response).await.unwrap();
    //
    // let responder = responder.receive_complete(complete).unwrap();
    //
    // let responder_key = responder
    //     .our_did_doc()
    //     .verification_method()
    //     .first()
    //     .unwrap()
    //     .public_key()
    //     .unwrap()
    //     .base58();
    // assert_eq!(
    //     requester
    //         .their_did_doc()
    //         .verification_method()
    //         .first()
    //         .unwrap()
    //         .public_key()
    //         .unwrap()
    //         .base58(),
    //     responder_key
    // );
    //
    // let requester_key = requester
    //     .our_did_doc()
    //     .verification_method()
    //     .first()
    //     .unwrap()
    //     .public_key()
    //     .unwrap()
    //     .base58();
    // assert_eq!(
    //     responder
    //         .their_did_doc()
    //         .verification_method()
    //         .first()
    //         .unwrap()
    //         .public_key()
    //         .unwrap()
    //         .base58(),
    //     requester_key
    // );
}
