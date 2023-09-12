#[macro_use]
extern crate log;
extern crate serde_json;

pub mod utils;

use aries_vcx::common::ledger::transactions::write_endpoint_legacy;
use aries_vcx::protocols::connection::GenericConnection;
use aries_vcx::protocols::mediated_connection::pairwise_info::PairwiseInfo;
use aries_vcx::utils::encryption_envelope::EncryptionEnvelope;

use aries_vcx::utils::devsetup::*;
use chrono::Utc;
use diddoc_legacy::aries::service::AriesService;
use messages::decorators::timing::Timing;
use messages::msg_fields::protocols::basic_message::{BasicMessage, BasicMessageContent, BasicMessageDecorators};
use messages::AriesMessage;
use utils::test_agent::TestAgent;
use uuid::Uuid;

use crate::utils::scenarios::{
    create_connections_via_oob_invite, create_connections_via_pairwise_invite, create_connections_via_public_invite,
};
use crate::utils::test_agent::{create_test_agent, create_test_agent_trustee};

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
    consumer: &TestAgent,
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
    consumer: &TestAgent,
    insitution: &TestAgent,
    institution_to_consumer: &GenericConnection,
    consumer_to_institution: &GenericConnection,
    message: &AriesMessage,
) -> AriesMessage {
    let encrypted_message = institution_to_consumer
        .encrypt_message(&insitution.profile.inject_wallet(), message)
        .await
        .unwrap()
        .0;
    decrypt_message(consumer, encrypted_message, consumer_to_institution).await
}

async fn create_service(faber: &TestAgent) {
    let pairwise_info = PairwiseInfo::create(&faber.profile.inject_wallet()).await.unwrap();
    let service = AriesService::create()
        .set_service_endpoint("http://dummy.org".parse().unwrap())
        .set_recipient_keys(vec![pairwise_info.pw_vk.clone()]);
    write_endpoint_legacy(
        &faber.profile.inject_indy_ledger_write(),
        &faber.institution_did,
        &service,
    )
    .await
    .unwrap();
}

#[tokio::test]
#[ignore]
async fn test_agency_pool_establish_connection_via_public_invite() {
    SetupPoolDirectory::run(|setup| async move {
        let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path).await;
        create_service(&institution).await;

        let (consumer_to_institution, institution_to_consumer) =
            create_connections_via_public_invite(&mut consumer, &mut institution).await;

        let basic_message = build_basic_message("Hello TestAgent".to_string());
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
        let mut institution = create_test_agent(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path).await;

        let (consumer_to_institution, institution_to_consumer) =
            create_connections_via_pairwise_invite(&mut consumer, &mut institution).await;

        let basic_message = build_basic_message("Hello TestAgent".to_string());
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
        let mut institution = create_test_agent_trustee(setup.genesis_file_path.clone()).await;
        let mut consumer = create_test_agent(setup.genesis_file_path).await;
        create_service(&institution).await;

        let (consumer_to_institution, institution_to_consumer) =
            create_connections_via_oob_invite(&mut consumer, &mut institution).await;

        let basic_message = build_basic_message("Hello TestAgent".to_string());
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
