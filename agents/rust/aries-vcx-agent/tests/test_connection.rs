#[cfg(test)]
pub mod utils;

use actix::prelude::*;
use aries_vcx::{
    error::{VcxError, VcxErrorKind},
    handlers::connection::connection::ConnectionState,
    messages::{a2a::A2AMessage, did_doc::DidDoc},
    protocols::{
        connection::{invitee::state_machine::InviteeState, inviter::state_machine::InviterState},
        SendClosureConnection,
    },
};
use aries_vcx_agent::a2a_msg_actix::A2AMessageActix;
use aries_vcx_agent::Agent;

fn _send_message(sender: Addr<Agent>) -> Option<SendClosureConnection> {
    Some(Box::new(
        move |message: A2AMessage, _sender_vk: String, _did_doc: DidDoc| {
            Box::pin(async move {
                sender.send(A2AMessageActix(message)).await.unwrap().map_err(|err| {
                    VcxError::from_msg(VcxErrorKind::IOError, format!("Failed to send message: {:?}", err))
                })
            })
        },
    ))
}

#[actix::test]
async fn test_establish_connection() {
    let alice = utils::initialize_agent().await;
    let faber = utils::initialize_agent().await;

    let alice_addr = alice.clone().start();
    let faber_addr = faber.clone().start();

    let invite = faber.connections().create_invitation().await.unwrap();
    let tid = alice.connections().receive_invitation(invite).await.unwrap();

    alice
        .connections()
        .send_request(&tid, _send_message(faber_addr.clone()))
        .await
        .unwrap();
    faber
        .connections()
        .send_response(&tid, _send_message(alice_addr))
        .await
        .unwrap();
    alice
        .connections()
        .send_ack(&tid, _send_message(faber_addr.clone()))
        .await
        .unwrap();

    assert_eq!(
        alice.connections().get_state(&tid).unwrap(),
        ConnectionState::Invitee(InviteeState::Completed)
    );
    assert_eq!(
        faber.connections().get_state(&tid).unwrap(),
        ConnectionState::Inviter(InviterState::Completed)
    );

    let content = "Hello from Alice";
    alice
        .connections()
        .send_message(&tid, content, _send_message(faber_addr.clone()))
        .await
        .unwrap();
    if let Some(&A2AMessage::BasicMessage(ref msg)) = faber.received_messages().read().unwrap().back() {
        assert_eq!(msg.content, content);
    } else {
        panic!("Received unexpected message")
    };
}
