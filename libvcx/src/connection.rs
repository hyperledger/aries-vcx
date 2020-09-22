use std::collections::HashMap;

use serde_json;

use api::VcxStateType;
use error::prelude::*;
use messages;
use messages::SerializableObjectWithState;
use messages::get_message::Message;
use messages::invite::{InviteDetail, RedirectDetail};
use object_cache::ObjectCache;
use settings;
use settings::ProtocolTypes;
use utils::error;
use utils::json::KeyMatch;
use v3::handlers::connection::connection::Connection as ConnectionV3;
use v3::handlers::connection::states::ActorDidExchangeState;
use v3::handlers::connection::agent::AgentInfo;
use v3::messages::a2a::A2AMessage;
use v3::messages::connection::did_doc::DidDoc;
use v3::messages::connection::invite::Invitation as InvitationV3;

lazy_static! {
    static ref CONNECTION_MAP: ObjectCache<Connections> = ObjectCache::<Connections>::new("connections-cache");
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "version")]
enum Connections {
    #[serde(rename = "1.0")]
    V1(Connection),
    #[serde(rename = "2.0")]
    V3(ConnectionV3),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectionOptions {
    #[serde(default)]
    pub connection_type: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    pub use_public_did: Option<bool>,
}

impl Default for ConnectionOptions {
    fn default() -> Self {
        ConnectionOptions {
            connection_type: None,
            phone: None,
            use_public_did: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Connection {
}

pub fn create_agent_keys(source_id: &str, pw_did: &str, pw_verkey: &str) -> VcxResult<(String, String)> {
    /*
        Create User Pairwise Agent in old way.
        Send Messages corresponding to V2 Protocol version to avoid code changes on Agency side.
    */
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
        match connection {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::ActionNotSupported)),
            Connections::V3(ref connection) => connection.remote_vk()
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn is_valid_handle(handle: u32) -> bool {
    CONNECTION_MAP.has_handle(handle)
}

pub fn get_agent_did(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::ActionNotSupported)),
            Connections::V3(ref connection) => Ok(connection.agent_info().agent_did.to_string())
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_pw_did(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::ActionNotSupported)),
            Connections::V3(ref connection) => Ok(connection.agent_info().pw_did.to_string())
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_their_pw_did(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::ActionNotSupported)),
            Connections::V3(ref connection) => connection.remote_did()
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_agent_verkey(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::ActionNotSupported)),
            Connections::V3(ref connection) => Ok(connection.agent_info().agent_vk.clone())
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_version(handle: u32) -> VcxResult<Option<ProtocolTypes>> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::ActionNotSupported)),
            Connections::V3(_) => Ok(Some(settings::get_protocol_type()))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_pw_verkey(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::ActionNotSupported)),
            Connections::V3(ref connection) => Ok(connection.agent_info().pw_vk.clone())
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_state(handle: u32) -> u32 {
    trace!("get_state >>> handle = {:?}", handle);
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(_) => Ok(0),
            Connections::V3(ref connection) => Ok(connection.state())
        }
    }).unwrap_or(0)
}

pub fn get_source_id(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::ActionNotSupported)),
            Connections::V3(ref connection) => Ok(connection.get_source_id())
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

fn store_connection(connection: Connections) -> VcxResult<u32> {
    CONNECTION_MAP.add(connection)
        .or(Err(VcxError::from(VcxErrorKind::CreateConnection)))
}

pub fn create_connection(source_id: &str) -> VcxResult<u32> {
    trace!("create_connection >>> source_id: {}", source_id);
    let connection = Connections::V3(ConnectionV3::create(source_id));
    return store_connection(connection)
}

pub fn create_connection_with_invite(source_id: &str, details: &str) -> VcxResult<u32> {
    debug!("create connection {} with invite {}", source_id, details);
    if let Some(invitation) = serde_json::from_str::<InvitationV3>(details).ok() {
        let connection = Connections::V3(ConnectionV3::create_with_invite(source_id, invitation)?);
        store_connection(connection)
    } else {
        Err(VcxError::from_msg(VcxErrorKind::InvalidJson, "Used invite has invalid structure")) // TODO: Specific error type
    }
}

pub fn send_generic_message(connection_handle: u32, msg: &str, msg_options: &str) -> VcxResult<String> {
    CONNECTION_MAP.get(connection_handle, |connection| {
        match connection {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::ActionNotSupported)),
            Connections::V3(ref connection) => connection.send_generic_message(msg, msg_options)
        }
    })
}

pub fn update_state_with_message(handle: u32, message: A2AMessage) -> VcxResult<u32> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::ActionNotSupported)),
            Connections::V3(ref mut connection) => {
                connection.update_state(Some(&message))?;
                Ok(error::SUCCESS.code_num)
            }
        }
    })
        .or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn update_state(handle: u32, message: Option<A2AMessage>) -> VcxResult<u32> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::ActionNotSupported)),
            Connections::V3(ref mut connection) => {
                connection.update_state(message.as_ref())?;
                Ok(error::SUCCESS.code_num)
            }
        }
    })
        .or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn delete_connection(handle: u32) -> VcxResult<u32> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::ActionNotSupported)),
            Connections::V3(ref mut connection) => {
                connection.delete()?;
                Ok(error::SUCCESS.code_num)
            }
        }
    })
        .map(|_| error::SUCCESS.code_num)
        .or(Err(VcxError::from(VcxErrorKind::DeleteConnection)))
        .and(release(handle))
        .and_then(|_| Ok(error::SUCCESS.code_num))
}

pub fn connect(handle: u32, _options: Option<String>) -> VcxResult<u32> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::ActionNotSupported)),
            Connections::V3(ref mut connection) => {
                connection.connect()?;
                Ok(error::SUCCESS.code_num)
            }
        }
    })
}

pub fn to_string(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::ActionNotSupported)),
            Connections::V3(ref connection) => {
                let (state, data, source_id) = connection.to_owned().into();
                let object = SerializableObjectWithState::V3 { data, state, source_id };

                ::serde_json::to_string(&object)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidState, format!("Cannot serialize Connection: {:?}", err)))
            }
        }
    })
}

pub fn from_string(connection_data: &str) -> VcxResult<u32> {
    let object: SerializableObjectWithState<AgentInfo, ::v3::handlers::connection::states::ActorDidExchangeState> = ::serde_json::from_str(connection_data)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize Connection: {:?}", err)))?;

    let handle = match object {
        SerializableObjectWithState::V3 { data, state, source_id } => {
            CONNECTION_MAP.add(Connections::V3((state, data, source_id).into()))?
        },
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
        match connection {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::ActionNotSupported)),
            Connections::V3(ref connection) => connection.get_invite_details()
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

//**********
// Code to convert InviteDetails to Abbreviated String
//**********

impl KeyMatch for (String, Option<String>) {
    fn matches(&self, key: &String, context: &Vec<String>) -> bool {
        if key.eq(&self.0) {
            match context.last() {
                Some(parent) => {
                    if let Some(ref expected_parent) = self.1 {
                        return parent.eq(expected_parent);
                    }
                }
                None => {
                    return self.1.is_none();
                }
            }
        }
        false
    }
}


lazy_static! {
    static ref ABBREVIATIONS: Vec<(String, String)> = {
        vec![
        ("statusCode".to_string(),          "sc".to_string()),
        ("connReqId".to_string(),           "id".to_string()),
        ("senderDetail".to_string(),        "s".to_string()),
        ("name".to_string(),                "n".to_string()),
        ("agentKeyDlgProof".to_string(),    "dp".to_string()),
        ("agentDID".to_string(),            "d".to_string()),
        ("agentDelegatedKey".to_string(),   "k".to_string()),
        ("signature".to_string(),           "s".to_string()),
        ("DID".to_string(), "d".to_string()),
        ("logoUrl".to_string(), "l".to_string()),
        ("verKey".to_string(), "v".to_string()),
        ("senderAgencyDetail".to_string(), "sa".to_string()),
        ("endpoint".to_string(), "e".to_string()),
        ("targetName".to_string(), "t".to_string()),
        ("statusMsg".to_string(), "sm".to_string()),
        ]
    };
}

impl Into<(ActorDidExchangeState, AgentInfo, String)> for ConnectionV3 {
    fn into(self) -> (ActorDidExchangeState, AgentInfo, String) {
        (self.state_object().to_owned(), self.agent_info().to_owned(), self.source_id())
    }
}

impl From<(ActorDidExchangeState, AgentInfo, String)> for ConnectionV3 {
    fn from((state, agent_info, source_id): (ActorDidExchangeState, AgentInfo, String)) -> ConnectionV3 {
        ConnectionV3::from_parts(source_id, agent_info, state)
    }
}

pub fn get_messages(handle: u32) -> VcxResult<HashMap<String, A2AMessage>> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V3(ref mut connection) => connection.get_messages(),
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    })
}

pub fn update_message_status(handle: u32, uid: String) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V3(ref mut connection) => connection.update_message_status(uid.clone()),
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    })
}

pub fn get_message_by_id(handle: u32, msg_id: String) -> VcxResult<A2AMessage> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V3(ref mut connection) => connection.get_message_by_id(&msg_id),
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    })
}

pub fn decode_message(handle: u32, message: Message) -> VcxResult<A2AMessage> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V3(ref mut connection) => connection.decode_message(&message),
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    })
}

pub fn send_message(handle: u32, message: A2AMessage) -> VcxResult<()> {
    trace!("connection::send_message >>>");
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)),
            Connections::V3(ref mut connection) => connection.send_message(&message)
        }
    })
}

pub fn send_message_to_self_endpoint(message: A2AMessage, did_doc: &DidDoc) -> VcxResult<()> {
    ConnectionV3::send_message_to_self_endpoint(&message, did_doc)
}

pub fn is_v3_connection(connection_handle: u32) -> VcxResult<bool> {
    CONNECTION_MAP.get(connection_handle, |connection| {
        match connection {
            Connections::V1(_) => Ok(false),
            Connections::V3(_) => Ok(true)
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn send_ping(connection_handle: u32, comment: Option<String>) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(connection_handle, |connection| {
        match connection {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::ActionNotSupported)),
            Connections::V3(ref mut connection) => connection.send_ping(comment.clone())
        }
    })
}

pub fn send_discovery_features(connection_handle: u32, query: Option<String>, comment: Option<String>) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(connection_handle, |connection| {
        match connection {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::ActionNotSupported)),
            Connections::V3(ref mut connection) => connection.send_discovery_features(query.clone(), comment.clone())
        }
    })
}

pub fn get_connection_info(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::ActionNotSupported)),
            Connections::V3(ref connection) => connection.get_connection_info()
        }
    })
}

#[cfg(test)]
pub mod tests {
    use std::thread;
    use std::time::Duration;

    use messages::get_message::*;
    use utils::constants::*;
    use utils::constants;
    use utils::devsetup::*;
    use utils::httpclient::AgencyMockDecrypted;
    use utils::mockdata::mockdata_connection::{ARIES_CONNECTION_ACK, ARIES_CONNECTION_INVITATION, ARIES_CONNECTION_REQUEST};

    use super::*;

    pub fn build_test_connection() -> u32 {
        let handle = create_connection("alice").unwrap();
        connect(handle, Some("{}".to_string())).unwrap();
        handle
    }

    pub fn create_connected_connections() -> (u32, u32) {
        debug!("Institution is going to create connection.");
        ::utils::devsetup::set_institution();
        let faber_to_alice = create_connection("alice").unwrap();
        let _my_public_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        connect(faber_to_alice, None).unwrap();
        update_state(faber_to_alice, None).unwrap();
        let details = get_invite_details(faber_to_alice, false).unwrap();

        ::utils::devsetup::set_consumer();
        debug!("Consumer is going to accept connection invitation.");
        let alice_to_faber = create_connection_with_invite("faber", &details).unwrap();
        connect(alice_to_faber, None).unwrap();
        update_state(alice_to_faber, None).unwrap();
        // assert_eq!(VcxStateType::VcxStateRequestReceived as u32, get_state(faber));

        debug!("Institution is going to process connection request.");
        ::utils::devsetup::set_institution();
        thread::sleep(Duration::from_millis(500));
        update_state(faber_to_alice, None).unwrap();

        debug!("Consumer is going to complete the connection protocol.");
        ::utils::devsetup::set_consumer();
        update_state(alice_to_faber, None).unwrap();
        assert_eq!(VcxStateType::VcxStateAccepted as u32, get_state(alice_to_faber));

        debug!("Institution is going to complete the connection protocol.");
        ::utils::devsetup::set_institution();
        thread::sleep(Duration::from_millis(500));
        update_state(faber_to_alice, None).unwrap();
        assert_eq!(VcxStateType::VcxStateAccepted as u32, get_state(faber_to_alice));

        (alice_to_faber, faber_to_alice)
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_connection() {
        let _setup = SetupAriesMocks::init();

        let handle = create_connection("test_create_connection").unwrap();
        assert_eq!(get_state(handle), VcxStateType::VcxStateInitialized as u32);


        connect(handle, Some("{}".to_string())).unwrap();
        assert_eq!(get_pw_did(handle).unwrap(), constants::DID);
        assert_eq!(get_pw_verkey(handle).unwrap(), constants::VERKEY);

        AgencyMockDecrypted::set_next_decrypted_response(constants::GET_MESSAGES_DECRYPTED_RESPONSE);
        AgencyMockDecrypted::set_next_decrypted_message(ARIES_CONNECTION_REQUEST);
        update_state(handle, None).unwrap();
        assert_eq!(get_state(handle), VcxStateType::VcxStateRequestReceived as u32);

        AgencyMockDecrypted::set_next_decrypted_response(constants::GET_MESSAGES_DECRYPTED_RESPONSE);
        AgencyMockDecrypted::set_next_decrypted_message(ARIES_CONNECTION_ACK);
        update_state(handle, None).unwrap();
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

        connect(handle, None).unwrap();

        let details = get_invite_details(handle, true).unwrap();
        assert!(details.contains("\"serviceEndpoint\":"));

        assert_eq!(get_invite_details(0, true).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
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

        connect(handle, None).unwrap();
        connect(handle, None).unwrap();
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
        connect(handle, None).unwrap();

        let handle_2 = create_connection_with_invite("alice", ARIES_CONNECTION_INVITATION).unwrap();
        connect(handle_2, None).unwrap();
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
    fn test_create_with_legacy_invite_details() {
        let _setup = SetupAriesMocks::init();

        let err = create_connection_with_invite("alice", constants::INVITE_DETAIL_V1_STRING).unwrap_err();
        assert_eq!(err.kind(), VcxErrorKind::InvalidJson);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_different_protocol_version() {
        let _setup = SetupAriesMocks::init();
        let err = create_connection_with_invite("alice", INVITE_DETAIL_V1_STRING).unwrap_err();
        assert_eq!(err.kind(), VcxErrorKind::InvalidJson);

        let _setup = SetupAriesMocks::init();
        let handle = create_connection_with_invite("alice", ARIES_CONNECTION_INVITATION).unwrap();

        CONNECTION_MAP.get_mut(handle, |connection| {
            match connection {
                Connections::V1(_) => Err(VcxError::from_msg(VcxErrorKind::InvalidState, "It is suppose to be V3")),
                Connections::V3(_) => Ok(()),
            }
        }).unwrap();

        let _serialized = to_string(handle).unwrap();
    }
}
