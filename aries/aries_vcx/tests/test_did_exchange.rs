extern crate log;

use std::{error::Error, sync::Arc, thread, time::Duration};

use aries_vcx::{
    common::ledger::{
        service_didsov::{DidSovServiceType, EndpointDidSov},
        transactions::write_endpoint,
    },
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
};
use aries_vcx_core::ledger::indy_vdr_ledger::DefaultIndyLedgerRead;
use did_parser::Did;
use did_peer::{
    peer_did::{numalgos::numalgo2::Numalgo2, PeerDid},
    resolver::PeerDidResolver,
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

#[tokio::test]
#[ignore]
async fn did_exchange_test() -> Result<(), Box<dyn Error>> {
    let setup = SetupPoolDirectory::init().await;
    let dummy_url: Url = "http://dummyurl.org".parse().unwrap();
    let agent_trustee = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
    // todo: patrik: update create_test_agent_endorser_2 to not consume trustee agent
    let agent_inviter =
        create_test_agent_endorser_2(&setup.genesis_file_path, agent_trustee).await?;
    // todo: patrik: what does it take to get rid of this custom aries-vcx struct?
    let create_service = EndpointDidSov::create()
        .set_service_endpoint("https://example.org".parse()?)
        .set_types(Some(vec![
            DidSovServiceType::DidCommunication,
            DidSovServiceType::Endpoint,
        ]));
    write_endpoint(
        &agent_inviter.wallet,
        &agent_inviter.ledger_write,
        &agent_inviter.institution_did,
        &create_service,
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

    let (our_did_document, _our_verkey) =
        create_our_did_document(&agent_invitee.wallet, dummy_url.clone(), vec![]).await?;
    info!("Inviter prepares did document: {our_did_document}");
    let peer_did_invitee = PeerDid::<Numalgo2>::from_did_doc(our_did_document.clone())?;
    info!("Invitee prepares their peer:did: {peer_did_invitee}");
    let did_inviter: Did = invitation_get_first_did_service(&invitation)?;

    let TransitionResult {
        output: request, ..
    } = DidExchangeRequester::<RequestSent>::construct_request(
        resolver_registry.clone(),
        &did_inviter,
        &peer_did_invitee,
    )
    .await
    .unwrap();
    info!(
        "Invitee processes invitation, builds up request {}",
        &request
    );

    let (our_did_document, _our_verkey) =
        create_our_did_document(&agent_invitee.wallet, dummy_url.clone(), vec![]).await?;
    let peer_did_inviter = PeerDid::<Numalgo2>::from_did_doc(our_did_document.clone())?;
    info!("Inviter prepares their peer:did: {peer_did_inviter}");

    let TransitionResult {
        output: _response, ..
    } = DidExchangeResponder::<ResponseSent>::receive_request(
        &agent_inviter.wallet,
        resolver_registry,
        request,
        &peer_did_inviter,
        invitation_key,
    )
    .await
    .unwrap();
    // todo: patrik: enable rest of the test
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
    Ok(())
}
