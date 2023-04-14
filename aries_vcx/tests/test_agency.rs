#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_json;

pub mod utils;

#[cfg(test)]
mod integration_tests {
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    use agency_client::agency_client::AgencyClient;
    use agency_client::api::downloaded_message::DownloadedMessage;
    use agency_client::messages::update_message::UIDsByConn;
    use agency_client::MessageStatusCode;
    use aries_vcx::global::settings;
    use aries_vcx::utils::devsetup::SetupPool;
    use aries_vcx_core::wallet::agency_client_wallet::ToBaseAgencyClientWallet;
    use messages::msg_fields::protocols::cred_issuance::offer_credential::OfferCredentialDecorators;
    use messages::msg_fields::protocols::cred_issuance::{CredentialAttr, CredentialPreview};

    use crate::utils::devsetup_agent::test_utils::create_test_alice_instance;
    use crate::utils::devsetup_agent::test_utils::Faber;
    use crate::utils::scenarios::test_utils::create_connected_connections;

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_send_and_download_messages() {
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
    #[ignore]
    async fn test_agency_pool_connection_send_works() {
        use aries_vcx::handlers::util::AttachmentId;
        use messages::{
            decorators::thread::Thread,
            msg_fields::protocols::{
                cred_issuance::offer_credential::{OfferCredential, OfferCredentialContent},
                notification::{Ack, AckContent, AckDecorators, AckStatus},
            },
            AriesMessage,
        };

        SetupPool::run(|setup| async move {
            let mut faber = Faber::setup(setup.pool_handle).await;
            let mut alice = create_test_alice_instance(&setup).await;

            let invite = faber.create_invite().await;
            alice.accept_invite(&invite).await;

            faber.update_state(3).await;
            alice.update_state(4).await;
            faber.update_state(4).await;

            let uid: String;
            let id = "test_id".to_owned();
            let content = AckContent::new(AckStatus::Ok);
            let thread = Thread::new("testid".to_owned());
            let decorators = AckDecorators::new(thread);
            let message = Ack::with_decorators(id, content, decorators);

            info!("test_connection_send_works:: Test if Send Message works");
            {
                faber.connection.send_message_closure(&faber.profile).await.unwrap()(AriesMessage::from(
                    message.clone(),
                ))
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
                    AriesMessage::Notification(received_message) => assert_eq!(message, received_message.clone()),
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

                let id = "test_id".to_owned();
                let content = AckContent::new(AckStatus::Ok);
                let thread = Thread::new("testid".to_owned());
                let decorators = AckDecorators::new(thread);
                let _ack = Ack::with_decorators(id, content, decorators);

                match message {
                    AriesMessage::Notification(ack) => assert_eq!(_ack, ack),
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
                    AriesMessage::BasicMessage(message) => assert_eq!(basic_message, message.content.content),
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
                let attachment_json = json!({
                    "schema_id":"NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0",
                    "cred_def_id":"NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:TAG1"
                });

                let attach_type = messages::decorators::attachment::AttachmentType::Base64(base64::encode(
                    &attachment_json.to_string(),
                ));
                let attach_data = messages::decorators::attachment::AttachmentData::new(attach_type);
                let mut attach = messages::decorators::attachment::Attachment::new(attach_data);
                attach.id = Some(AttachmentId::CredentialOffer.as_ref().to_owned());
                attach.mime_type = Some(messages::misc::MimeType::Json);

                let id = "test_id".to_owned();
                let preview =
                    CredentialPreview::new(vec![CredentialAttr::new("attribute".to_owned(), "value".to_owned())]);
                let content = OfferCredentialContent::new(preview, vec![attach]);
                let decorators = OfferCredentialDecorators::default();

                let credential_offer = OfferCredential::with_decorators(id, content, decorators);

                faber.connection.send_message_closure(&faber.profile).await.unwrap()(AriesMessage::from(
                    credential_offer,
                ))
                .await
                .unwrap();

                let msgs = alice
                    .connection
                    .download_messages(&alice.agency_client, Some(vec![MessageStatusCode::Received]), None)
                    .await
                    .unwrap();
                let message: DownloadedMessage = msgs[0].clone();
                let _payload: OfferCredential = serde_json::from_str(&message.decrypted_msg).unwrap();

                alice
                    .connection
                    .update_message_status(&message.uid, &alice.agency_client)
                    .await
                    .unwrap();
            }
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_download_messages() {
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

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_update_agency_messages() {
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

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_download_messages_from_multiple_connections() {
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

    #[tokio::test]
    #[ignore]
    async fn test_agency_pool_update_agent_webhook() {
        use aries_vcx_core::{
            indy::wallet::{create_and_open_wallet, WalletConfig},
            wallet::{base_wallet::BaseWallet, indy_wallet::IndySdkWallet},
        };

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

            let wallet_handle = create_and_open_wallet(&wallet_config).await.unwrap();
            let wallet: Arc<dyn BaseWallet> = Arc::new(IndySdkWallet::new(wallet_handle));
            let mut client = AgencyClient::new();
            let agency_url = "http://localhost:8080".parse().unwrap();
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
        })
        .await;
    }
}
