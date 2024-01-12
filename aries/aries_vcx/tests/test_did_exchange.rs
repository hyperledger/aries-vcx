extern crate log;

use std::sync::Arc;

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
use aries_vcx_core::test_utils::devsetup::SetupPoolDirectory;
use did_doc_sov::{
    extra_fields::{didcommv1::ExtraFieldsDidCommV1, KeyKind},
    service::{didcommv1::ServiceDidCommV1, ServiceSov},
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
use url::Url;
use uuid::Uuid;

use crate::utils::test_agent::{create_test_agent, create_test_agent_trustee};

pub mod utils;

#[tokio::test]
#[ignore]
async fn did_exchange_test() {
    let setup = SetupPoolDirectory::init().await;
    let institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
    let consumer = create_test_agent(setup.genesis_file_path).await;

    let did_peer_resolver = PeerDidResolver::new();
    let resolver_registry = Arc::new(
        ResolverRegistry::new()
            .register_resolver::<PeerDidResolver>("peer".into(), did_peer_resolver),
    );

    let url: Url = "http://dummyurl.org".parse().unwrap();

    let public_key = generate_keypair(&institution.wallet, KeyType::Ed25519)
        .await
        .unwrap();
    let service = {
        let service_id = Uuid::new_v4().to_string();
        ServiceSov::DIDCommV1(
            ServiceDidCommV1::new(
                service_id.parse().unwrap(),
                url.clone().into(),
                ExtraFieldsDidCommV1::builder()
                    .set_recipient_keys(vec![KeyKind::DidKey(public_key.try_into().unwrap())])
                    .build(),
            )
            .unwrap(),
        )
    };
    let invitation = OutOfBandSender::create()
        .append_service(&OobService::SovService(service))
        .append_handshake_protocol(Protocol::DidExchangeType(DidExchangeType::V1(
            DidExchangeTypeV1::new_v1_0(),
        )))
        .unwrap()
        .oob
        .clone();

    let invitation_id = invitation.id.clone();
    let invitation_key = resolve_key_from_invitation(&invitation, &resolver_registry)
        .await
        .unwrap();

    let TransitionResult {
        state: requester,
        output: request,
    } = DidExchangeRequester::<RequestSent>::construct_request_pairwise(
        &consumer.wallet,
        invitation,
        resolver_registry.clone(),
        url.clone(),
        vec![],
    )
    .await
    .unwrap();

    let TransitionResult {
        state: responder,
        output: response,
    } = DidExchangeResponder::<ResponseSent>::receive_request(
        &institution.wallet,
        resolver_registry,
        request,
        url.clone(),
        vec![],
        invitation_id,
        invitation_key,
    )
    .await
    .unwrap();

    let TransitionResult {
        state: requester,
        output: complete,
    } = requester.receive_response(response).await.unwrap();

    let responder = responder.receive_complete(complete).unwrap();

    let responder_key = responder
        .our_did_doc()
        .verification_method()
        .first()
        .unwrap()
        .public_key()
        .unwrap()
        .base58();
    assert_eq!(
        requester
            .their_did_doc()
            .verification_method()
            .first()
            .unwrap()
            .public_key()
            .unwrap()
            .base58(),
        responder_key
    );

    let requester_key = requester
        .our_did_doc()
        .verification_method()
        .first()
        .unwrap()
        .public_key()
        .unwrap()
        .base58();
    assert_eq!(
        responder
            .their_did_doc()
            .verification_method()
            .first()
            .unwrap()
            .public_key()
            .unwrap()
            .base58(),
        requester_key
    );
}
