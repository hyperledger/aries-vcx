extern crate log;

use std::{error::Error, sync::Arc, thread, time::Duration};

use aries_vcx::{
    common::ledger::transactions::write_endpoint_from_service,
    protocols::did_exchange::{
        resolve_enc_key_from_invitation,
        state_machine::{
            create_our_did_document,
            requester::{helpers::invitation_get_first_did_service, DidExchangeRequester},
            responder::DidExchangeResponder,
        },
        states::{requester::request_sent::RequestSent, responder::response_sent::ResponseSent},
        transition::transition_result::TransitionResult,
    },
    utils::{didcomm_utils::resolve_base58_key_agreement, encryption_envelope::EncryptionEnvelope},
};
use aries_vcx_core::ledger::indy_vdr_ledger::DefaultIndyLedgerRead;
use did_doc::schema::{
    did_doc::DidDocument, service::typed::didcommv1::ServiceDidCommV1, types::uri::Uri,
};
use did_parser::Did;
use did_peer::{
    peer_did::{numalgos::numalgo2::Numalgo2, PeerDid},
    resolver::{options::PublicKeyEncoding, PeerDidResolver},
};
use did_resolver_registry::ResolverRegistry;
use did_resolver_sov::resolution::DidSovResolver;
use log::info;
use messages::msg_fields::protocols::out_of_band::invitation::{
    Invitation, InvitationContent, OobService,
};
use test_utils::devsetup::{dev_build_profile_vdr_ledger, SetupPoolDirectory};
use url::Url;

use crate::utils::test_agent::{
    create_test_agent, create_test_agent_endorser_2, create_test_agent_trustee,
};

pub mod utils;

fn assert_key_agreement(a: DidDocument, b: DidDocument) {
    let a_key = resolve_base58_key_agreement(&a).unwrap();
    let b_key = resolve_base58_key_agreement(&b).unwrap();
    assert_eq!(a_key, b_key);
}

#[tokio::test]
#[ignore]
async fn did_exchange_test() -> Result<(), Box<dyn Error>> {
    let setup = SetupPoolDirectory::init().await;
    let dummy_url: Url = "http://dummyurl.org".parse().unwrap();
    let agent_trustee = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
    // todo: patrik: update create_test_agent_endorser_2 to not consume trustee agent
    let agent_inviter =
        create_test_agent_endorser_2(&setup.genesis_file_path, agent_trustee).await?;
    let create_service = ServiceDidCommV1::new(
        Uri::new("#service-0").unwrap(),
        dummy_url.clone(),
        0,
        vec![],
        vec![],
    );
    write_endpoint_from_service(
        &agent_inviter.wallet,
        &agent_inviter.ledger_write,
        &agent_inviter.institution_did,
        &create_service.try_into()?,
    )
    .await?;
    thread::sleep(Duration::from_millis(100));

    let agent_invitee = create_test_agent(setup.genesis_file_path.clone()).await;

    let (ledger_read_2, _) = dev_build_profile_vdr_ledger(setup.genesis_file_path);
    let ledger_read_2_arc = Arc::new(ledger_read_2);

    // if we were to use, more generally, the `dev_build_featured_indy_ledger`, we would need to
    // here the type based on the feature flag (indy vs proxy vdr client) which is pain
    // we need to improve DidSovResolver such that Rust compiler can fully infer the return type
    let did_sov_resolver: DidSovResolver<Arc<DefaultIndyLedgerRead>, DefaultIndyLedgerRead> =
        DidSovResolver::new(ledger_read_2_arc);

    let resolver_registry = Arc::new(
        ResolverRegistry::new()
            .register_resolver::<PeerDidResolver>("peer".into(), PeerDidResolver::new())
            .register_resolver("sov".into(), did_sov_resolver),
    );

    let invitation = Invitation::builder()
        .id("test_invite_id".to_owned())
        .content(
            InvitationContent::builder()
                .services(vec![OobService::Did(format!(
                    "did:sov:{}",
                    agent_inviter.institution_did
                ))])
                .build(),
        )
        .build();
    let invitation_key = resolve_enc_key_from_invitation(&invitation, &resolver_registry)
        .await
        .unwrap();
    info!(
        "Inviter prepares invitation and passes to invitee {}",
        invitation
    );

    let (requesters_did_document, _our_verkey) =
        create_our_did_document(&agent_invitee.wallet, dummy_url.clone(), vec![]).await?;
    info!("Requester prepares did document: {requesters_did_document}");
    let requesters_peer_did = PeerDid::<Numalgo2>::from_did_doc(requesters_did_document.clone())?;
    info!("Requester prepares their peer:did: {requesters_peer_did}");
    let did_inviter: Did = invitation_get_first_did_service(&invitation)?;

    let TransitionResult {
        state: requester,
        output: request,
    } = DidExchangeRequester::<RequestSent>::construct_request(
        resolver_registry.clone(),
        &did_inviter,
        &requesters_peer_did,
    )
    .await
    .unwrap();
    info!(
        "Invitee processes invitation, builds up request {}",
        &request
    );

    let (responders_did_document, _our_verkey) =
        create_our_did_document(&agent_invitee.wallet, dummy_url.clone(), vec![]).await?;
    info!("Responder prepares did document: {responders_did_document}");
    let responders_peer_did = PeerDid::<Numalgo2>::from_did_doc(responders_did_document.clone())?;
    info!("Responder prepares their peer:did: {responders_peer_did}");

    let check_diddoc: DidDocument = responders_peer_did.to_did_doc(PublicKeyEncoding::Base58)?;
    info!("Responder decodes constructed peer:did as did document: {check_diddoc}");

    let TransitionResult {
        output: response,
        state: responder,
    } = DidExchangeResponder::<ResponseSent>::receive_request(
        &agent_inviter.wallet,
        resolver_registry,
        request,
        &responders_peer_did,
        invitation_key,
    )
    .await
    .unwrap();

    let TransitionResult {
        state: requester,
        output: complete,
    } = requester.receive_response(response).await.unwrap();

    let responder = responder.receive_complete(complete).unwrap();

    info!("Asserting did document of requester");
    assert_key_agreement(
        requester.our_did_doc().clone(),
        responder.their_did_doc().clone(),
    );
    info!("Asserting did document of responder");
    assert_key_agreement(
        responder.our_did_doc().clone(),
        requester.their_did_doc().clone(),
    );

    info!(
        "Requesters did document (requesters view): {}",
        requester.our_did_doc()
    );
    info!(
        "Responders did document (requesters view): {}",
        requester.their_did_doc()
    );

    let data = "Hello world";
    let m = EncryptionEnvelope::create(
        &agent_invitee.wallet,
        data.as_bytes(),
        requester.our_did_doc(),
        requester.their_did_doc(),
    )
    .await?;

    info!("Encrypted message: {:?}", m);

    let expected_sender_vk = resolve_base58_key_agreement(&requesters_did_document)?;
    let unpacked =
        EncryptionEnvelope::auth_unpack(&agent_invitee.wallet, m.0, &expected_sender_vk).await?;

    info!("Unpacked message: {:?}", unpacked);

    Ok(())
}
