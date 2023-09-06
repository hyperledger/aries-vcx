#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

pub mod utils;

#[cfg(test)]
mod integration_tests {
    use async_channel::bounded;

    use aries_vcx::agency_client::MessageStatusCode;
    use aries_vcx::common::ledger::transactions::into_did_doc;
    use aries_vcx::handlers::connection::mediated_connection::ConnectionState;
    use aries_vcx::handlers::out_of_band::receiver::OutOfBandReceiver;
    use aries_vcx::handlers::out_of_band::sender::OutOfBandSender;
    use aries_vcx::handlers::util::AnyInvitation;
    use aries_vcx::protocols::mediated_connection::invitee::state_machine::InviteeState;
    use aries_vcx::utils::devsetup::*;
    use aries_vcx::utils::mockdata::mockdata_proof::REQUESTED_ATTRIBUTES;
    use messages::msg_fields::protocols::connection::Connection;
    use messages::msg_fields::protocols::out_of_band::invitation::OobService;
    use messages::msg_fields::protocols::out_of_band::{OobGoalCode, OutOfBand};
    use messages::msg_fields::protocols::present_proof::PresentProof;
    use messages::msg_types::connection::{ConnectionType, ConnectionTypeV1};
    use messages::msg_types::Protocol;
    use messages::AriesMessage;

    use crate::utils::devsetup_alice::create_alice;
    use crate::utils::devsetup_faber::create_faber_trustee;
    use crate::utils::scenarios::test_utils::{
        _send_message, connect_using_request_sent_to_public_agent, create_connected_connections,
        create_connected_connections_via_public_invite, create_proof_request_data, create_verifier_from_request_data,
    };

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_establish_connection_via_public_invite() {
        SetupPoolDirectory::run(|setup| async move {
            let mut institution = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path).await;

            let (consumer_to_institution, institution_to_consumer) =
                create_connected_connections_via_public_invite(&mut consumer, &mut institution).await;

            institution_to_consumer
                .send_generic_message(&institution.profile.inject_wallet(), "Hello Alice, Faber here")
                .await
                .unwrap();

            let consumer_msgs = consumer_to_institution
                .download_messages(&consumer.agency_client, Some(vec![MessageStatusCode::Received]), None)
                .await
                .unwrap();
            assert_eq!(consumer_msgs.len(), 1);
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_oob_connection_bootstrap() {
        SetupPoolDirectory::run(|setup| async move {
            let mut institution = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path).await;
            let (sender, receiver) = bounded::<AriesMessage>(1);

            let presentation_request_data =
                create_proof_request_data(&mut institution, REQUESTED_ATTRIBUTES, "[]", "{}", None).await;
            let request_sender = create_verifier_from_request_data(presentation_request_data)
                .await
                .get_presentation_request_msg()
                .unwrap();

            let did = institution.institution_did.clone();
            let oob_sender = OutOfBandSender::create()
                .set_label("test-label")
                .set_goal_code(OobGoalCode::P2PMessaging)
                .set_goal("To exchange message")
                .append_service(&OobService::Did(did))
                .append_handshake_protocol(Protocol::ConnectionType(ConnectionType::V1(
                    ConnectionTypeV1::new_v1_0(),
                )))
                .unwrap()
                .append_a2a_message(AriesMessage::from(request_sender.clone()))
                .unwrap();
            let invitation = AnyInvitation::Oob(oob_sender.oob.clone());
            let ddo = into_did_doc(&consumer.profile.inject_indy_ledger_read(), &invitation)
                .await
                .unwrap();
            let oob_msg = AriesMessage::from(oob_sender.oob.clone());

            let oob_receiver = OutOfBandReceiver::create_from_a2a_msg(&oob_msg).unwrap();
            let conns = vec![];
            let conn = oob_receiver
                .connection_exists(&consumer.profile.inject_indy_ledger_read(), &conns)
                .await
                .unwrap();
            assert!(conn.is_none());
            let mut conn_receiver = oob_receiver
                .build_connection(&consumer.profile.inject_wallet(), &consumer.agency_client, ddo, true)
                .await
                .unwrap();
            conn_receiver
                .connect(
                    &consumer.profile.inject_wallet(),
                    &consumer.agency_client,
                    _send_message(sender),
                )
                .await
                .unwrap();
            assert_eq!(
                ConnectionState::Invitee(InviteeState::Requested),
                conn_receiver.get_state()
            );
            assert_eq!(oob_sender.oob.id, oob_receiver.oob.id);

            let request = if let AriesMessage::Connection(Connection::Request(request)) = receiver.recv().await.unwrap()
            {
                request
            } else {
                panic!("Received invalid message type")
            };
            let conn_sender = connect_using_request_sent_to_public_agent(
                &mut consumer,
                &mut institution,
                &mut conn_receiver,
                request,
            )
            .await;

            let (conn_receiver_pw1, _conn_sender_pw1) =
                create_connected_connections(&mut consumer, &mut institution).await;
            let (conn_receiver_pw2, _conn_sender_pw2) =
                create_connected_connections(&mut consumer, &mut institution).await;

            let conns = vec![&conn_receiver, &conn_receiver_pw1, &conn_receiver_pw2];
            let conn = oob_receiver
                .connection_exists(&consumer.profile.inject_indy_ledger_read(), &conns)
                .await
                .unwrap();
            assert!(conn.is_some());
            assert!(*conn.unwrap() == conn_receiver);

            let conns = vec![&conn_receiver_pw1, &conn_receiver_pw2];
            let conn = oob_receiver
                .connection_exists(&consumer.profile.inject_indy_ledger_read(), &conns)
                .await
                .unwrap();
            assert!(conn.is_none());

            let a2a_msg = oob_receiver.extract_a2a_message().unwrap().unwrap();
            assert!(matches!(
                a2a_msg,
                AriesMessage::PresentProof(PresentProof::RequestPresentation(..))
            ));
            if let AriesMessage::PresentProof(PresentProof::RequestPresentation(request_receiver)) = a2a_msg {
                assert_eq!(
                    request_receiver.content.request_presentations_attach,
                    request_sender.content.request_presentations_attach
                );
            }

            conn_sender
                .send_generic_message(
                    &institution.profile.inject_wallet(),
                    "Hello oob receiver, from oob sender",
                )
                .await
                .unwrap();
            conn_receiver
                .send_generic_message(&consumer.profile.inject_wallet(), "Hello oob sender, from oob receiver")
                .await
                .unwrap();
            let sender_msgs = conn_sender
                .download_messages(&institution.agency_client, None, None)
                .await
                .unwrap();
            let receiver_msgs = conn_receiver
                .download_messages(&consumer.agency_client, None, None)
                .await
                .unwrap();
            assert_eq!(sender_msgs.len(), 2);
            assert_eq!(receiver_msgs.len(), 2);
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_oob_connection_reuse() {
        SetupPoolDirectory::run(|setup| async move {
            let mut institution = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path).await;

            let (consumer_to_institution, institution_to_consumer) =
                create_connected_connections_via_public_invite(&mut consumer, &mut institution).await;

            let did = institution.institution_did.clone();
            let oob_sender = OutOfBandSender::create()
                .set_label("test-label")
                .set_goal_code(OobGoalCode::P2PMessaging)
                .set_goal("To exchange message")
                .append_service(&OobService::Did(did));
            let oob_msg = AriesMessage::from(oob_sender.oob.clone());

            let oob_receiver = OutOfBandReceiver::create_from_a2a_msg(&oob_msg).unwrap();
            let conns = vec![&consumer_to_institution];
            let conn = oob_receiver
                .connection_exists(&consumer.profile.inject_indy_ledger_read(), &conns)
                .await
                .unwrap();
            assert!(conn.is_some());
            conn.unwrap()
                .send_generic_message(&consumer.profile.inject_wallet(), "Hello oob sender, from oob receiver")
                .await
                .unwrap();

            let msgs = institution_to_consumer
                .download_messages(&institution.agency_client, None, None)
                .await
                .unwrap();
            assert_eq!(msgs.len(), 2);
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_oob_connection_handshake_reuse() {
        SetupPoolDirectory::run(|setup| async move {
            let mut institution = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut consumer = create_alice(setup.genesis_file_path).await;

            let (mut consumer_to_institution, mut institution_to_consumer) =
                create_connected_connections_via_public_invite(&mut consumer, &mut institution).await;

            let did = institution.institution_did.clone();
            let oob_sender = OutOfBandSender::create()
                .set_label("test-label")
                .set_goal_code(OobGoalCode::P2PMessaging)
                .set_goal("To exchange message")
                .append_service(&OobService::Did(did));
            let sender_oob_id = oob_sender.get_id();
            let oob_msg = AriesMessage::from(oob_sender.oob.clone());

            let oob_receiver = OutOfBandReceiver::create_from_a2a_msg(&oob_msg).unwrap();
            let conns = vec![&consumer_to_institution];
            let conn = oob_receiver
                .connection_exists(&consumer.profile.inject_indy_ledger_read(), &conns)
                .await
                .unwrap();
            assert!(conn.is_some());
            let receiver_oob_id = oob_receiver.get_id();
            let receiver_msg = serde_json::to_string(&AriesMessage::from(oob_receiver.oob.clone())).unwrap();
            conn.unwrap()
                .send_handshake_reuse(&consumer.profile.inject_wallet(), &receiver_msg)
                .await
                .unwrap();

            let mut msgs = institution_to_consumer
                .download_messages(
                    &institution.agency_client,
                    Some(vec![MessageStatusCode::Received]),
                    None,
                )
                .await
                .unwrap();
            assert_eq!(msgs.len(), 1);
            let reuse_msg = match serde_json::from_str::<AriesMessage>(&msgs.pop().unwrap().decrypted_msg).unwrap() {
                AriesMessage::OutOfBand(OutOfBand::HandshakeReuse(ref a2a_msg)) => {
                    assert_eq!(
                        sender_oob_id,
                        a2a_msg.decorators.thread.pthid.as_ref().unwrap().to_string()
                    );
                    assert_eq!(
                        receiver_oob_id,
                        a2a_msg.decorators.thread.pthid.as_ref().unwrap().to_string()
                    );
                    assert_eq!(a2a_msg.id, a2a_msg.decorators.thread.thid.to_string());
                    a2a_msg.clone()
                }
                _ => {
                    panic!("Expected OutOfBandHandshakeReuse message type");
                }
            };
            institution_to_consumer
                .handle_message(
                    AriesMessage::OutOfBand(OutOfBand::HandshakeReuse(reuse_msg.clone())),
                    &institution.profile.inject_wallet(),
                )
                .await
                .unwrap();

            let mut msgs = consumer_to_institution
                .download_messages(&consumer.agency_client, Some(vec![MessageStatusCode::Received]), None)
                .await
                .unwrap();
            assert_eq!(msgs.len(), 1);
            let _reuse_ack_msg = match serde_json::from_str::<AriesMessage>(&msgs.pop().unwrap().decrypted_msg).unwrap()
            {
                AriesMessage::OutOfBand(OutOfBand::HandshakeReuseAccepted(ref a2a_msg)) => {
                    assert_eq!(
                        sender_oob_id,
                        a2a_msg.decorators.thread.pthid.as_ref().unwrap().to_string()
                    );
                    assert_eq!(
                        receiver_oob_id,
                        a2a_msg.decorators.thread.pthid.as_ref().unwrap().to_string()
                    );
                    assert_eq!(reuse_msg.id, a2a_msg.decorators.thread.thid.to_string());
                    a2a_msg.clone()
                }
                _ => {
                    panic!("Expected OutOfBandHandshakeReuseAccepted message type");
                }
            };
            consumer_to_institution
                .find_and_handle_message(&consumer.profile.inject_wallet(), &consumer.agency_client)
                .await
                .unwrap();
            assert_eq!(
                consumer_to_institution
                    .download_messages(&consumer.agency_client, Some(vec![MessageStatusCode::Received]), None)
                    .await
                    .unwrap()
                    .len(),
                0
            );
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    pub async fn test_agency_pool_two_enterprise_connections() {
        SetupPoolDirectory::run(|setup| async move {
            let mut institution = create_faber_trustee(setup.genesis_file_path.clone()).await;
            let mut consumer1 = create_alice(setup.genesis_file_path).await;

            let (_faber, _alice) = create_connected_connections(&mut consumer1, &mut institution).await;
            let (_faber, _alice) = create_connected_connections(&mut consumer1, &mut institution).await;
        })
        .await;
    }
}
