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
use aries_vcx::transport::Transport;
use aries_vcx::utils::mockdata::profile::mock_ledger::MockLedger;
use aries_vcx_core::ledger::base_ledger::IndyLedgerRead;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use diddoc_legacy::aries::service::AriesService;
use env_logger;
use messages::msg_fields::protocols::basic_message::{BasicMessage, BasicMessageContent, BasicMessageDecorators};
use messages::msg_fields::protocols::connection::request::Request;
use messages::msg_fields::protocols::connection::response::Response;
use messages::msg_parts::MsgParts;
use messages::AriesMessage;
use serde::Deserialize;
use simple_message_relay::build_msg_relay;
use std::process::exit;
use std::sync::Arc;
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

async fn decrypt_as_msg<T>(wallet: Arc<dyn BaseWallet>, didcomm_msg: &[u8]) -> (T, String)
where
    T: for<'de> Deserialize<'de>,
{
    let decrypted_msg = wallet.unpack_message(&didcomm_msg).await.unwrap();
    let unpacked: MessageData = serde_json::from_slice(&decrypted_msg).unwrap();
    let msg: T = serde_json::from_str(&unpacked.message).unwrap();
    return (msg, unpacked.sender_verkey);
}

fn build_basic_message(text_message: String) -> BasicMessage {
    let bm = BasicMessageContent {
        content: text_message,
        sent_time: Default::default(),
    };
    let bmd = BasicMessageDecorators {
        l10n: None,
        thread: None,
        timing: None,
    };
    MsgParts::with_decorators(uuid::Uuid::new_v4().to_string(), bm, bmd)
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
        .send_request(&alice.wallet, alice_url, vec![], &HttpClient)
        .await
        .unwrap();
    {
        info!("Faber waiting for msg");
        let didcomm_msg = mediator_receiver.recv().await.unwrap();
        info!("Faber received a msg");
        let (request, _key) = decrypt_as_msg::<Request>(faber.wallet.clone(), didcomm_msg.as_slice()).await;
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
        let requested = inviter
            .handle_request(&faber.wallet, request, faber_url, vec![], &HttpClient)
            .await
            .unwrap();
        info!("inviter processed connection-request");
        let request = requested.get_connection_response_msg();
        requested
            .send_message(&faber.wallet, &request.into(), &HttpClient)
            .await
            .unwrap();
    }
    info!("Alice waiting for msg");
    let didcomm_msg = mediator_receiver.recv().await.unwrap();
    info!("Alice received a msg");
    let (response, _key) = decrypt_as_msg::<Response>(alice.wallet.clone(), didcomm_msg.as_slice()).await;
    info!("Alice received message {response:?}");
    let invitee_complete = invitee_requested
        .handle_response(&alice.wallet.clone(), response, &HttpClient)
        .await
        .unwrap();
    info!("Alice processed response");
    let basic_message = build_basic_message("Hello faber, this is alice.".into());
    invitee_complete
        .send_message(&alice.wallet, &AriesMessage::BasicMessage(basic_message), &HttpClient)
        .await
        .unwrap();
    {
        info!("Faber waiting for msg");
        let didcomm_msg = mediator_receiver.recv().await.unwrap();
        info!("Faber received a msg");
        let (request, _key) = decrypt_as_msg::<AriesMessage>(faber.wallet.clone(), didcomm_msg.as_slice()).await;
        info!("Faber received message {request:?}");
    }
    exit(0);
}
