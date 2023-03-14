#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_json;

pub mod utils;

#[cfg(test)]
#[cfg(feature = "agency_pool_tests")]
mod integration_tests {
    use std::{sync::Arc, thread, time::Duration};

    use agency_client::{
        agency_client::AgencyClient, api::downloaded_message::DownloadedMessage, messages::update_message::UIDsByConn,
        MessageStatusCode,
    };
    use aries_vcx::{
        global::settings,
        indy::wallet::{close_wallet, WalletConfig},
        messages::{a2a::A2AMessage, concepts::ack::test_utils::_ack},
        plugins::wallet::{
            agency_client_wallet::ToBaseAgencyClientWallet, base_wallet::BaseWallet, indy_wallet::IndySdkWallet,
        },
        utils::devsetup::SetupPool,
    };

    use crate::utils::{
        devsetup_agent::test_utils::{create_test_alice_instance, Faber},
        scenarios::test_utils::create_connected_connections,
    };

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_send_and_download_messages() {
        SetupPool::run(|setup| async move {
            let mut institution = Faber::setup(setup.pool_handle).await;
            let mut consumer = create_test_alice_instance(&setup).await;

            let (alice_to_faber, faber_to_alice) = create_connected_connections(&mut consumer, &mut institution).await;

            faber_to_alice
                .send_generic_message(&institution.profile, "Hello Alice")
                .await
                .unwrap();
            faber_to_alice
                .send_generic_message(&institution.profile, "How are you Alice?")
                .await
                .unwrap();

            alice_to_faber
                .send_generic_message(&consumer.profile, "Hello Faber")
                .await
                .unwrap();

            thread::sleep(Duration::from_millis(100));

            let msgs = faber_to_alice
                .download_messages(&institution.agency_client, None, None)
                .await
                .unwrap();
            assert_eq!(msgs.len(), 2);
            let ack_msg = msgs
                .iter()
                .find(|msg| {
                    msg.decrypted_msg
                        .clone()
                        .contains("https://didcomm.org/notification/1.0/ack")
                })
                .unwrap();
            assert_eq!(ack_msg.status_code, MessageStatusCode::Reviewed);
            let hello_msg = msgs
                .iter()
                .find(|msg| msg.decrypted_msg.clone().contains("Hello Faber"))
                .unwrap();
            assert_eq!(hello_msg.status_code, MessageStatusCode::Received);

            let received = faber_to_alice
                .download_messages(
                    &institution.agency_client,
                    Some(vec![MessageStatusCode::Received]),
                    None,
                )
                .await
                .unwrap();
            assert_eq!(received.len(), 1);
            received
                .iter()
                .find(|msg| msg.decrypted_msg.clone().contains("Hello Faber"))
                .unwrap();

            let msgs_by_uid = faber_to_alice
                .download_messages(&institution.agency_client, None, Some(vec![hello_msg.uid.clone()]))
                .await
                .unwrap();
            assert_eq!(msgs_by_uid.len(), 1);
            assert_eq!(msgs_by_uid[0].uid, hello_msg.uid);

            let double_filter = faber_to_alice
                .download_messages(
                    &institution.agency_client,
                    Some(vec![MessageStatusCode::Received]),
                    Some(vec![hello_msg.uid.clone()]),
                )
                .await
                .unwrap();
            assert_eq!(double_filter.len(), 1);
            assert_eq!(double_filter[0].uid, hello_msg.uid);

            let msgs = faber_to_alice
                .download_messages(&institution.agency_client, None, Some(vec!["abcd123".into()]))
                .await
                .unwrap();
            assert_eq!(msgs.len(), 0);
        })
        .await;
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_connection_send_works() {
        SetupPool::run(|setup| async move {
            let mut faber = Faber::setup(setup.pool_handle).await;
            let mut alice = create_test_alice_instance(&setup).await;

            let invite = faber.create_invite().await;
            alice.accept_invite(&invite).await;

            faber.update_state(3).await;
            alice.update_state(4).await;
            faber.update_state(4).await;

            let uid: String;
            let message = _ack();

            info!("test_connection_send_works:: Test if Send Message works");
            {
                faber.connection.send_message_closure(&faber.profile).await.unwrap()(message.to_a2a_message())
                    .await
                    .unwrap();
            }

            {
                info!("test_connection_send_works:: Test if Get Messages works");

                let messages = alice.connection.get_messages(&alice.agency_client).await.unwrap();
                assert_eq!(1, messages.len());

                uid = messages.keys().next().unwrap().clone();
                let received_message = messages.values().next().unwrap().clone();

                match received_message {
                    A2AMessage::Ack(received_message) => assert_eq!(message, received_message.clone()),
                    _ => assert!(false),
                }
            }

            info!("test_connection_send_works:: Test if Get Message by id works");
            {
                let message = alice
                    .connection
                    .get_message_by_id(&uid.clone(), &alice.agency_client)
                    .await
                    .unwrap();

                match message {
                    A2AMessage::Ack(ack) => assert_eq!(_ack(), ack),
                    _ => assert!(false),
                }
            }

            info!("test_connection_send_works:: Test if Update Message Status works");
            {
                alice
                    .connection
                    .update_message_status(&uid, &alice.agency_client)
                    .await
                    .unwrap();
                let messages = alice.connection.get_messages(&alice.agency_client).await.unwrap();
                assert_eq!(0, messages.len());
            }

            info!("test_connection_send_works:: Test if Send Basic Message works");
            {
                let basic_message = r#"Hi there"#;
                faber
                    .connection
                    .send_generic_message(&faber.profile, basic_message)
                    .await
                    .unwrap();

                let messages = alice.connection.get_messages(&alice.agency_client).await.unwrap();
                assert_eq!(1, messages.len());

                let uid = messages.keys().next().unwrap().clone();
                let message = messages.values().next().unwrap().clone();

                match message {
                    A2AMessage::BasicMessage(message) => assert_eq!(basic_message, message.content),
                    _ => assert!(false),
                }
                alice
                    .connection
                    .update_message_status(&uid, &alice.agency_client)
                    .await
                    .unwrap();
            }

            info!("test_connection_send_works:: Test if Download Messages");
            {
                let credential_offer =
                    aries_vcx::messages::protocols::issuance::credential_offer::test_utils::_credential_offer();

                faber.connection.send_message_closure(&faber.profile).await.unwrap()(credential_offer.to_a2a_message())
                    .await
                    .unwrap();

                let msgs = alice
                    .connection
                    .download_messages(&alice.agency_client, Some(vec![MessageStatusCode::Received]), None)
                    .await
                    .unwrap();
                let message: DownloadedMessage = msgs[0].clone();
                let _payload: aries_vcx::messages::protocols::issuance::credential_offer::CredentialOffer =
                    serde_json::from_str(&message.decrypted_msg).unwrap();

                alice
                    .connection
                    .update_message_status(&message.uid, &alice.agency_client)
                    .await
                    .unwrap();
            }
        })
        .await;
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_download_messages() {
        SetupPool::run(|setup| async move {
            let mut institution = Faber::setup(setup.pool_handle).await;
            let mut consumer1 = create_test_alice_instance(&setup).await;
            let mut consumer2 = create_test_alice_instance(&setup).await;

            let (consumer1_to_institution, institution_to_consumer1) =
                create_connected_connections(&mut consumer1, &mut institution).await;
            let (consumer2_to_institution, institution_to_consumer2) =
                create_connected_connections(&mut consumer2, &mut institution).await;

            consumer1_to_institution
                .send_generic_message(&consumer1.profile, "Hello Institution from consumer1")
                .await
                .unwrap();
            consumer2_to_institution
                .send_generic_message(&consumer2.profile, "Hello Institution from consumer2")
                .await
                .unwrap();

            let consumer1_msgs = institution_to_consumer1
                .download_messages(&institution.agency_client, None, None)
                .await
                .unwrap();
            assert_eq!(consumer1_msgs.len(), 2);

            let consumer2_msgs = institution_to_consumer2
                .download_messages(&institution.agency_client, None, None)
                .await
                .unwrap();
            assert_eq!(consumer2_msgs.len(), 2);

            let consumer1_received_msgs = institution_to_consumer1
                .download_messages(
                    &institution.agency_client,
                    Some(vec![MessageStatusCode::Received]),
                    None,
                )
                .await
                .unwrap();
            assert_eq!(consumer1_received_msgs.len(), 1);

            let consumer1_reviewed_msgs = institution_to_consumer1
                .download_messages(
                    &institution.agency_client,
                    Some(vec![MessageStatusCode::Reviewed]),
                    None,
                )
                .await
                .unwrap();
            assert_eq!(consumer1_reviewed_msgs.len(), 1);
        })
        .await;
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_update_agency_messages() {
        SetupPool::run(|setup| async move {
            let mut faber = Faber::setup(setup.pool_handle).await;
            let mut alice = create_test_alice_instance(&setup).await;

            let (alice_to_faber, faber_to_alice) = create_connected_connections(&mut alice, &mut faber).await;

            faber_to_alice
                .send_generic_message(&faber.profile, "Hello 1")
                .await
                .unwrap();
            faber_to_alice
                .send_generic_message(&faber.profile, "Hello 2")
                .await
                .unwrap();
            faber_to_alice
                .send_generic_message(&faber.profile, "Hello 3")
                .await
                .unwrap();

            thread::sleep(Duration::from_millis(100));

            let received = alice_to_faber
                .download_messages(&alice.agency_client, Some(vec![MessageStatusCode::Received]), None)
                .await
                .unwrap();
            assert_eq!(received.len(), 3);
            let uid = received[0].uid.clone();

            let reviewed = alice_to_faber
                .download_messages(&alice.agency_client, Some(vec![MessageStatusCode::Reviewed]), None)
                .await
                .unwrap();
            let reviewed_count_before = reviewed.len();

            let pairwise_did = alice_to_faber.pairwise_info().pw_did.clone();
            alice
                .agency_client
                .update_messages(
                    MessageStatusCode::Reviewed,
                    vec![UIDsByConn {
                        pairwise_did: pairwise_did.clone(),
                        uids: vec![uid.clone()],
                    }],
                )
                .await
                .unwrap();

            let received = alice_to_faber
                .download_messages(&alice.agency_client, Some(vec![MessageStatusCode::Received]), None)
                .await
                .unwrap();
            assert_eq!(received.len(), 2);

            let reviewed = alice_to_faber
                .download_messages(&alice.agency_client, Some(vec![MessageStatusCode::Reviewed]), None)
                .await
                .unwrap();
            let reviewed_count_after = reviewed.len();
            assert_eq!(reviewed_count_after, reviewed_count_before + 1);

            let specific_review = alice_to_faber
                .download_messages(
                    &alice.agency_client,
                    Some(vec![MessageStatusCode::Reviewed]),
                    Some(vec![uid.clone()]),
                )
                .await
                .unwrap();
            assert_eq!(specific_review[0].uid, uid);
        })
        .await;
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_download_messages_from_multiple_connections() {
        SetupPool::run(|setup| async move {
            let mut institution = Faber::setup(setup.pool_handle).await;
            let mut consumer1 = create_test_alice_instance(&setup).await;
            let mut consumer2 = create_test_alice_instance(&setup).await;

            let (consumer1_to_institution, institution_to_consumer1) =
                create_connected_connections(&mut consumer1, &mut institution).await;
            let (consumer2_to_institution, institution_to_consumer2) =
                create_connected_connections(&mut consumer2, &mut institution).await;

            consumer1_to_institution
                .send_generic_message(&consumer1.profile, "Hello Institution from consumer1")
                .await
                .unwrap();
            consumer2_to_institution
                .send_generic_message(&consumer2.profile, "Hello Institution from consumer2")
                .await
                .unwrap();

            let consumer1_msgs = institution_to_consumer1
                .download_messages(&institution.agency_client, None, None)
                .await
                .unwrap();
            assert_eq!(consumer1_msgs.len(), 2);

            let consumer2_msgs = institution_to_consumer2
                .download_messages(&institution.agency_client, None, None)
                .await
                .unwrap();
            assert_eq!(consumer2_msgs.len(), 2);
        })
        .await;
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_update_agent_webhook() {
        SetupPool::run(|_setup| async move {
            let wallet_config = WalletConfig {
                wallet_name: format!("wallet_{}", uuid::Uuid::new_v4().to_string()),
                wallet_key: settings::DEFAULT_WALLET_KEY.into(),
                wallet_key_derivation: settings::WALLET_KDF_RAW.into(),
                wallet_type: None,
                storage_config: None,
                storage_credentials: None,
                rekey: None,
                rekey_derivation_method: None,
            };

            let wallet_handle = aries_vcx::indy::wallet::create_and_open_wallet(&wallet_config)
                .await
                .unwrap();
            let wallet: Arc<dyn BaseWallet> = Arc::new(IndySdkWallet::new(wallet_handle));
            let mut client = AgencyClient::new();
            let agency_url = "http://localhost:8080";
            let agency_did = "VsKV7grR1BUE29mG2Fm2kX";
            let agency_vk = "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR";
            let (my_did, my_vk) = wallet.create_and_store_my_did(None, None).await.unwrap();
            client
                .provision_cloud_agent(
                    wallet.to_base_agency_client_wallet(),
                    &my_did,
                    &my_vk,
                    agency_did,
                    agency_vk,
                    agency_url,
                )
                .await
                .unwrap();
            let config = client.get_config().unwrap();
            let client = client
                .configure(wallet.to_base_agency_client_wallet(), &config)
                .unwrap();
            client.update_agent_webhook("https://example.org").await.unwrap();
            close_wallet(wallet_handle).await.unwrap();
        })
        .await;
    }
}
