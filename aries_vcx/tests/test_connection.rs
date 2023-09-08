#[macro_use]
extern crate log;
extern crate serde_json;

pub mod utils;

use aries_vcx::protocols::oob::build_handshake_reuse_msg;
use aries_vcx::utils::encryption_envelope::EncryptionEnvelope;
use async_channel::bounded;

use aries_vcx::handlers::out_of_band::receiver::OutOfBandReceiver;
use aries_vcx::handlers::out_of_band::sender::OutOfBandSender;
use aries_vcx::utils::devsetup::*;
use chrono::Utc;
use messages::decorators::timing::Timing;
use messages::msg_fields::protocols::basic_message::{BasicMessage, BasicMessageContent, BasicMessageDecorators};
use messages::msg_fields::protocols::out_of_band::invitation::OobService;
use messages::msg_fields::protocols::out_of_band::{OobGoalCode, OutOfBand};
use messages::AriesMessage;
use uuid::Uuid;

use crate::utils::devsetup_alice::create_alice;
use crate::utils::devsetup_faber::create_faber_trustee;
use crate::utils::scenarios::{create_connections_via_oob_invite, create_connections_via_public_invite};
use crate::utils::transport_trait::TestTransport;

fn build_basic_message(content: String) -> BasicMessage {
    let now = Utc::now();

    let content = BasicMessageContent::new(content, now);

    let mut decorators = BasicMessageDecorators::default();
    let mut timing = Timing::default();
    timing.out_time = Some(now);

    decorators.timing = Some(timing);

    BasicMessage::with_decorators(Uuid::new_v4().to_string(), content, decorators)
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_establish_connection_via_public_invite_and_reuse() {
    SetupPoolDirectory::run(|setup| async move {
        let mut institution = create_faber_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_alice(setup.genesis_file_path).await;

        let (consumer_to_institution, institution_to_consumer) =
            create_connections_via_public_invite(&mut consumer, &mut institution).await;

        // Try exchanging encrypted message (to verify verkeys were exchanged correctly)
        let basic_message = build_basic_message("Hello Faber".to_string());
        let (sender, receiver) = bounded::<Vec<u8>>(1);
        institution_to_consumer
            .send_message(
                &institution.profile.inject_wallet(),
                &basic_message.clone().into(),
                &TestTransport { sender },
            )
            .await
            .unwrap();

        // TODO: Extract
        let received = receiver.recv().await.unwrap();
        let unpacked = EncryptionEnvelope::auth_unpack(
            &consumer.profile.inject_wallet(),
            received,
            &consumer_to_institution.remote_vk().unwrap(),
        )
        .await
        .unwrap();
        if let AriesMessage::BasicMessage(message) = unpacked {
            assert_eq!(message.content.content, basic_message.content.content);
        } else {
            panic!("Unexpected message type");
        }
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_establish_connection_via_out_of_band() {
    SetupPoolDirectory::run(|setup| async move {
        let mut institution = create_faber_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_alice(setup.genesis_file_path).await;

        // Connect by sending request
        let (consumer_to_institution, institution_to_consumer) =
            create_connections_via_oob_invite(&mut consumer, &mut institution).await;

        // Extract A2A message from OOB message
        // let a2a_msg = oob_receiver.extract_a2a_message().unwrap().unwrap();
        // assert!(matches!(
        //     a2a_msg,
        //     AriesMessage::PresentProof(PresentProof::RequestPresentation(..))
        // ));
        // if let AriesMessage::PresentProof(PresentProof::RequestPresentation(request_receiver)) = a2a_msg {
        //     assert_eq!(
        //         request_receiver.content.request_presentations_attach,
        //         presentation_request.content.request_presentations_attach
        //     );
        // }

        let basic_message = build_basic_message("Hello Faber".to_string());
        let (sender, receiver) = bounded::<Vec<u8>>(1);
        institution_to_consumer
            .send_message(
                &institution.profile.inject_wallet(),
                &basic_message.clone().into(),
                &TestTransport { sender },
            )
            .await
            .unwrap();

        // TODO: Extract
        let received = receiver.recv().await.unwrap();
        let unpacked = EncryptionEnvelope::auth_unpack(
            &consumer.profile.inject_wallet(),
            received,
            &consumer_to_institution.remote_vk().unwrap(),
        )
        .await
        .unwrap();
        if let AriesMessage::BasicMessage(message) = unpacked {
            assert_eq!(message.content.content, basic_message.content.content);
        } else {
            panic!("Unexpected message type");
        }
    })
    .await;
}

// TODO: This can be done in test_agency_pool_establish_connection_via_public_invite_and_reuse
#[tokio::test]
#[ignore]
async fn test_agency_pool_oob_connection_handshake_reuse() {
    SetupPoolDirectory::run(|setup| async move {
        let (sender, receiver) = bounded::<Vec<u8>>(1);

        let mut institution = create_faber_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_alice(setup.genesis_file_path).await;

        let (consumer_to_institution, institution_to_consumer) =
            create_connections_via_public_invite(&mut consumer, &mut institution).await;

        let did = institution.institution_did.clone();
        let oob_sender = OutOfBandSender::create()
            .set_label("test-label")
            .set_goal_code(OobGoalCode::P2PMessaging)
            .set_goal("To exchange message")
            .append_service(&OobService::Did(did));
        let sender_oob_id = oob_sender.get_id();
        let oob_msg = AriesMessage::from(oob_sender.oob.clone());

        let oob_receiver = OutOfBandReceiver::create_from_a2a_msg(&oob_msg).unwrap();
        let receiver_oob_id = oob_receiver.get_id();
        let hanshake_reuse_msg = build_handshake_reuse_msg(&oob_receiver.oob);

        consumer_to_institution
            .send_message(
                &consumer.profile.inject_wallet(),
                &hanshake_reuse_msg.into(),
                &TestTransport { sender },
            )
            .await
            .unwrap();

        let received = receiver.recv().await.unwrap();
        let unpacked = EncryptionEnvelope::auth_unpack(
            &institution.profile.inject_wallet(),
            received,
            &institution_to_consumer.remote_vk().unwrap(),
        )
        .await
        .unwrap();

        let a2a_msg = if let AriesMessage::OutOfBand(OutOfBand::HandshakeReuse(message)) = unpacked {
            message
        } else {
            panic!("Unexpected message type");
        };

        assert_eq!(
            sender_oob_id,
            a2a_msg.decorators.thread.pthid.as_ref().unwrap().to_string()
        );
        assert_eq!(
            receiver_oob_id,
            a2a_msg.decorators.thread.pthid.as_ref().unwrap().to_string()
        );
        assert_eq!(a2a_msg.id, a2a_msg.decorators.thread.thid.to_string());
    })
    .await;
}
