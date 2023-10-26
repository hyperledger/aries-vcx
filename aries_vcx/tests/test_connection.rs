pub mod utils;

use aries_vcx::{
    common::ledger::transactions::write_endpoint_legacy,
    protocols::{connection::GenericConnection, mediated_connection::pairwise_info::PairwiseInfo},
    utils::encryption_envelope::EncryptionEnvelope,
};
use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds,
    ledger::base_ledger::{
        AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite,
    },
    wallet::base_wallet::BaseWallet,
};
use chrono::Utc;
use diddoc_legacy::aries::service::AriesService;
use messages::{
    decorators::timing::Timing,
    msg_fields::protocols::basic_message::{
        BasicMessage, BasicMessageContent, BasicMessageDecorators,
    },
    AriesMessage,
};
use test_utils::devsetup::*;
use utils::test_agent::TestAgent;
use uuid::Uuid;

use crate::utils::{
    scenarios::{
        create_connections_via_oob_invite, create_connections_via_pairwise_invite,
        create_connections_via_public_invite,
    },
    test_agent::{create_test_agent, create_test_agent_trustee},
};

fn build_basic_message(content: String) -> BasicMessage {
    let now = Utc::now();

    let content = BasicMessageContent::builder()
        .content(content)
        .sent_time(now)
        .build();

    let decorators = BasicMessageDecorators::builder()
        .timing(Timing::builder().out_time(now).build())
        .build();

    BasicMessage::builder()
        .id(Uuid::new_v4().to_string())
        .content(content)
        .decorators(decorators)
        .build()
}

async fn decrypt_message(
    consumer: &TestAgent<
        impl IndyLedgerRead + AnoncredsLedgerRead,
        impl IndyLedgerWrite + AnoncredsLedgerWrite,
        impl BaseAnonCreds,
        impl BaseWallet,
    >,
    received: Vec<u8>,
    consumer_to_institution: &GenericConnection,
) -> AriesMessage {
    EncryptionEnvelope::auth_unpack(
        &consumer.wallet,
        received,
        &consumer_to_institution.remote_vk().unwrap(),
    )
    .await
    .unwrap()
}

async fn send_and_receive_message(
    consumer: &TestAgent<
        impl IndyLedgerRead + AnoncredsLedgerRead,
        impl IndyLedgerWrite + AnoncredsLedgerWrite,
        impl BaseAnonCreds,
        impl BaseWallet,
    >,
    insitution: &TestAgent<
        impl IndyLedgerRead + AnoncredsLedgerRead,
        impl IndyLedgerWrite + AnoncredsLedgerWrite,
        impl BaseAnonCreds,
        impl BaseWallet,
    >,
    institution_to_consumer: &GenericConnection,
    consumer_to_institution: &GenericConnection,
    message: &AriesMessage,
) -> AriesMessage {
    let encrypted_message = institution_to_consumer
        .encrypt_message(&insitution.wallet, message)
        .await
        .unwrap()
        .0;
    decrypt_message(consumer, encrypted_message, consumer_to_institution).await
}

async fn create_service(
    faber: &TestAgent<
        impl IndyLedgerRead + AnoncredsLedgerRead,
        impl IndyLedgerWrite + AnoncredsLedgerWrite,
        impl BaseAnonCreds,
        impl BaseWallet,
    >,
) {
    let pairwise_info = PairwiseInfo::create(&faber.wallet).await.unwrap();
    let service = AriesService::create()
        .set_service_endpoint("http://dummy.org".parse().unwrap())
        .set_recipient_keys(vec![pairwise_info.pw_vk.clone()]);
    write_endpoint_legacy(
        &faber.wallet,
        &faber.ledger_write,
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
