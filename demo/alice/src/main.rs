pub mod demo_agent;
pub mod http_client;

use crate::demo_agent::DemoAgent;
use crate::http_client::HttpClient;
use aries_vcx::handlers::out_of_band::sender::OutOfBandSender;
use aries_vcx::handlers::util::AnyInvitation;
use aries_vcx::messages::msg_fields::protocols::out_of_band::invitation::OobService;
use aries_vcx::protocols::connection::invitee::states::initial::Initial;
use aries_vcx::protocols::connection::invitee::InviteeConnection;
use aries_vcx::protocols::connection::inviter::InviterConnection;
use aries_vcx::protocols::connection::pairwise_info::PairwiseInfo;
use aries_vcx::utils::mockdata::profile::mock_ledger::MockLedger;
use aries_vcx_core::ledger::base_ledger::IndyLedgerRead;
use diddoc_legacy::aries::service::AriesService;
use env_logger;
use messages::msg_fields::protocols::connection::request::Request;
use simple_message_relay::build_msg_relay;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate log;

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageData {
    message: String,
    recipient_verkey: String,
    sender_verkey: String,
}

#[tokio::main]
async fn main() {
    let faber_url = Url::parse("http://localhost:5901/send_user_message/faber").unwrap();
    let alice_url = Url::parse("http://localhost:5901/send_user_message/alice").unwrap();

    env_logger::init();
    let alice = DemoAgent::new().await;
    let faber = DemoAgent::new().await;
    info!("Created agents");
    let (mediator_server, mut mediator_receiver) = build_msg_relay("127.0.0.1", 5901).unwrap();

    tokio::task::spawn(async move { mediator_server.await });

    let (faber_pw_did, faber_invite_key) = faber.wallet.create_and_store_my_did(None, None).await.unwrap();
    let service = AriesService {
        id: uuid::Uuid::new_v4().to_string(),
        type_: "".to_string(),
        priority: 0,
        recipient_keys: vec![faber_invite_key.clone()],
        routing_keys: vec![],
        service_endpoint: faber_url.clone(),
    };
    let sender = OutOfBandSender::create().append_service(&OobService::AriesService(service));
    let oob_msg = sender.as_invitation_msg();
    info!("Prepare invitation: {oob_msg:?}");

    let (alice_pw_did, alice_pw_key) = alice.wallet.create_and_store_my_did(None, None).await.unwrap();
    let pw_info_alice = PairwiseInfo {
        pw_did: alice_pw_did,
        pw_vk: alice_pw_key,
    };
    info!("Alice generated pairwise info {pw_info_alice:?}");
    let mut invitee_initial = InviteeConnection::<Initial>::new_invitee("foo".into(), pw_info_alice);
    let mock_ledger: Arc<dyn IndyLedgerRead> = Arc::new(MockLedger {}); // cause we know we want call ledger *eew...*
    let mut invitee_inviter = invitee_initial
        .accept_invitation(&mock_ledger, AnyInvitation::Oob(oob_msg.into()))
        .await
        .unwrap();
    let invitee_requested = invitee_inviter
        .send_request(&Arc::new(alice.wallet), alice_url, vec![], &HttpClient)
        .await
        .unwrap();
    {
        info!("Faber waiting for msg");
        let didcomm_msg = mediator_receiver.recv().await.unwrap();
        info!("Faber received a msg");

        let decrypted_msg = faber.wallet.unpack_message(&didcomm_msg).await.unwrap();
        let unpacked: MessageData = serde_json::from_slice(&decrypted_msg).unwrap();
        let request: Request = serde_json::from_str(&unpacked.message).unwrap();
        info!("Faber received message {request:?}");

        let pw_info_faber = PairwiseInfo {
            pw_did: faber_pw_did,
            pw_vk: faber_invite_key,
        };
        let inviter = InviterConnection::new_inviter("".to_owned(), pw_info_faber).into_invited(
            &request
                .decorators
                .thread
                .as_ref()
                .map(|t| t.thid.as_str())
                .unwrap_or(request.id.as_str()),
        );
        let (faber_pw_did, faber_pw_key) = faber.wallet.create_and_store_my_did(None, None).await.unwrap();
        // todo: noticed discrepancy, this is creating pw keys internally, but in the other cases we had to supply them
        inviter
            .handle_request(&faber.wallet, request, faber_url, vec![], &HttpClient)
            .await
            .unwrap();
        info!("inviter processed connection-request");
    }
    {
        info!("Alice waiting for msg");
        let didcomm_msg = mediator_receiver.recv().await.unwrap();
        info!("Alice received a msg");
    }
    exit(0);
}
