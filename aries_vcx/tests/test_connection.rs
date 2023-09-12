#[macro_use]
extern crate log;
extern crate serde_json;

pub mod utils;

use aries_vcx::protocols::connection::GenericConnection;
use aries_vcx::utils::encryption_envelope::EncryptionEnvelope;
use async_channel::bounded;

use aries_vcx::utils::devsetup::*;
use chrono::Utc;
use messages::decorators::timing::Timing;
use messages::msg_fields::protocols::basic_message::{BasicMessage, BasicMessageContent, BasicMessageDecorators};
use messages::AriesMessage;
use utils::devsetup_alice::Alice;
use utils::devsetup_faber::Faber;
use uuid::Uuid;

use crate::utils::devsetup_alice::create_alice;
use crate::utils::devsetup_faber::{create_faber, create_faber_trustee};
use crate::utils::scenarios::{
    create_connections_via_oob_invite, create_connections_via_pairwise_invite, create_connections_via_public_invite,
};
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

async fn decrypt_message(
    consumer: &Alice,
    received: Vec<u8>,
    consumer_to_institution: &GenericConnection,
) -> AriesMessage {
    EncryptionEnvelope::auth_unpack(
        &consumer.profile.inject_wallet(),
        received,
        &consumer_to_institution.remote_vk().unwrap(),
    )
    .await
    .unwrap()
}

async fn send_and_receive_message(
    consumer: &Alice,
    insitution: &Faber,
    institatuion_to_consumer: &GenericConnection,
    consumer_to_institution: &GenericConnection,
    message: &AriesMessage,
) -> AriesMessage {
    let (sender, receiver) = bounded::<Vec<u8>>(1);
    institatuion_to_consumer
        .send_message(&insitution.profile.inject_wallet(), message, &TestTransport { sender })
        .await
        .unwrap();
    let received = receiver.recv().await.unwrap();
    decrypt_message(consumer, received, consumer_to_institution).await
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_establish_connection_via_public_invite() {
    SetupPoolDirectory::run(|setup| async move {
        let mut institution = create_faber_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_alice(setup.genesis_file_path).await;

        let (consumer_to_institution, institution_to_consumer) =
            create_connections_via_public_invite(&mut consumer, &mut institution).await;

        let basic_message = build_basic_message("Hello Faber".to_string());
        if let AriesMessage::BasicMessage(message) = send_and_receive_message(
            &consumer,
            &institution,
            &institution_to_consumer,
            &consumer_to_institution,
            &basic_message.clone().into(),
        )
        .await
        {
            assert_eq!(message.content.content, basic_message.content.content);
        } else {
            panic!("Unexpected message type");
        }
    })
    .await;
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_establish_connection_via_pairwise_invite() {
    SetupPoolDirectory::run(|setup| async move {
        let mut institution = create_faber(setup.genesis_file_path.clone()).await;
        let mut consumer = create_alice(setup.genesis_file_path).await;

        let (consumer_to_institution, institution_to_consumer) =
            create_connections_via_pairwise_invite(&mut consumer, &mut institution).await;

        let basic_message = build_basic_message("Hello Faber".to_string());
        if let AriesMessage::BasicMessage(message) = send_and_receive_message(
            &consumer,
            &institution,
            &institution_to_consumer,
            &consumer_to_institution,
            &basic_message.clone().into(),
        )
        .await
        {
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

        let (consumer_to_institution, institution_to_consumer) =
            create_connections_via_oob_invite(&mut consumer, &mut institution).await;

        let basic_message = build_basic_message("Hello Faber".to_string());
        if let AriesMessage::BasicMessage(message) = send_and_receive_message(
            &consumer,
            &institution,
            &institution_to_consumer,
            &consumer_to_institution,
            &basic_message.clone().into(),
        )
        .await
        {
            assert_eq!(message.content.content, basic_message.content.content);
        } else {
            panic!("Unexpected message type");
        }
    })
    .await;
}
