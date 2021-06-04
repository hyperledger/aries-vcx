pub mod pairwise_info;
pub mod cloud_agent;
pub mod legacy_agent_info;
pub mod connection;
mod invitee;
mod inviter;
mod util;

#[cfg(test)]
pub mod tests {
    use crate::aries::messages::connection::invite::tests::_invitation_json;
    use crate::connection::tests::build_test_connection_inviter_requested;
    use crate::utils::devsetup::{SetupEmpty, SetupMocks};

    pub fn mock_connection() -> u32 {
        build_test_connection_inviter_requested()
    }

    fn _setup() {
        let _setup = SetupEmpty::init();
    }

    fn _source_id() -> &'static str {
        "test connection"
    }

    mod aries {
        use crate::aries;
        use crate::aries::messages::a2a::A2AMessage;
        use crate::aries::messages::ack::tests::_ack;
        use crate::connection;

        use super::*;
        use crate::utils::devsetup_agent::test::{Faber, Alice, TestAgent};

        #[test]
        #[cfg(feature = "aries")]
        fn test_create_connection_works() {
            _setup();
            let connection_handle = connection::create_connection(_source_id()).unwrap();
            assert!(connection::is_valid_handle(connection_handle));
            assert_eq!(0, connection::get_state(connection_handle));
        }

        #[test]
        #[cfg(feature = "aries")]
        fn test_create_connection_with_invite_works() {
            let _setup = SetupMocks::init();
            let connection_handle = connection::create_connection_with_invite(_source_id(), &_invitation_json()).unwrap();
            assert!(connection::is_valid_handle(connection_handle));
            assert_eq!(1, connection::get_state(connection_handle));
        }

        #[test]
        #[cfg(feature = "aries")]
        fn test_get_connection_state_works() {
            _setup();
            let connection_handle = connection::create_connection(_source_id()).unwrap();
            assert_eq!(0, connection::get_state(connection_handle));
        }

        #[test]
        #[cfg(feature = "aries")]
        fn test_connection_delete() {
            _setup();
            warn!(">> test_connection_delete going to create connection");
            let connection_handle = connection::create_connection(_source_id()).unwrap();
            warn!(">> test_connection_delete checking is valid handle");
            assert!(connection::is_valid_handle(connection_handle));

            connection::release(connection_handle).unwrap();
            assert!(!connection::is_valid_handle(connection_handle));
        }
    }
}

