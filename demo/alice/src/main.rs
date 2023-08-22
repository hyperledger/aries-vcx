pub mod demo_agent;
pub mod http_client;

use crate::demo_agent::DemoAgent;
use crate::http_client::HttpClient;
use aries_vcx::handlers::out_of_band::sender::OutOfBandSender;
use aries_vcx::handlers::util::AnyInvitation;
use aries_vcx::messages::msg_fields::protocols::out_of_band::invitation::OobService;
use aries_vcx::protocols::connection::invitee::states::initial::Initial;
use aries_vcx::protocols::connection::invitee::InviteeConnection;
use aries_vcx::protocols::connection::pairwise_info::PairwiseInfo;
use aries_vcx::utils::mockdata::profile::mock_ledger::MockLedger;
use aries_vcx_core::ledger::base_ledger::IndyLedgerRead;
use diddoc_legacy::aries::service::AriesService;
use env_logger;
use simple_message_relay::build_msg_relay;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    env_logger::init();
    let alice = DemoAgent::new().await;
    let faber = DemoAgent::new().await;
    info!("Created agents");
    let (mediator_server, mut mediator_receiver) = build_msg_relay("127.0.0.1", 5901).unwrap();

    tokio::task::spawn(async move { mediator_server.await });

    let (_, faber_invite_key) = faber.wallet.create_and_store_my_did(None, None).await.unwrap();
    let service = AriesService {
        id: "".to_string(),
        type_: "".to_string(),
        priority: 0,
        recipient_keys: vec![faber_invite_key],
        routing_keys: vec![],
        service_endpoint: Url::parse("http://localhost:5901/send_user_message/faber").unwrap(),
    };
    let sender = OutOfBandSender::create().append_service(&OobService::AriesService(service));
    let oob_msg = sender.as_invitation_msg();

    let (alice_pw_did, alice_pw_key) = alice.wallet.create_and_store_my_did(None, None).await.unwrap();
    let pw_info = PairwiseInfo {
        pw_did: alice_pw_did,
        pw_vk: alice_pw_key,
    };
    info!("Alice generated pairwise info {pw_info:?}");
    let mut invitee_initial = InviteeConnection::<Initial>::new_invitee("foo".into(), pw_info);
    let mock_ledger: Arc<dyn IndyLedgerRead> = Arc::new(MockLedger {}); // cause we know we want call ledger *eew...*
    let mut invitee_inviter = invitee_initial
        .accept_invitation(&mock_ledger, AnyInvitation::Oob(oob_msg.into()))
        .await
        .unwrap();
    let invitee_requested = invitee_inviter
        .send_request(
            &Arc::new(alice.wallet),
            Url::parse("http://localhost:5901/send_user_message/alice").unwrap(),
            vec![],
            &HttpClient,
        )
        .await
        .unwrap();
    info!("Faber waiting for msg");
    let msg = mediator_receiver.recv().await;
    info!("Faber received message {msg:?}");
    exit(0);
}
