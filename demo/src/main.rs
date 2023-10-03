#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::process::exit;

use aries_vcx::utils::encryption_envelope::EncryptionEnvelope;
use messages::{
    msg_fields::protocols::{
        connection::{request::Request, response::Response},
        out_of_band::invitation::Invitation,
    },
    AriesMessage,
};
use tokio::sync::mpsc;
use url::Url;

use crate::demo_agent::{build_basic_message, DemoAgent, COLOR_GREEN, COLOR_YELLOW};

pub mod demo_agent;
pub mod mpsc_registry;

static ALICE: &str = "alice";
static FABER: &str = "faber";

async fn init() -> (DemoAgent, DemoAgent) {
    env_logger::init();

    let mediator_address = "127.0.0.1";
    let mediator_port = 5901;

    let mediator_url = Url::parse(&format!("http://{mediator_address}:{mediator_port}")).unwrap();
    let agent_alice = DemoAgent::new(ALICE.into(), mediator_url.clone(), COLOR_YELLOW.into()).await;
    let agent_faber = DemoAgent::new(FABER.into(), mediator_url.clone(), COLOR_GREEN.into()).await;

    (agent_alice, agent_faber)
}

async fn setup_transport_channels(alice: &DemoAgent, faber: &DemoAgent) -> String {
    let transport_id = format!("relation_{}_{}", alice.get_name(), faber.get_name());
    // This simulates trusted communication channel, over which Faber send Alice an invitation
    // In real setting, the equivalent could be scanning a QR code on trustworthy https:// website of Faber company
    let (inviter_sender, invitee_receiver) = mpsc::channel::<Invitation>(1);
    faber
        .register_invitation_sender(transport_id.clone(), inviter_sender)
        .await;
    alice
        .register_invitation_receiver(transport_id.clone(), invitee_receiver)
        .await;

    // These channels simulate communicating via didcomm. Sending a message in real setting means
    // sending encrypted message, using keys specified in counterparty's DidDoc,
    // as a POST HTTP call to an URL, specified as service endpoint specified in counterparty's
    // DidDoc.
    let (faber_didcomm_sender, alice_didcomm_receiver) = mpsc::channel::<EncryptionEnvelope>(1);
    let (alice_didcomm_sender, faber_didcomm_receiver) = mpsc::channel::<EncryptionEnvelope>(1);
    faber
        .register_didcomm_channel(
            transport_id.clone(),
            faber_didcomm_sender,
            faber_didcomm_receiver,
        )
        .await;
    alice
        .register_didcomm_channel(
            transport_id.clone(),
            alice_didcomm_sender,
            alice_didcomm_receiver,
        )
        .await;

    transport_id
}

async fn workflow_connection_by_role(alice: DemoAgent, faber: DemoAgent, transport_id: String) {
    let transport_id_copy = transport_id.clone();
    let handle2 = tokio::task::spawn(async move { workflow_inviter(faber, &transport_id).await });
    let handle1 =
        tokio::task::spawn(async move { workflow_invitee(alice, &transport_id_copy).await });
    let res = tokio::join!(handle1, handle2);
    res.0.unwrap();
    res.1.unwrap();
}

async fn workflow_invitee(alice: DemoAgent, transport_id: &str) {
    let msg_oob_invitation: Invitation = alice.wait_for_invitation_message(transport_id).await;
    info!("****** Alice receives connection invitation via a trusted channel ******");
    let (invitee_requested, msg_connection_request) =
        alice.process_invitation(msg_oob_invitation).await;
    alice
        .send_didcomm_message(transport_id, msg_connection_request)
        .await;

    let response = alice
        .wait_for_didcomm_message::<Response>(transport_id)
        .await
        .unwrap();
    info!(
        "****** Alice receives 'connection-response', will reply back with 'connection-response' \
         ******"
    );
    let invitee_complete = alice
        .process_connection_response(invitee_requested, response)
        .await;

    info!(
        "****** Alice and Faber can now securely exchange other messages via established didcomm \
         connection ******"
    );
    let msg_hello = alice
        .encrypt_message(
            invitee_complete,
            build_basic_message("Hello Faber, this is Alice.".into()).into(),
        )
        .await;
    alice.send_didcomm_message(transport_id, msg_hello).await;

    let msg_any = alice
        .wait_for_didcomm_message::<AriesMessage>(transport_id)
        .await;
    info!("Alice received message {msg_any:?}");
}

async fn workflow_inviter(faber: DemoAgent, transport_id: &str) {
    info!("****** Faber is preparing connection invitation to pass to Alice ****** ");
    let (msg_oob_invitation, faber_invite_info) = faber.prepare_invitation().await;
    faber
        .send_invitation_message(transport_id, msg_oob_invitation)
        .await;

    info!(
        "****** Faber received connection invitation via trusted means, will reply sending \
         'connection-request' message ******"
    );
    let msg_request = faber
        .wait_for_didcomm_message::<Request>(transport_id)
        .await
        .unwrap();
    // todo: make the decryption part of agent's receive function
    let (inviter_requested, connection_response) = faber
        .process_connection_request(msg_request, faber_invite_info)
        .await;
    faber
        .send_didcomm_message(transport_id, connection_response)
        .await;

    let msg_any = faber
        .wait_for_didcomm_message::<AriesMessage>(transport_id)
        .await
        .unwrap();
    info!("Faber received message {msg_any:?}");

    let msg_hello = faber
        .encrypt_message(
            inviter_requested,
            build_basic_message("Hello Faber, this is Alice.".into()).into(),
        )
        .await;
    faber.send_didcomm_message(transport_id, msg_hello).await;
}

#[tokio::main]
async fn main() {
    let (alice, faber) = init().await;
    let transport_id = setup_transport_channels(&alice, &faber).await;
    workflow_connection_by_role(alice, faber, transport_id).await;
    exit(0);
}
