use std::collections::HashMap;

use serde_json;

use aries::handlers::connection::agent_info::AgentInfo;
use aries::handlers::connection::connection::{Connection as ConnectionV3, SmConnectionState};
use aries::messages::a2a::A2AMessage;
use aries::messages::connection::did_doc::DidDoc;
use aries::messages::connection::invite::Invitation as InvitationV3;
use error::prelude::*;
use messages;
use messages::get_message::Message;
use messages::SerializableObjectWithState;
use settings;
use settings::ProtocolTypes;
use utils::error;
use utils::object_cache::ObjectCache;

lazy_static! {
    static ref CONNECTION_MAP: ObjectCache<ConnectionV3> = ObjectCache::<ConnectionV3>::new("connections-cache");
}

pub fn create_agent_keys(source_id: &str, pw_did: &str, pw_verkey: &str) -> VcxResult<(String, String)> {
    debug!("creating pairwise keys on agent for connection {}", source_id);

    let (agent_did, agent_verkey) = messages::create_keys()
        .for_did(pw_did)?
        .for_verkey(pw_verkey)?
        .version(&Some(settings::get_protocol_type()))?
        .send_secure()
        .map_err(|err| err.extend("Cannot create pairwise keys"))?;

    Ok((agent_did, agent_verkey))
}

pub fn get_their_pw_verkey(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        connection.remote_vk()
    })
}

pub fn is_valid_handle(handle: u32) -> bool {
    CONNECTION_MAP.has_handle(handle)
}

pub fn get_agent_did(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        Ok(connection.agent_info().agent_did.to_string())
    })
}

pub fn get_pw_did(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        Ok(connection.agent_info().pw_did.to_string())
    })
}

pub fn get_their_pw_did(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        connection.remote_did()
    })
}

pub fn get_agent_verkey(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        Ok(connection.agent_info().agent_vk.clone())
    })
}

pub fn get_version(_handle: u32) -> VcxResult<Option<ProtocolTypes>> {
    Ok(Some(settings::get_protocol_type()))
}

pub fn get_pw_verkey(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        Ok(connection.agent_info().pw_vk.clone())
    })
}

pub fn get_state(handle: u32) -> u32 {
    trace!("get_state >>> handle = {:?}", handle);
    CONNECTION_MAP.get(handle, |connection| {
        Ok(connection.state())
    }).unwrap_or(0)
}

pub fn get_source_id(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        Ok(connection.get_source_id())
    })
}

fn store_connection(connection: ConnectionV3) -> VcxResult<u32> {
    CONNECTION_MAP.add(connection)
        .or(Err(VcxError::from(VcxErrorKind::CreateConnection)))
}

pub fn create_connection(source_id: &str) -> VcxResult<u32> {
    trace!("create_connection >>> source_id: {}", source_id);
    let connection = ConnectionV3::create(source_id);
    return store_connection(connection);
}

pub fn create_connection_with_invite(source_id: &str, details: &str) -> VcxResult<u32> {
    debug!("create connection {} with invite {}", source_id, details);
    if let Some(invitation) = serde_json::from_str::<InvitationV3>(details).ok() {
        let connection = ConnectionV3::create_with_invite(source_id, invitation)?;
        store_connection(connection)
    } else {
        Err(VcxError::from_msg(VcxErrorKind::InvalidJson, "Used invite has invalid structure")) // TODO: Specific error type
    }
}

pub fn send_generic_message(connection_handle: u32, msg: &str, msg_options: &str) -> VcxResult<String> {
    CONNECTION_MAP.get(connection_handle, |connection| {
        connection.send_generic_message(msg, msg_options)
    })
}

pub fn update_state_with_message(handle: u32, message: A2AMessage) -> VcxResult<u32> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        connection.update_state_with_message(&message)?;
        Ok(error::SUCCESS.code_num)
    })
}

pub fn update_state(handle: u32) -> VcxResult<u32> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        connection.update_state()?;
        Ok(error::SUCCESS.code_num)
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
        Ok(connection.get_invite_details())
    })
}

pub fn to_string(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        let (state, data, source_id) = connection.to_owned().into();
        let object = SerializableObjectWithState::V3 { data, state, source_id };

        ::serde_json::to_string(&object)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidState, format!("Cannot serialize Connection: {:?}", err)))
    })
}

pub fn from_string(connection_data: &str) -> VcxResult<u32> {
    let object: SerializableObjectWithState<AgentInfo, SmConnectionState> = ::serde_json::from_str(connection_data)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize Connection: {:?}", err)))?;

    let handle = match object {
        SerializableObjectWithState::V3 { data, state, source_id } => {
            CONNECTION_MAP.add((state, data, source_id).into())?
        }
        _ => return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Unexpected format of serialized connection: {:?}", object)))
    };
    Ok(handle)
}

pub fn release(handle: u32) -> VcxResult<()> {
    CONNECTION_MAP.release(handle)
        .or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn release_all() {
    CONNECTION_MAP.drain().ok();
}

pub fn get_invite_details(handle: u32, _abbreviated: bool) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        return connection.get_invite_details()
            .ok_or(VcxError::from(VcxErrorKind::ActionNotSupported));
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

impl Into<(SmConnectionState, AgentInfo, String)> for ConnectionV3 {
    fn into(self) -> (SmConnectionState, AgentInfo, String) {
        (self.state_object(), self.agent_info().to_owned(), self.source_id())
    }
}

impl From<(SmConnectionState, AgentInfo, String)> for ConnectionV3 {
    fn from((state, agent_info, source_id): (SmConnectionState, AgentInfo, String)) -> ConnectionV3 {
        ConnectionV3::from_parts(source_id, agent_info, state)
    }
}

pub fn get_messages(handle: u32) -> VcxResult<HashMap<String, A2AMessage>> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        connection.get_messages()
    })
}

pub fn update_message_status(handle: u32, uid: String) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        connection.update_message_status(uid.clone())
    })
}

pub fn get_message_by_id(handle: u32, msg_id: String) -> VcxResult<A2AMessage> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        connection.get_message_by_id(&msg_id)
    })
}

pub fn decode_message(handle: u32, message: Message) -> VcxResult<A2AMessage> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        connection.decode_message(&message)
    })
}

pub fn send_message(handle: u32, message: A2AMessage) -> VcxResult<()> {
    trace!("connection::send_message >>>");
    CONNECTION_MAP.get_mut(handle, |connection| {
        connection.send_message(&message)
    })
}

pub fn send_message_to_self_endpoint(message: A2AMessage, did_doc: &DidDoc) -> VcxResult<()> {
    ConnectionV3::send_message_to_self_endpoint(&message, did_doc)
}

pub fn is_v3_connection(connection_handle: u32) -> VcxResult<bool> {
    CONNECTION_MAP.get(connection_handle, |_| {
        Ok(true)
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn send_ping(connection_handle: u32, comment: Option<String>) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(connection_handle, |connection| {
        connection.send_ping(comment.clone())
    })
}

pub fn send_discovery_features(connection_handle: u32, query: Option<String>, comment: Option<String>) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(connection_handle, |connection| {
        connection.send_discovery_features(query.clone(), comment.clone())
    })
}

pub fn get_connection_info(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        connection.get_connection_info()
    })
}

#[cfg(test)]
pub mod tests {
    use std::thread;
    use std::time::Duration;

    use serde_json::Value;

    use api_c::VcxStateType;
    use messages::get_message::download_messages;
    use messages::MessageStatusCode;
    use utils::constants::*;
    use utils::constants;
    use utils::devsetup::*;
    use utils::httpclient::AgencyMockDecrypted;
    use utils::mockdata::mockdata_connection::{ARIES_CONNECTION_ACK, ARIES_CONNECTION_INVITATION, ARIES_CONNECTION_REQUEST, CONNECTION_SM_INVITEE_COMPLETED, CONNECTION_SM_INVITEE_INVITED, CONNECTION_SM_INVITEE_REQUESTED, CONNECTION_SM_INVITER_COMPLETED};

    use super::*;

    pub fn build_test_connection_inviter_null() -> u32 {
        let handle = create_connection("faber_to_alice").unwrap();
        handle
    }

    pub fn build_test_connection_inviter_invited() -> u32 {
        let handle = create_connection("faber_to_alice").unwrap();
        connect(handle).unwrap();
        handle
    }

    pub fn build_test_connection_inviter_requested() -> u32 {
        let handle = build_test_connection_inviter_invited();
        let msg: A2AMessage = serde_json::from_str(ARIES_CONNECTION_REQUEST).unwrap();
        update_state_with_message(handle, msg).unwrap();
        handle
    }

    pub fn create_connected_connections(consumer_handle: Option<u32>) -> (u32, u32) {
        debug!("Institution is going to create connection.");
        ::utils::devsetup::set_institution();
        let faber_to_alice = create_connection("alice").unwrap();
        let _my_public_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let details = connect(faber_to_alice).unwrap().unwrap();
        update_state(faber_to_alice).unwrap();

        ::utils::devsetup::set_consumer(consumer_handle);
        debug!("Consumer is going to accept connection invitation.");
        let alice_to_faber = create_connection_with_invite("faber", &details).unwrap();
        connect(alice_to_faber).unwrap();
        update_state(alice_to_faber).unwrap();
        // assert_eq!(VcxStateType::VcxStateRequestReceived as u32, get_state(faber));

        debug!("Institution is going to process connection request.");
        ::utils::devsetup::set_institution();
        thread::sleep(Duration::from_millis(500));
        update_state(faber_to_alice).unwrap();

        debug!("Consumer is going to complete the connection protocol.");
        ::utils::devsetup::set_consumer(consumer_handle);
        update_state(alice_to_faber).unwrap();
        assert_eq!(VcxStateType::VcxStateAccepted as u32, get_state(alice_to_faber));

        debug!("Institution is going to complete the connection protocol.");
        ::utils::devsetup::set_institution();
        thread::sleep(Duration::from_millis(500));
        update_state(faber_to_alice).unwrap();
        assert_eq!(VcxStateType::VcxStateAccepted as u32, get_state(faber_to_alice));

        (alice_to_faber, faber_to_alice)
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_connection() {
        let _setup = SetupAriesMocks::init();

        let handle = create_connection("test_create_connection").unwrap();
        assert_eq!(get_state(handle), VcxStateType::VcxStateInitialized as u32);


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
        let _setup = SetupAriesMocks::init();

        let handle = create_connection("test_create_drop_create").unwrap();

        assert_eq!(get_state(handle), VcxStateType::VcxStateInitialized as u32);
        let did1 = get_pw_did(handle).unwrap();

        release(handle).unwrap();

        let handle2 = create_connection("test_create_drop_create").unwrap();

        assert_eq!(get_state(handle2), VcxStateType::VcxStateInitialized as u32);
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
        let _setup = SetupAriesMocks::init();

        let handle = create_connection("test_get_qr_code_data").unwrap();

        connect(handle).unwrap();

        let details = get_invite_details(handle, true).unwrap();
        assert!(details.contains("\"serviceEndpoint\":"));

        assert_eq!(get_invite_details(0, true).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_deserialize_connection_inviter_completed() {
        let _setup = SetupAriesMocks::init();

        let handle = from_string(CONNECTION_SM_INVITER_COMPLETED).unwrap();
        let _second_string = to_string(handle).unwrap();

        assert_eq!(get_pw_did(handle).unwrap(), "2ZHFFhzA2XtTD6hJqzL7ux");
        assert_eq!(get_pw_verkey(handle).unwrap(), "rCw3x5h1jS6gPo7rRrt3EYbXXe5nNjnGbdf1jAwUxuj");
        assert_eq!(get_agent_did(handle).unwrap(), "EZrZyu4bfydm4ByNm56kPP");
        assert_eq!(get_agent_verkey(handle).unwrap(), "8Ps2WosJ9AV1eXPoJKsEJdM3NchPhSyS8qFt6LQUTKv2");
        assert_eq!(get_state(handle), VcxStateType::VcxStateAccepted as u32);
        assert!(release(handle).is_ok());
    }

    fn test_deserialize_and_serialize(sm_serialized: &str) {
        let original_object: Value = serde_json::from_str(sm_serialized).unwrap();
        let handle_conn = from_string(sm_serialized).unwrap();
        let reserialized = to_string(handle_conn).unwrap();
        let reserialized_object: Value = serde_json::from_str(&reserialized).unwrap();

        assert_eq!(original_object, reserialized_object);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_deserialize_and_serialize_should_produce_the_same_object() {
        let _setup = SetupAriesMocks::init();

        test_deserialize_and_serialize(CONNECTION_SM_INVITEE_INVITED);
        test_deserialize_and_serialize(CONNECTION_SM_INVITEE_REQUESTED);
        test_deserialize_and_serialize(CONNECTION_SM_INVITEE_COMPLETED);
        test_deserialize_and_serialize(CONNECTION_SM_INVITER_COMPLETED);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_serialize_deserialize() {
        let _setup = SetupAriesMocks::init();

        let handle = create_connection("test_serialize_deserialize").unwrap();

        let first_string = to_string(handle).unwrap();
        info!("{:?}", first_string);
        assert!(release(handle).is_ok());
        let handle = from_string(&first_string).unwrap();
        let second_string = to_string(handle).unwrap();

        assert_eq!(first_string, second_string);

        assert!(release(handle).is_ok());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_deserialize_existing() {
        let _setup = SetupAriesMocks::init();

        let handle = create_connection("test_serialize_deserialize").unwrap();

        let _pw_did = get_pw_did(handle).unwrap();
        let first_string = to_string(handle).unwrap();

        let handle = from_string(&first_string).unwrap();

        let _pw_did = get_pw_did(handle).unwrap();
        let second_string = to_string(handle).unwrap();

        assert_eq!(first_string, second_string);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_retry_connection() {
        let _setup = SetupAriesMocks::init();

        let handle = create_connection("test_serialize_deserialize").unwrap();

        assert_eq!(get_state(handle), VcxStateType::VcxStateInitialized as u32);

        connect(handle).unwrap();
        connect(handle).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_release_all() {
        let _setup = SetupAriesMocks::init();

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
        let _setup = SetupAriesMocks::init();

        let handle = create_connection_with_invite("alice", ARIES_CONNECTION_INVITATION).unwrap();
        connect(handle).unwrap();

        let handle_2 = create_connection_with_invite("alice", ARIES_CONNECTION_INVITATION).unwrap();
        connect(handle_2).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_process_acceptance_message() {
        let _setup = SetupAriesMocks::init();

        let handle = create_connection("test_process_acceptance_message").unwrap();
        let message = serde_json::from_str(ARIES_CONNECTION_REQUEST).unwrap();
        assert_eq!(error::SUCCESS.code_num, update_state_with_message(handle, message).unwrap());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_connection_handle_is_found() {
        let _setup = SetupAriesMocks::init();
        let handle = create_connection_with_invite("alice", ARIES_CONNECTION_INVITATION).unwrap();

        CONNECTION_MAP.get_mut(handle, |_connection| {
            Ok(())
        }).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_send_generic_message_fails_with_invalid_connection() {
        let _setup = SetupAriesMocks::init();

        let handle = ::connection::tests::build_test_connection_inviter_invited();

        let err = send_generic_message(handle, "this is the message", &json!({"msg_type":"type", "msg_title": "title", "ref_msg_id":null}).to_string()).unwrap_err();
        assert_eq!(err.kind(), VcxErrorKind::NotReady);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[test]
    fn test_send_and_download_messages() {
        let _setup = SetupLibraryAgencyV2::init();
        let (alice_to_faber, faber_to_alice) = ::connection::tests::create_connected_connections(None);

        send_generic_message(faber_to_alice, "Hello Alice", &json!({"msg_type": "toalice", "msg_title": "msg1"}).to_string()).unwrap();
        send_generic_message(faber_to_alice, "How are you Alice?", &json!({"msg_type": "toalice", "msg_title": "msg2"}).to_string()).unwrap();

        // AS CONSUMER GET MESSAGES
        ::utils::devsetup::set_consumer(None);
        send_generic_message(alice_to_faber, "Hello Faber", &json!({"msg_type": "tofaber", "msg_title": "msg1"}).to_string()).unwrap();

        // make sure messages has bee delivered
        thread::sleep(Duration::from_millis(1000));

        let all_messages = download_messages(None, None, None).unwrap();
        assert_eq!(all_messages.len(), 1);
        assert_eq!(all_messages[0].msgs.len(), 3);
        assert!(all_messages[0].msgs[0].decrypted_payload.is_some());
        assert!(all_messages[0].msgs[1].decrypted_payload.is_some());

        let received = download_messages(None, Some(vec![MessageStatusCode::Received.to_string()]), None).unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].msgs.len(), 2);
        assert!(received[0].msgs[0].decrypted_payload.is_some());
        assert_eq!(received[0].msgs[0].status_code, MessageStatusCode::Received);
        assert!(received[0].msgs[1].decrypted_payload.is_some());

        // there should be messages in "Reviewed" status connections/1.0/response from Aries-Faber connection protocol
        let reviewed = download_messages(None, Some(vec![MessageStatusCode::Reviewed.to_string()]), None).unwrap();
        assert_eq!(reviewed.len(), 1);
        assert_eq!(reviewed[0].msgs.len(), 1);
        assert!(reviewed[0].msgs[0].decrypted_payload.is_some());
        assert_eq!(reviewed[0].msgs[0].status_code, MessageStatusCode::Reviewed);

        let rejected = download_messages(None, Some(vec![MessageStatusCode::Rejected.to_string()]), None).unwrap();
        assert_eq!(rejected.len(), 1);
        assert_eq!(rejected[0].msgs.len(), 0);

        let specific = download_messages(None, None, Some(vec![received[0].msgs[0].uid.clone()])).unwrap();
        assert_eq!(specific.len(), 1);
        assert_eq!(specific[0].msgs.len(), 1);

        let unknown_did = "CmrXdgpTXsZqLQtGpX5Yee".to_string();
        let empty = download_messages(Some(vec![unknown_did]), None, None).unwrap();
        assert_eq!(empty.len(), 0);
    }
}
