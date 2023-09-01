#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

use std::process::exit;
use std::sync::Arc;

use env_logger;
use serde::Deserialize;
use tokio::sync::mpsc::Receiver;
use url::Url;

use aries_vcx::handlers::out_of_band::sender::OutOfBandSender;
use aries_vcx::handlers::util::AnyInvitation;
use aries_vcx::messages::msg_fields::protocols::out_of_band::invitation::OobService;
use aries_vcx::protocols::connection::invitee::states::initial::Initial;
use aries_vcx::protocols::connection::invitee::InviteeConnection;
// todo: the fact that Invitee and Inviter states have same name can easily cause mis-matches in consumer's code
//       where they attempt to build Inviter<T>, but T is invitee state, like: invitee::states::responded::Responded
use aries_vcx::protocols::connection::invitee::states::responded::Responded;
use aries_vcx::protocols::connection::inviter::states::requested::Requested;
use aries_vcx::protocols::connection::inviter::InviterConnection;
use aries_vcx::protocols::connection::pairwise_info::PairwiseInfo;
use aries_vcx::transport::Transport;
use aries_vcx::utils::mockdata::profile::mock_ledger::MockLedger;
use aries_vcx_core::ledger::base_ledger::IndyLedgerRead;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use diddoc_legacy::aries::service::AriesService;
use messages::msg_fields::protocols::basic_message::{BasicMessage, BasicMessageContent, BasicMessageDecorators};
use messages::msg_fields::protocols::connection::request::Request;
use messages::msg_fields::protocols::connection::response::Response;
use messages::msg_fields::protocols::out_of_band::invitation::Invitation;
use messages::msg_parts::MsgParts;
use messages::AriesMessage;
use simple_message_relay::{build_msg_relay, UserMessage};

use crate::demo_agent::DemoAgent;
use crate::http_client::HttpClient;

pub mod demo_agent;
pub mod http_client;

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

fn build_oob_invitation(recipient_key: String, url: Url) -> Invitation {
    let service = AriesService {
        id: uuid::Uuid::new_v4().to_string(),
        type_: "".to_string(),
        priority: 0,
        recipient_keys: vec![recipient_key],
        routing_keys: vec![],
        service_endpoint: url,
    };
    let sender = OutOfBandSender::create().append_service(&OobService::AriesService(service));
    sender.as_invitation_msg()
}

async fn init() -> (DemoAgent, DemoAgent, Receiver<UserMessage>) {
    env_logger::init();
    let alice = DemoAgent::new(Url::parse("http://localhost:5901/send_user_message/alice").unwrap()).await;
    let faber = DemoAgent::new(Url::parse("http://localhost:5901/send_user_message/faber").unwrap()).await;
    let mediator_address = "127.0.0.1";
    let mediator_port = 5901;
    info!("Simple Alice & Faber agent abstractions created");

    let (mediator_server, mut mediator_receiver) = build_msg_relay(mediator_address, mediator_port).unwrap();

    tokio::task::spawn(async move {
        info!("Simple mediator relay service is listening on {mediator_address}:{mediator_port}");
        mediator_server.await
    });

    (alice, faber, mediator_receiver)
}

// todo: The return type is ugly - we don't care anymore about Connection protocol per se, we care that we have a didcomm channel
//       established. We should perhaps introduce a "DidcommChannel" struct or a trait
async fn workflow_alice_faber_connection(
    alice: &DemoAgent,
    faber: &DemoAgent,
    mediator_receiver: &mut Receiver<UserMessage>,
) -> (InviteeConnection<Responded>, InviterConnection<Requested>) {
    info!("Starting Alice & Faber workflow to establish didcomm connection");
    let (faber_pw_did, faber_invite_key) = faber.wallet.create_and_store_my_did(None, None).await.unwrap();
    let msg_oob_invitation = build_oob_invitation(faber_invite_key.clone(), faber.endpoint_url.clone());
    info!("Prepare invitation: {msg_oob_invitation:?}");

    let (alice_pw_did, alice_pw_key) = alice.wallet.create_and_store_my_did(None, None).await.unwrap();
    let pw_info_alice = PairwiseInfo {
        pw_did: alice_pw_did,
        pw_vk: alice_pw_key,
    };
    info!("Alice generated pairwise info {pw_info_alice:?}");
    let mock_ledger: Arc<dyn IndyLedgerRead> = Arc::new(MockLedger {}); // cause we know we want call ledger *eew...*
    let mut invitee_invited = InviteeConnection::<Initial>::new_invitee("foo".into(), pw_info_alice)
        .accept_invitation(&mock_ledger, AnyInvitation::Oob(msg_oob_invitation.into()))
        .await
        .unwrap();

    let invitee_requested = invitee_invited
        .prepare_request(alice.endpoint_url.clone(), vec![])
        .await
        .unwrap();
    info!("Faber waiting for msg");
    let didcomm_msg = mediator_receiver.recv().await.unwrap();
    info!("Faber received a msg");
    let (msg_request, _key) = decrypt_as_msg::<Request>(faber.wallet.clone(), didcomm_msg.as_slice()).await;
    info!("Faber received message {msg_request:?}");

    let pw_info_faber = PairwiseInfo {
        pw_did: faber_pw_did,
        pw_vk: faber_invite_key,
    };
    let inviter_invited = InviterConnection::new_inviter("".to_owned(), pw_info_faber).into_invited(
        &msg_request
            .decorators
            .thread
            .as_ref()
            .map(|t| t.thid.as_str())
            .unwrap_or(msg_request.id.as_str()),
    );
    let inviter_requested = inviter_invited
        .handle_request(
            &faber.wallet,
            msg_request,
            faber.endpoint_url.clone(),
            vec![],
            &HttpClient,
        )
        .await
        .unwrap();
    info!("inviter processed connection-request");
    let msg_response = inviter_requested.get_connection_response_msg();
    inviter_requested
        .send_message(&faber.wallet, &msg_response.into(), &HttpClient)
        .await
        .unwrap();

    info!("Alice waiting for msg");
    let didcomm_msg = mediator_receiver.recv().await.unwrap();
    info!("Alice received a msg");
    let (response, _key) = decrypt_as_msg::<Response>(alice.wallet.clone(), didcomm_msg.as_slice()).await;
    info!("Alice received message {response:?}");
    let invitee_complete = invitee_requested
        .handle_response(&alice.wallet.clone(), response, &HttpClient)
        .await
        .unwrap();
    (invitee_complete, inviter_requested)
}

async fn workflow_alice_faber_talk(
    connection_alice: &InviteeConnection<Responded>,
    alice: &DemoAgent,
    connection_faber: &InviterConnection<Requested>,
    faber: &DemoAgent,
    mediator_receiver: &mut Receiver<UserMessage>,
) {
    let msg_hello = build_basic_message("Hello Faber, this is Alice.".into());
    connection_alice
        .send_message(&alice.wallet, &AriesMessage::BasicMessage(msg_hello), &HttpClient)
        .await
        .unwrap();
    {
        info!("Faber waiting for msg");
        let didcomm_msg = mediator_receiver.recv().await.unwrap();
        info!("Faber received a msg");
        let (msg_any, _key) = decrypt_as_msg::<AriesMessage>(faber.wallet.clone(), didcomm_msg.as_slice()).await;
        info!("Faber received message {msg_any:?}");
    }

    let msg_hello = build_basic_message("Hello Alice, this is Faber.".into());
    connection_faber
        .send_message(&faber.wallet, &AriesMessage::BasicMessage(msg_hello), &HttpClient)
        .await
        .unwrap();
    {
        info!("Alice waiting for msg");
        let didcomm_msg = mediator_receiver.recv().await.unwrap();
        info!("Alice received a msg");
        let (msg_any, _key) = decrypt_as_msg::<AriesMessage>(alice.wallet.clone(), didcomm_msg.as_slice()).await;
        info!("Alice received message {msg_any:?}");
    }
}

#[tokio::main]
async fn main() {
    let (alice, faber, mut mediator_receiver) = init().await;
    let (connection_alice, connection_faber) =
        workflow_alice_faber_connection(&alice, &faber, &mut mediator_receiver).await;
    workflow_alice_faber_talk(
        &connection_alice,
        &alice,
        &connection_faber,
        &faber,
        &mut mediator_receiver,
    )
    .await;
    exit(0);
}
