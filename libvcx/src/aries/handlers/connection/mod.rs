pub mod agent_info;
pub mod connection;
pub mod messages;
mod invitee;
mod inviter;
mod util;

#[cfg(test)]
pub mod tests {
    use connection::tests::build_test_connection_inviter_requested;
    use utils::devsetup::SetupEmpty;
    use aries::messages::connection::invite::tests::_invitation_json;

    pub fn mock_connection() -> u32 {
        build_test_connection_inviter_requested()
    }

    fn _setup() {
        let _setup = SetupEmpty::init();
        ::settings::set_config_value(::settings::CONFIG_PROTOCOL_TYPE, "4.0");
    }

    fn _source_id() -> &'static str {
        "test connection"
    }

    mod aries {
        use aries;
        use aries::messages::a2a::A2AMessage;
        use aries::messages::ack::tests::_ack;
        use aries::test::{Alice, Faber};

        use super::*;

        #[test]
        #[cfg(feature = "aries")]
        fn test_create_connection_works() {
            _setup();
            let connection_handle = ::connection::create_connection(_source_id()).unwrap();
            assert!(::connection::is_valid_handle(connection_handle));
            assert_eq!(1, ::connection::get_state(connection_handle));
        }

        #[test]
        #[cfg(feature = "aries")]
        fn test_create_connection_with_invite_works() {
            _setup();
            let connection_handle = ::connection::create_connection_with_invite(_source_id(), &_invitation_json()).unwrap();
            assert!(::connection::is_valid_handle(connection_handle));
            assert_eq!(2, ::connection::get_state(connection_handle));
        }

        #[test]
        #[cfg(feature = "agency_v2")]
        fn test_connection_send_works() {
            _setup();
            let mut faber = Faber::setup();
            let mut alice = Alice::setup();

            let invite = faber.create_invite();
            alice.accept_invite(&invite);

            faber.update_state(3);
            alice.update_state(4);
            faber.update_state(4);

            let uid: String;
            let message = _ack();

            info!("test_connection_send_works:: Test if Send Message works");
            {
                faber.activate();
                ::connection::send_message(faber.connection_handle, message.to_a2a_message()).unwrap();
            }

            {
                info!("test_connection_send_works:: Test if Get Messages works");
                alice.activate();

                let messages = ::connection::get_messages(alice.connection_handle).unwrap();
                assert_eq!(1, messages.len());

                uid = messages.keys().next().unwrap().clone();
                let received_message = messages.values().next().unwrap().clone();

                match received_message {
                    A2AMessage::Ack(received_message) => assert_eq!(message, received_message.clone()),
                    _ => assert!(false)
                }
            }

            let _res = ::messages::get_message::download_messages_noauth(None, None, Some(vec![uid.clone()])).unwrap();

            info!("test_connection_send_works:: Test if Get Message by id works");
            {
                alice.activate();

                let message = ::connection::get_message_by_id(alice.connection_handle, uid.clone()).unwrap();

                match message {
                    A2AMessage::Ack(ack) => assert_eq!(_ack(), ack),
                    _ => assert!(false)
                }
            }

            info!("test_connection_send_works:: Test if Update Message Status works");
            {
                alice.activate();

                ::connection::update_message_status(alice.connection_handle, uid).unwrap();
                let messages = ::connection::get_messages(alice.connection_handle).unwrap();
                assert_eq!(0, messages.len());
            }

            info!("test_connection_send_works:: Test if Send Basic Message works");
            {
                faber.activate();

                let basic_message = r#"Hi there"#;
                ::connection::send_generic_message(faber.connection_handle, basic_message).unwrap();

                alice.activate();

                let messages = ::connection::get_messages(alice.connection_handle).unwrap();
                assert_eq!(1, messages.len());

                let uid = messages.keys().next().unwrap().clone();
                let message = messages.values().next().unwrap().clone();

                match message {
                    A2AMessage::BasicMessage(message) => assert_eq!(basic_message, message.content),
                    _ => assert!(false)
                }
                ::connection::update_message_status(alice.connection_handle, uid).unwrap();
            }

            info!("test_connection_send_works:: Test if Download Messages");
            {
                use messages::get_message::{download_messages_noauth, MessageByConnection, Message};

                let credential_offer = ::aries::messages::issuance::credential_offer::tests::_credential_offer();

                faber.activate();
                ::connection::send_message(faber.connection_handle, credential_offer.to_a2a_message()).unwrap();

                alice.activate();

                let messages: Vec<MessageByConnection> = download_messages_noauth(None, Some(vec!["MS-103".to_string()]), None).unwrap();
                let message: Message = messages[0].msgs[0].clone();
                let decrypted_msg = message.decrypted_msg.unwrap();
                let _payload: aries::messages::issuance::credential_offer::CredentialOffer = ::serde_json::from_str(&decrypted_msg).unwrap();

                ::connection::update_message_status(alice.connection_handle, message.uid).unwrap();
            }

            info!("test_connection_send_works:: Test Helpers");
            {
                faber.activate();

                ::connection::get_pw_did(faber.connection_handle).unwrap();
                ::connection::get_pw_verkey(faber.connection_handle).unwrap();
                ::connection::get_their_pw_verkey(faber.connection_handle).unwrap();
                ::connection::get_source_id(faber.connection_handle).unwrap();
            }
        }

        #[test]
        #[cfg(feature = "aries")]
        fn test_get_connection_state_works() {
            _setup();
            let connection_handle = ::connection::create_connection(_source_id()).unwrap();
            assert_eq!(1, ::connection::get_state(connection_handle));
        }

        #[test]
        #[cfg(feature = "aries")]
        fn test_connection_delete() {
            _setup();
            warn!(">> test_connection_delete going to create connection");
            let connection_handle = ::connection::create_connection(_source_id()).unwrap();
            warn!(">> test_connection_delete checking is valid handle");
            assert!(::connection::is_valid_handle(connection_handle));

            ::connection::release(connection_handle).unwrap();
            assert!(!::connection::is_valid_handle(connection_handle));
        }
    }
}

