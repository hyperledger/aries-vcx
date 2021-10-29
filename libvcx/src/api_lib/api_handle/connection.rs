use std::collections::HashMap;

use serde_json;

use aries_vcx::agency_client::get_message::MessageByConnection;
use aries_vcx::agency_client::MessageStatusCode;
use aries_vcx::utils::error;

use crate::api_lib::api_handle::agent::PUBLIC_AGENT_MAP;
use crate::api_lib::api_handle::object_cache::ObjectCache;
use crate::aries_vcx::handlers::connection::connection::Connection;
use crate::aries_vcx::messages::a2a::A2AMessage;
use crate::aries_vcx::messages::connection::invite::Invitation as InvitationV3;
use crate::aries_vcx::messages::connection::request::Request;
use crate::error::prelude::*;

lazy_static! {
    pub static ref CONNECTION_MAP: ObjectCache<Connection> = ObjectCache::<Connection>::new("connections-cache");
}

pub fn is_valid_handle(handle: u32) -> bool {
    CONNECTION_MAP.has_handle(handle)
}

pub fn get_agent_did(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        Ok(connection.cloud_agent_info().agent_did.to_string())
    })
}

pub fn get_agent_verkey(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        Ok(connection.cloud_agent_info().agent_vk.clone())
    })
}

pub fn get_pw_did(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        Ok(connection.pairwise_info().pw_did.to_string())
    })
}

pub fn get_pw_verkey(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        Ok(connection.pairwise_info().pw_vk.clone())
    })
}

pub fn get_their_pw_did(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        connection.remote_did().map_err(|err| err.into())
    })
}

pub fn get_their_pw_verkey(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        connection.remote_vk().map_err(|err| err.into())
    })
}

pub fn get_thread_id(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        connection.get_thread_id().map_err(|err| err.into())
    })
}

pub fn get_state(handle: u32) -> u32 {
    trace!("get_state >>> handle = {:?}", handle);
    CONNECTION_MAP.get(handle, |connection| {
        Ok(connection.get_state().into())
    }).unwrap_or(0)
}

pub fn get_source_id(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        Ok(connection.get_source_id())
    })
}

pub fn store_connection(connection: Connection) -> VcxResult<u32> {
    CONNECTION_MAP.add(connection)
        .or(Err(VcxError::from(VcxErrorKind::CreateConnection)))
}

pub fn create_connection(source_id: &str) -> VcxResult<u32> {
    trace!("create_connection >>> source_id: {}", source_id);
    let connection = Connection::create(source_id, true)?;
    return store_connection(connection);
}

pub fn create_connection_with_invite(source_id: &str, details: &str) -> VcxResult<u32> {
    debug!("create connection {} with invite {}", source_id, details);
    if let Some(invitation) = serde_json::from_str::<InvitationV3>(details).ok() {
        let connection = Connection::create_with_invite(source_id, invitation, true)?;
        store_connection(connection)
    } else {
        Err(VcxError::from_msg(VcxErrorKind::InvalidJson, "Used invite has invalid structure")) // TODO: Specific error type
    }
}

pub fn create_connection_with_connection_request(request: &str, agent_handle: u32) -> VcxResult<u32> {
    PUBLIC_AGENT_MAP.get(agent_handle, |agent| {
        let request: Request = serde_json::from_str(request)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize connection request: {:?}", err)))?;
        let connection = Connection::create_with_connection_request(request, &agent)?;
        store_connection(connection)
    })
}

pub fn send_generic_message(connection_handle: u32, msg: &str) -> VcxResult<String> {
    CONNECTION_MAP.get(connection_handle, |connection| {
        connection.send_generic_message(msg).map_err(|err| err.into())
    })
}

pub fn update_state_with_message(handle: u32, message: &str) -> VcxResult<u32> {
    let message: A2AMessage = serde_json::from_str(&message)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Failed to deserialize message {} into A2AMessage, err: {:?}", message, err)))?;

    CONNECTION_MAP.get_mut(handle, |connection| {
        connection.update_state_with_message(&message)?;
        Ok(error::SUCCESS.code_num)
    })
}

// fn get_bootstrap_agent_messages(remote_vk: VcxResult<String>, bootstrap_agent_info: Option<&PairwiseInfo>) -> VcxResult<Option<(HashMap<String, A2AMessage>, PairwiseInfo)>> {
//     let expected_sender_vk = match remote_vk {
//         Ok(vk) => vk,
//         Err(_) => return Ok(None)
//     };
//     if let Some(bootstrap_agent_info) = bootstrap_agent_info {
//         let messages = bootstrap_agent_info.get_messages(&expected_sender_vk)?;
//         return Ok(Some((messages, bootstrap_agent_info.clone())));
//     }
//     Ok(None)
// }

pub fn update_state(handle: u32) -> VcxResult<u32> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection.update_state() {
            Ok(_) => Ok(error::SUCCESS.code_num),
            Err(err) => Err(err.into())
        }
    })
}

pub fn delete_connection(handle: u32) -> VcxResult<u32> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        connection.delete()?;
        Ok(error::SUCCESS.code_num)
    })
        .map(|_| error::SUCCESS.code_num)
        .or(Err(VcxError::from(VcxErrorKind::DeleteConnection)))
        .and(release(handle))
        .and_then(|_| Ok(error::SUCCESS.code_num))
}

pub fn connect(handle: u32) -> VcxResult<Option<String>> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        connection.connect()?;
        let invitation = connection.get_invite_details()
            .map(|invitation| match invitation {
                InvitationV3::Pairwise(invitation) => json!(invitation.to_a2a_message()).to_string(),
                InvitationV3::Public(invitation) => json!(invitation.to_a2a_message()).to_string()
            });
        Ok(invitation)
    })
}

pub fn to_string(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        connection.to_string().map_err(|err| err.into())
    })
}

pub fn from_string(connection_data: &str) -> VcxResult<u32> {
    let connection = Connection::from_string(connection_data)?;
    let handle = CONNECTION_MAP.add(connection)?;
    Ok(handle)
}

pub fn release(handle: u32) -> VcxResult<()> {
    CONNECTION_MAP.release(handle)
        .or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn release_all() {
    CONNECTION_MAP.drain().ok();
}

pub fn get_invite_details(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        connection.get_invite_details()
            .map(|invitation| match invitation {
                InvitationV3::Pairwise(invitation) => json!(invitation.to_a2a_message()).to_string(),
                InvitationV3::Public(invitation) => json!(invitation.to_a2a_message()).to_string()
            })
            .ok_or(VcxError::from(VcxErrorKind::ActionNotSupported))
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}


pub fn get_messages(handle: u32) -> VcxResult<HashMap<String, A2AMessage>> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        connection.get_messages().map_err(|err| err.into())
    })
}

pub fn update_message_status(handle: u32, uid: String) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        connection.update_message_status(uid.clone()).map_err(|err| err.into())
    })
}

pub fn get_message_by_id(handle: u32, msg_id: String) -> VcxResult<A2AMessage> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        connection.get_message_by_id(&msg_id).map_err(|err| err.into())
    })
}

pub fn send_message(handle: u32, message: A2AMessage) -> VcxResult<()> {
    trace!("connection::send_message >>>");
    let send_message = send_message_closure(handle)?;
    send_message(&message).map_err(|err| err.into())
}

pub fn send_message_closure(handle: u32) -> VcxResult<impl Fn(&A2AMessage) -> aries_vcx::error::VcxResult<()>> {
    CONNECTION_MAP.get(handle, |connection| {
        return connection.send_message_closure().map_err(|err| err.into());
    })
}

pub fn is_v3_connection(connection_handle: u32) -> VcxResult<bool> {
    CONNECTION_MAP.get(connection_handle, |_| {
        Ok(true)
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn send_ping(connection_handle: u32, comment: Option<String>) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(connection_handle, |connection| {
        connection.send_ping(comment.clone()).map_err(|err| err.into())
    })
}

pub fn send_discovery_features(connection_handle: u32, query: Option<String>, comment: Option<String>) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(connection_handle, |connection| {
        connection.send_discovery_features(query.clone(), comment.clone()).map_err(|err| err.into())
    })
}

pub fn get_connection_info(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        connection.get_connection_info().map_err(|err| err.into())
    })
}

pub fn download_messages(conn_handles: Vec<u32>, status_codes: Option<Vec<MessageStatusCode>>, uids: Option<Vec<String>>) -> VcxResult<Vec<MessageByConnection>> {
    trace!("download_messages >>> cann_handles: {:?}, status_codes: {:?}, uids: {:?}", conn_handles, status_codes, uids);
    let mut res = Vec::new();
    let mut connections = Vec::new();
    for conn_handle in conn_handles {
        let connection = CONNECTION_MAP.get(
            conn_handle, |connection| {
                Ok(connection.clone())
            },
        )?;
        connections.push(connection)
    };
    for connection in connections {
        let msgs = connection.download_messages(status_codes.clone(), uids.clone())?;
        res.push(MessageByConnection { pairwise_did: connection.pairwise_info().pw_did.clone(), msgs });
    }
    trace!("download_messages <<< res: {:?}", res);
    Ok(res)
}

#[cfg(test)]
pub mod tests {
    use aries_vcx;
    use aries_vcx::agency_client::mocking::AgencyMockDecrypted;
    use aries_vcx::messages::connection::invite::test_utils::{_pairwise_invitation_json, _public_invitation_json};
    use aries_vcx::utils::constants;
    use aries_vcx::utils::devsetup::{SetupEmpty, SetupMocks};
    use aries_vcx::utils::mockdata::mockdata_connection::{ARIES_CONNECTION_ACK, ARIES_CONNECTION_INVITATION, ARIES_CONNECTION_REQUEST, CONNECTION_SM_INVITEE_COMPLETED};

    use crate::api_lib::api_handle::agent::create_public_agent;
    use crate::api_lib::api_handle::connection;
    use crate::api_lib::VcxStateType;

    use super::*;
    use aries_vcx::settings;

    pub fn mock_connection() -> u32 {
        build_test_connection_inviter_requested()
    }

    fn _setup() {
        let _setup = SetupEmpty::init();
    }

    fn _source_id() -> &'static str {
        "test connection"
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_connection_works() {
        let _setup = SetupMocks::init();
        let connection_handle = connection::create_connection(_source_id()).unwrap();
        assert!(connection::is_valid_handle(connection_handle));
        assert_eq!(0, connection::get_state(connection_handle));
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_connection_with_pairwise_invite() {
        let _setup = SetupMocks::init();
        let connection_handle = connection::create_connection_with_invite(_source_id(), &_pairwise_invitation_json()).unwrap();
        assert!(connection::is_valid_handle(connection_handle));
        assert_eq!(1, connection::get_state(connection_handle));
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_connection_with_public_invite() {
        let _setup = SetupMocks::init();
        let connection_handle = connection::create_connection_with_invite(_source_id(), &_public_invitation_json()).unwrap();
        assert!(connection::is_valid_handle(connection_handle));
        assert_eq!(1, connection::get_state(connection_handle));
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_connection_with_request() {
        let _setup = SetupMocks::init();
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let agent_handle = create_public_agent("test", &institution_did).unwrap();
        let connection_handle = connection::create_connection_with_connection_request(ARIES_CONNECTION_REQUEST, agent_handle).unwrap();
        assert!(connection::is_valid_handle(connection_handle));
        assert_eq!(2, connection::get_state(connection_handle));
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_get_connection_state_works() {
        let _setup = SetupMocks::init();
        let connection_handle = connection::create_connection(_source_id()).unwrap();
        assert_eq!(0, connection::get_state(connection_handle));
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_connection_delete() {
        let _setup = SetupMocks::init();
        warn!(">> test_connection_delete going to create connection");
        let connection_handle = connection::create_connection(_source_id()).unwrap();
        warn!(">> test_connection_delete checking is valid handle");
        assert!(connection::is_valid_handle(connection_handle));

        connection::release(connection_handle).unwrap();
        assert!(!connection::is_valid_handle(connection_handle));
    }

    pub fn build_test_connection_inviter_null() -> u32 {
        let handle = create_connection("faber_to_alice").unwrap();
        handle
    }

    pub fn build_test_connection_inviter_invited() -> u32 {
        let handle = create_connection("faber_to_alice").unwrap();
        connect(handle).unwrap();
        handle
    }

    pub fn build_test_connection_invitee_completed() -> u32 {
        from_string(CONNECTION_SM_INVITEE_COMPLETED).unwrap()
    }

    pub fn build_test_connection_inviter_requested() -> u32 {
        let handle = build_test_connection_inviter_invited();
        update_state_with_message(handle, ARIES_CONNECTION_REQUEST).unwrap();
        handle
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_connection() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_create_connection").unwrap();
        assert_eq!(get_state(handle), VcxStateType::VcxStateNone as u32);


        connect(handle).unwrap();
        assert_eq!(get_pw_did(handle).unwrap(), constants::DID);
        assert_eq!(get_pw_verkey(handle).unwrap(), constants::VERKEY);

        AgencyMockDecrypted::set_next_decrypted_response(constants::GET_MESSAGES_DECRYPTED_RESPONSE);
        AgencyMockDecrypted::set_next_decrypted_message(ARIES_CONNECTION_REQUEST);
        update_state(handle).unwrap();
        assert_eq!(get_state(handle), VcxStateType::VcxStateRequestReceived as u32);

        AgencyMockDecrypted::set_next_decrypted_response(constants::GET_MESSAGES_DECRYPTED_RESPONSE);
        AgencyMockDecrypted::set_next_decrypted_message(ARIES_CONNECTION_ACK);
        update_state(handle).unwrap();
        assert_eq!(get_state(handle), VcxStateType::VcxStateAccepted as u32);

        AgencyMockDecrypted::set_next_decrypted_response(constants::DELETE_CONNECTION_DECRYPTED_RESPONSE);
        assert_eq!(delete_connection(handle).unwrap(), 0);

        // This errors b/c we release handle in delete connection
        assert!(release(handle).is_err());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_drop_create() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_create_drop_create").unwrap();

        assert_eq!(get_state(handle), VcxStateType::VcxStateNone as u32);
        let did1 = get_pw_did(handle).unwrap();

        release(handle).unwrap();

        let handle2 = create_connection("test_create_drop_create").unwrap();

        assert_eq!(get_state(handle2), VcxStateType::VcxStateNone as u32);
        let did2 = get_pw_did(handle2).unwrap();

        assert_ne!(handle, handle2);
        assert_eq!(did1, did2);

        release(handle2).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_connection_release_fails() {
        let _setup = SetupEmpty::init();

        let rc = release(1);
        assert_eq!(rc.unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_get_state_fails() {
        let _setup = SetupEmpty::init();

        let state = get_state(1);
        assert_eq!(state, VcxStateType::VcxStateNone as u32);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_get_string_fails() {
        let _setup = SetupEmpty::init();

        let rc = to_string(0);
        assert_eq!(rc.unwrap_err().kind(), VcxErrorKind::InvalidHandle);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_get_service_endpoint() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_get_qr_code_data").unwrap();

        connect(handle).unwrap();

        let details = get_invite_details(handle).unwrap();
        assert!(details.contains("\"serviceEndpoint\":"));

        assert_eq!(get_invite_details(0).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_retry_connection() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_serialize_deserialize").unwrap();

        assert_eq!(get_state(handle), VcxStateType::VcxStateNone as u32);

        connect(handle).unwrap();
        connect(handle).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_release_all() {
        let _setup = SetupMocks::init();

        let h1 = create_connection("rel1").unwrap();
        let h2 = create_connection("rel2").unwrap();
        let h3 = create_connection("rel3").unwrap();
        let h4 = create_connection("rel4").unwrap();
        let h5 = create_connection("rel5").unwrap();
        release_all();
        assert_eq!(release(h1).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
        assert_eq!(release(h2).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
        assert_eq!(release(h3).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
        assert_eq!(release(h4).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
        assert_eq!(release(h5).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_with_valid_invite_details() {
        let _setup = SetupMocks::init();

        let handle = create_connection_with_invite("alice", ARIES_CONNECTION_INVITATION).unwrap();
        connect(handle).unwrap();

        let handle_2 = create_connection_with_invite("alice", ARIES_CONNECTION_INVITATION).unwrap();
        connect(handle_2).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_process_acceptance_message() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_process_acceptance_message").unwrap();
        assert_eq!(error::SUCCESS.code_num, update_state_with_message(handle, ARIES_CONNECTION_REQUEST).unwrap());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_connection_handle_is_found() {
        let _setup = SetupMocks::init();
        let handle = create_connection_with_invite("alice", ARIES_CONNECTION_INVITATION).unwrap();

        CONNECTION_MAP.get_mut(handle, |_connection| {
            Ok(())
        }).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_send_generic_message_fails_with_invalid_connection() {
        let _setup = SetupMocks::init();

        let handle = connection::tests::build_test_connection_inviter_invited();

        let err = send_generic_message(handle, "this is the message").unwrap_err();
        assert_eq!(err.kind(), VcxErrorKind::NotReady);
    }
}
