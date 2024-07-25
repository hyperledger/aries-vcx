extern crate log;

use std::{error::Error, sync::Arc, thread, time::Duration};

use aries_vcx::{
    common::ledger::transactions::write_endpoint_from_service,
    errors::error::AriesVcxErrorKind,
    protocols::did_exchange::{
        resolve_enc_key_from_invitation,
        state_machine::{
            helpers::create_peer_did_4,
            requester::{helpers::invitation_get_first_did_service, DidExchangeRequester},
            responder::DidExchangeResponder,
        },
        states::{requester::request_sent::RequestSent, responder::response_sent::ResponseSent},
        transition::transition_result::TransitionResult,
    },
    utils::{
        didcomm_utils::resolve_ed25519_base58_key_agreement,
        encryption_envelope::EncryptionEnvelope,
    },
};
use aries_vcx_anoncreds::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_ledger::ledger::{
    base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite},
    indy_vdr_ledger::DefaultIndyLedgerRead,
};
use aries_vcx_wallet::wallet::base_wallet::BaseWallet;
use did_doc::schema::{
    did_doc::DidDocument,
    service::typed::{didcommv1::ServiceDidCommV1, ServiceType},
    types::uri::Uri,
};
use did_parser_nom::Did;
use did_peer::resolver::PeerDidResolver;
use did_resolver_registry::ResolverRegistry;
use did_resolver_sov::resolution::DidSovResolver;
use log::info;
use messages::{
    msg_fields::protocols::out_of_band::invitation::{Invitation, InvitationContent, OobService},
    msg_types::protocols::did_exchange::DidExchangeTypeV1,
};
use pretty_assertions::assert_eq;
use test_utils::devsetup::{dev_build_profile_vdr_ledger, SetupPoolDirectory};
use url::Url;
use utils::test_agent::TestAgent;

use crate::utils::test_agent::{
    create_test_agent, create_test_agent_endorser_2, create_test_agent_trustee,
};

pub mod utils;

fn assert_key_agreement(a: DidDocument, b: DidDocument) {
    log::warn!("comparing did doc a: {}, b: {}", a, b);
    let a_key = resolve_ed25519_base58_key_agreement(&a).unwrap();
    let b_key = resolve_ed25519_base58_key_agreement(&b).unwrap();
    assert_eq!(a_key, b_key);
}

async fn did_exchange_test(
    inviter_did: String,
    agent_inviter: TestAgent<
        impl IndyLedgerRead + AnoncredsLedgerRead,
        impl IndyLedgerWrite + AnoncredsLedgerWrite,
        impl BaseAnonCreds,
        impl BaseWallet,
    >,
    agent_invitee: TestAgent<
        impl IndyLedgerRead + AnoncredsLedgerRead,
        impl IndyLedgerWrite + AnoncredsLedgerWrite,
        impl BaseAnonCreds,
        impl BaseWallet,
    >,
    resolver_registry: Arc<ResolverRegistry>,
) -> Result<(), Box<dyn Error>> {
    let dummy_url: Url = "http://dummyurl.org".parse().unwrap();

    let invitation = Invitation::builder()
        .id("test_invite_id".to_owned())
        .content(
            InvitationContent::builder()
                .services(vec![OobService::Did(inviter_did)])
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

    let (requesters_peer_did, _our_verkey) =
        create_peer_did_4(&agent_invitee.wallet, dummy_url.clone(), vec![]).await?;
    let did_inviter: Did = invitation_get_first_did_service(&invitation)?;
    info!(
        "Invitee resolves Inviter's DID from invitation {} (as a first DID service found in the \
         invitation)",
        did_inviter
    );

    let TransitionResult {
        state: requester,
        output: request,
    } = DidExchangeRequester::<RequestSent>::construct_request(
        &resolver_registry,
        Some(invitation.id),
        &did_inviter,
        &requesters_peer_did,
        "some-label".to_owned(),
        DidExchangeTypeV1::new_v1_1(),
    )
    .await
    .unwrap();
    info!(
        "Invitee processes invitation, builds up request {:?}",
        &request
    );

    let (responders_peer_did, _our_verkey) =
        create_peer_did_4(&agent_inviter.wallet, dummy_url.clone(), vec![]).await?;
    let responders_did_document = responders_peer_did.resolve_did_doc()?;
    info!("Responder prepares did document: {responders_did_document}");

    let check_diddoc = responders_peer_did.resolve_did_doc()?;
    info!("Responder decodes constructed peer:did as did document: {check_diddoc}");

    let TransitionResult {
        output: response,
        state: responder,
    } = DidExchangeResponder::<ResponseSent>::receive_request(
        &agent_inviter.wallet,
        &resolver_registry,
        request,
        &responders_peer_did,
        invitation_key.clone(),
    )
    .await
    .unwrap();

    let TransitionResult {
        state: requester,
        output: complete,
    } = requester
        .receive_response(
            &agent_invitee.wallet,
            &invitation_key,
            response,
            &resolver_registry,
        )
        .await
        .unwrap();

    let responder = responder.receive_complete(complete.into_inner()).unwrap();

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
    let service = requester
        .their_did_doc()
        .get_service_of_type(&ServiceType::DIDCommV1)?;
    let m = EncryptionEnvelope::create(
        &agent_invitee.wallet,
        data.as_bytes(),
        requester.our_did_doc(),
        requester.their_did_doc(),
        service.id(),
    )
    .await?;

    info!("Encrypted message: {:?}", m);

    let requesters_peer_did = requesters_peer_did.resolve_did_doc()?;
    let expected_sender_vk = resolve_ed25519_base58_key_agreement(&requesters_peer_did)?;
    let unpacked =
        EncryptionEnvelope::auth_unpack(&agent_inviter.wallet, m.0, &expected_sender_vk).await?;

    info!("Unpacked message: {:?}", unpacked);

    Ok(())
}

#[tokio::test]
#[ignore]
async fn did_exchange_test_sov_inviter() -> Result<(), Box<dyn Error>> {
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

    did_exchange_test(
        format!("did:sov:{}", agent_inviter.institution_did),
        agent_inviter,
        agent_invitee,
        resolver_registry,
    )
    .await
}

#[tokio::test]
#[ignore]
async fn did_exchange_test_peer_to_peer() -> Result<(), Box<dyn Error>> {
    let setup = SetupPoolDirectory::init().await;
    let dummy_url: Url = "http://dummyurl.org".parse().unwrap();

    let agent_inviter = create_test_agent(setup.genesis_file_path.clone()).await;
    let agent_invitee = create_test_agent(setup.genesis_file_path.clone()).await;

    let resolver_registry = Arc::new(
        ResolverRegistry::new()
            .register_resolver::<PeerDidResolver>("peer".into(), PeerDidResolver::new()),
    );

    let (inviter_peer_did, _) =
        create_peer_did_4(&agent_inviter.wallet, dummy_url.clone(), vec![]).await?;

    did_exchange_test(
        inviter_peer_did.to_string(),
        agent_inviter,
        agent_invitee,
        resolver_registry,
    )
    .await
}

#[tokio::test]
#[ignore]
async fn did_exchange_test_with_invalid_rotation_signature() -> Result<(), Box<dyn Error>> {
    let setup = SetupPoolDirectory::init().await;
    let dummy_url: Url = "http://dummyurl.org".parse().unwrap();

    let agent_inviter = create_test_agent(setup.genesis_file_path.clone()).await;
    let agent_invitee = create_test_agent(setup.genesis_file_path.clone()).await;

    let resolver_registry = Arc::new(
        ResolverRegistry::new()
            .register_resolver::<PeerDidResolver>("peer".into(), PeerDidResolver::new()),
    );

    let (inviter_peer_did, _) =
        create_peer_did_4(&agent_inviter.wallet, dummy_url.clone(), vec![]).await?;

    let dummy_url: Url = "http://dummyurl.org".parse().unwrap();

    let invitation = Invitation::builder()
        .id("test_invite_id".to_owned())
        .content(
            InvitationContent::builder()
                .services(vec![OobService::Did(inviter_peer_did.to_string())])
                .build(),
        )
        .build();
    let real_invitation_key =
        resolve_enc_key_from_invitation(&invitation, &resolver_registry).await?;

    let (requesters_peer_did, _our_verkey) =
        create_peer_did_4(&agent_invitee.wallet, dummy_url.clone(), vec![]).await?;
    let did_inviter: Did = invitation_get_first_did_service(&invitation)?;

    let TransitionResult {
        state: requester,
        output: request,
    } = DidExchangeRequester::<RequestSent>::construct_request(
        &resolver_registry,
        Some(invitation.id),
        &did_inviter,
        &requesters_peer_did,
        "some-label".to_owned(),
        DidExchangeTypeV1::new_v1_1(),
    )
    .await?;

    let (responders_peer_did, incorrect_invitation_key) =
        create_peer_did_4(&agent_inviter.wallet, dummy_url.clone(), vec![]).await?;

    // create a response with a DID Rotate signed by the wrong key (not the original invitation key)
    let TransitionResult {
        output: response,
        state: _,
    } = DidExchangeResponder::<ResponseSent>::receive_request(
        &agent_inviter.wallet,
        &resolver_registry,
        request,
        &responders_peer_did,
        // sign with NOT the invitation key
        incorrect_invitation_key,
    )
    .await?;

    // receiving the response should fail when verifying the signature
    let res = requester
        .receive_response(
            &agent_invitee.wallet,
            &real_invitation_key,
            response,
            &resolver_registry,
        )
        .await;
    assert_eq!(
        res.unwrap_err().error.kind(),
        AriesVcxErrorKind::InvalidInput
    );

    Ok(())
}
