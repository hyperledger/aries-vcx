use std::collections::HashMap;

use serde_json;

use agency_client;
use agency_client::{MessageStatusCode};
use agency_client::get_message::{Message, MessageByConnection};

use crate::aries::handlers::connection::pairwise_info::PairwiseInfo;
use crate::aries::handlers::connection::connection::{Connection, SmConnectionState};
use crate::aries::messages::a2a::A2AMessage;
use crate::aries::messages::connection::invite::Invitation as InvitationV3;
use crate::error::prelude::*;
use crate::utils::error;
use crate::utils::object_cache::ObjectCache;
use crate::aries::handlers::connection::cloud_agent::CloudAgentInfo;
use crate::aries::handlers::connection::legacy_agent_info::LegacyAgentInfo;
use crate::utils::serialization::SerializableObjectWithState;

lazy_static! {
    static ref CONNECTION_MAP: ObjectCache<Connection> = ObjectCache::<Connection>::new("connections-cache");
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
        connection.remote_did()
    })
}

pub fn get_their_pw_verkey(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        connection.remote_vk()
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

pub fn send_generic_message(connection_handle: u32, msg: &str) -> VcxResult<String> {
    CONNECTION_MAP.get(connection_handle, |connection| {
        connection.send_generic_message(msg)
    })
}

pub fn update_state_with_message(handle: u32, message: A2AMessage) -> VcxResult<u32> {
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

/**
Tries to update state of connection state machine in 3 steps:
  1. find relevant message in agency,
  2. use it to update connection state and possibly send response over network,
  3. update state of used message in agency to "Reviewed".
 */
pub fn update_state(handle: u32) -> VcxResult<u32> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection.update_state() {
            Ok(_) => Ok(error::SUCCESS.code_num),
            Err(err) => Err(err)
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
            .map(|invitation| json!(invitation.to_a2a_message()).to_string());
        Ok(invitation)
    })
}

pub fn to_string(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        let (state, pairwise_info, cloud_agent_info, source_id) = connection.to_owned().into();
        let data = LegacyAgentInfo {
            pw_did: pairwise_info.pw_did,
            pw_vk: pairwise_info.pw_vk,
            agent_did: cloud_agent_info.agent_did,
            agent_vk: cloud_agent_info.agent_vk
        };
        let object = SerializableObjectWithState::V1 { data, state, source_id };

        serde_json::to_string(&object)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidState, format!("Cannot serialize Connection: {:?}", err)))
    })
}

pub fn from_string(connection_data: &str) -> VcxResult<u32> {
    let object: SerializableObjectWithState<LegacyAgentInfo, SmConnectionState> = serde_json::from_str(connection_data)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize Connection: {:?}", err)))?;

    let handle = match object {
        SerializableObjectWithState::V1 { data, state, source_id } => {
            let pairwise_info = PairwiseInfo { pw_did: data.pw_did, pw_vk: data.pw_vk };
            let cloud_agent_info = CloudAgentInfo { agent_did: data.agent_did, agent_vk: data.agent_vk };
            let cconnection: Connection = (state, pairwise_info, cloud_agent_info, source_id).into();
            CONNECTION_MAP.add(cconnection)?
        }
    };
    Ok(handle)
}

impl Into<(SmConnectionState, PairwiseInfo, CloudAgentInfo, String)> for Connection {
    fn into(self) -> (SmConnectionState, PairwiseInfo, CloudAgentInfo, String) {
        (self.state_object(), self.pairwise_info().to_owned(), self.cloud_agent_info().to_owned(), self.source_id())
    }
}

impl From<(SmConnectionState, PairwiseInfo, CloudAgentInfo, String)> for Connection {
    fn from((state, pairwise_info, cloud_agent_info, source_id): (SmConnectionState, PairwiseInfo, CloudAgentInfo, String)) -> Connection {
        Connection::from_parts(source_id, pairwise_info, cloud_agent_info, state, true)
    }
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
            .map(|invitation| json!(invitation.to_a2a_message()).to_string())
            .ok_or(VcxError::from(VcxErrorKind::ActionNotSupported))
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
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

pub fn send_message(handle: u32, message: A2AMessage) -> VcxResult<()> {
    trace!("connection::send_message >>>");
    let send_message = send_message_closure(handle)?;
    send_message(&message)
}

pub fn send_message_closure(handle: u32) -> VcxResult<impl Fn(&A2AMessage) -> VcxResult<()>> {
    CONNECTION_MAP.get(handle, |connection| {
        return connection.send_message_closure()
    })
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

pub fn download_messages(conn_handles: Vec<u32>, status_codes: Option<Vec<MessageStatusCode>>, uids: Option<Vec<String>>) -> VcxResult<Vec<MessageByConnection>> {
    trace!("download_messages >>> cann_handles: {:?}, status_codes: {:?}, uids: {:?}", conn_handles, status_codes, uids);
    let mut res = Vec::new();
    for conn_handle in conn_handles {
        let msg_by_conn = CONNECTION_MAP.get(
            conn_handle, |connection| {
                let msgs = connection.download_messages(status_codes.clone(), uids.clone())?;
                Ok(MessageByConnection { pairwise_did: connection.agent_info().clone().pw_did, msgs })
            },
        )?;
        res.push(msg_by_conn);
    };
    trace!("download_messages <<< res: {:?}", res);
    Ok(res)
}

#[cfg(test)]
pub mod tests {
    use std::thread;
    use std::time::Duration;

    use serde_json::Value;

    use agency_client::get_message::download_messages_noauth;
    use agency_client::MessageStatusCode;
    use agency_client::mocking::AgencyMockDecrypted;
    use agency_client::update_message::{UIDsByConn, update_agency_messages};

    use crate::{connection, utils, settings, aries};
    use crate::api::VcxStateType;
    use crate::utils::constants;
    use crate::utils::devsetup::*;
    use crate::utils::mockdata::mockdata_connection::{ARIES_CONNECTION_ACK, ARIES_CONNECTION_INVITATION, ARIES_CONNECTION_REQUEST, CONNECTION_SM_INVITEE_COMPLETED, CONNECTION_SM_INVITEE_INVITED, CONNECTION_SM_INVITEE_REQUESTED, CONNECTION_SM_INVITER_COMPLETED};

    use super::*;
    use crate::utils::devsetup_agent::test::{Faber, Alice, TestAgent};
    use crate::aries::messages::ack::tests::_ack;
    use crate::aries::messages::connection::invite::tests::_invitation_json;


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
    fn test_create_connection_with_invite_works() {
        let _setup = SetupMocks::init();
        let connection_handle = connection::create_connection_with_invite(_source_id(), &_invitation_json()).unwrap();
        assert!(connection::is_valid_handle(connection_handle));
        assert_eq!(1, connection::get_state(connection_handle));
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
        let handle = from_string(CONNECTION_SM_INVITEE_COMPLETED).unwrap();
        handle
    }

    pub fn build_test_connection_inviter_requested() -> u32 {
        let handle = build_test_connection_inviter_invited();
        let msg: A2AMessage = serde_json::from_str(ARIES_CONNECTION_REQUEST).unwrap();
        update_state_with_message(handle, msg).unwrap();
        handle
    }

    pub fn create_and_store_connected_connections(consumer: &mut Alice, institution: &mut Faber) -> (u32, u32) {
        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(consumer, institution);
        let consumer_to_institution = store_connection(consumer_to_institution).unwrap();
        let institution_to_consumer = store_connection(institution_to_consumer).unwrap();
        (consumer_to_institution, institution_to_consumer)
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
    fn test_deserialize_connection_inviter_completed() {
        let _setup = SetupMocks::init();

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
        let _setup = SetupMocks::init();

        test_deserialize_and_serialize(CONNECTION_SM_INVITEE_INVITED);
        test_deserialize_and_serialize(CONNECTION_SM_INVITEE_REQUESTED);

        test_deserialize_and_serialize(CONNECTION_SM_INVITEE_COMPLETED);
        test_deserialize_and_serialize(CONNECTION_SM_INVITER_COMPLETED);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_serialize_deserialize() {
        let _setup = SetupMocks::init();

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
        let _setup = SetupMocks::init();

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
        let message = serde_json::from_str(ARIES_CONNECTION_REQUEST).unwrap();
        assert_eq!(error::SUCCESS.code_num, update_state_with_message(handle, message).unwrap());
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

    #[cfg(feature = "agency_pool_tests")]
    #[test]
    fn test_update_agency_messages() {
        let _setup = SetupLibraryAgencyV2::init();
        let mut institution = Faber::setup();
        let mut consumer1 = Alice::setup();
        let (alice_to_faber, faber_to_alice) = create_and_store_connected_connections(&mut consumer1, &mut institution);

        send_generic_message(faber_to_alice, "Hello 1").unwrap();
        send_generic_message(faber_to_alice, "Hello 2").unwrap();
        send_generic_message(faber_to_alice, "Hello 3").unwrap();

        thread::sleep(Duration::from_millis(1000));
        consumer1.activate().unwrap();

        let received = download_messages_noauth(None, Some(vec![MessageStatusCode::Received.to_string()]), None).unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].msgs.len(), 3);
        let pairwise_did = received[0].pairwise_did.clone();
        let uid = received[0].msgs[0].uid.clone();

        let reviewed = download_messages_noauth(Some(vec![pairwise_did.clone()]), Some(vec![MessageStatusCode::Reviewed.to_string()]), None).unwrap();
        let reviewed_count_before = reviewed[0].msgs.len();

        // update status
        let message = serde_json::to_string(&vec![UIDsByConn { pairwise_did: pairwise_did.clone(), uids: vec![uid.clone()] }]).unwrap();
        update_agency_messages("MS-106", &message).unwrap();

        let received = download_messages_noauth(None, Some(vec![MessageStatusCode::Received.to_string()]), None).unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].msgs.len(), 2);

        let reviewed = download_messages_noauth(Some(vec![pairwise_did.clone()]), Some(vec![MessageStatusCode::Reviewed.to_string()]), None).unwrap();
        let reviewed_count_after = reviewed[0].msgs.len();
        assert_eq!(reviewed_count_after, reviewed_count_before + 1);

        let specific_review = download_messages_noauth(Some(vec![pairwise_did.clone()]), Some(vec![MessageStatusCode::Reviewed.to_string()]), Some(vec![uid.clone()])).unwrap();
        assert_eq!(specific_review[0].msgs[0].uid, uid);
    }
}
