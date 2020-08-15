use std::collections::HashMap;

use rmp_serde;
use serde_json;
use serde_json::Value;

use api::VcxStateType;
use error::prelude::*;
use error::VcxErrorKind::InvalidInviteDetail;
use messages;
use messages::{GeneralMessage, MessageStatusCode, RemoteMessageType, SerializableObjectWithState};
use messages::get_message::{Message, MessagePayload};
use messages::invite::{Payload as ConnectionPayload};
use messages::payload::{PayloadKinds, Payloads};
use messages::send_message::SendMessageOptions;
use messages::thread::Thread;
use object_cache::ObjectCache;
use settings;
use settings::ProtocolTypes;
use utils::error;
use utils::json::KeyMatch;
use utils::json::mapped_key_rewrite;
use utils::libindy::crypto;
use utils::libindy::signus::create_and_store_my_did;
use v3::handlers::connection::agent::AgentInfo;
use v3::handlers::connection::connection::Connection as ConnectionV3;
use v3::handlers::connection::states::ActorDidExchangeState;
use v3::messages::a2a::A2AMessage;
use v3::messages::connection::did_doc::DidDoc;
use v3::messages::connection::invite::Invitation as InvitationV3;
use messages::invite::{InviteDetail, AcceptanceDetails, SenderDetail};

lazy_static! {
    static ref CONNECTION_MAP: ObjectCache<Connections> = Default::default();
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "version")]
enum Connections {
    // #[serde(rename = "1.0")]
    // V1(Connection),
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

impl ConnectionOptions {
    pub fn from_opt_str(options: Option<String>) -> VcxResult<ConnectionOptions> {
        Ok(
            match options.as_ref().map(|opt| opt.trim()) {
                None => ConnectionOptions::default(),
                Some(opt) if opt.is_empty() => ConnectionOptions::default(),
                Some(opt) => {
                    serde_json::from_str(&opt)
                        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidOption, format!("Cannot deserialize ConnectionOptions: {}", err)))?
                }
            }
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Connection {
    source_id: String,
    pw_did: String,
    pw_verkey: String,
    state: VcxStateType,
    uuid: String,
    endpoint: String,
    // For QR code invitation
    invite_detail: Option<InviteDetail>,
    invite_url: Option<String>,
    agent_did: String,
    agent_vk: String,
    their_pw_did: String,
    their_pw_verkey: String,
    // used by proofs/credentials when sending to edge device
    public_did: Option<String>,
    their_public_did: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<settings::ProtocolTypes>,
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

pub fn is_valid_handle(handle: u32) -> bool {
    CONNECTION_MAP.has_handle(handle)
}

pub fn set_agent_did(handle: u32, did: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_agent_did(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V3(ref connection) => Ok(connection.agent_info().agent_did.to_string())
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_pw_did(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V3(ref connection) => Ok(connection.agent_info().pw_did.to_string())
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_ver_str(handle: u32) -> VcxResult<Option<String>> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn set_pw_did(handle: u32, did: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_their_pw_did(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V3(ref connection) => connection.remote_did()
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn set_their_pw_did(handle: u32, did: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn set_their_public_did(handle: u32, did: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_their_public_did(handle: u32) -> VcxResult<Option<String>> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_their_pw_verkey(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V3(ref connection) => connection.remote_vk()
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn set_their_pw_verkey(handle: u32, did: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |cxn| {
        match cxn {
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_uuid(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn set_uuid(handle: u32, uuid: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |cxn| {
        match cxn {
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

// TODO: Add NO_ENDPOINT error to connection error
pub fn get_endpoint(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::NoEndpoint)))
}

pub fn set_endpoint(handle: u32, endpoint: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |cxn| {
        match cxn {
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_agent_verkey(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V3(ref connection) => Ok(connection.agent_info().agent_vk.clone())
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_version(handle: u32) -> VcxResult<Option<ProtocolTypes>> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V3(_) => Ok(Some(settings::get_protocol_type()))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn set_agent_verkey(handle: u32, verkey: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |cxn| {
        match cxn {
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_pw_verkey(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V3(ref connection) => Ok(connection.agent_info().pw_vk.clone())
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn set_pw_verkey(handle: u32, verkey: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |cxn| {
        match cxn {
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_state(handle: u32) -> u32 {
    CONNECTION_MAP.get(handle, |cxn| {
        debug!("get state for connection");
        match cxn {
            Connections::V3(ref connection) => Ok(connection.state())
        }
    }).unwrap_or(0)
}


pub fn set_state(handle: u32, state: VcxStateType) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |cxn| {
        match cxn {
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_source_id(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
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
    return store_connection(connection);
}

pub fn create_connection_with_invite(source_id: &str, details: &str) -> VcxResult<u32> {
    debug!("create connection {} with invite {}", source_id, details);

    // Invitation of new format -- redirect to v3 folder
    if let Ok(invitation) = serde_json::from_str::<InvitationV3>(details) {
        let connection = Connections::V3(ConnectionV3::create_with_invite(source_id, invitation)?);
        return store_connection(connection);
    }
    return Err(VcxError::from(VcxErrorKind::InvalidInviteDetail));
}

pub fn parse_acceptance_details(message: &Message) -> VcxResult<SenderDetail> {
    let my_vk = settings::get_config_value(settings::CONFIG_SDK_TO_REMOTE_VERKEY)?;

    let payload = message.payload
        .as_ref()
        .ok_or(VcxError::from_msg(VcxErrorKind::InvalidMessagePack, "Payload not found"))?;

    match payload {
        MessagePayload::V1(payload) => {
            // TODO: check returned verkey
            let (_, payload) = crypto::parse_msg(&my_vk, &messages::to_u8(&payload))
                .map_err(|err| err.map(VcxErrorKind::InvalidMessagePack, "Cannot decrypt connection payload"))?;

            let response: ConnectionPayload = rmp_serde::from_slice(&payload[..])
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidMessagePack, format!("Cannot parse connection payload: {}", err)))?;

            let payload = messages::to_u8(&response.msg);

            let response: AcceptanceDetails = rmp_serde::from_slice(&payload[..])
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidMessagePack, format!("Cannot deserialize AcceptanceDetails: {}", err)))?;

            Ok(response.sender_detail)
        }
        MessagePayload::V2(payload) => {
            let payload = Payloads::decrypt_payload_v2(&my_vk, &payload)?;
            let response: AcceptanceDetails = serde_json::from_str(&payload.msg)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize AcceptanceDetails: {}", err)))?;

            Ok(response.sender_detail)
        }
    }
}

pub fn send_generic_message(connection_handle: u32, msg: &str, msg_options: &str) -> VcxResult<String> {
    CONNECTION_MAP.get(connection_handle, |connection| {
        match connection {
            Connections::V3(ref connection) => connection.send_generic_message(msg, msg_options)
        }
    })
}

pub fn update_state_with_message(handle: u32, message: Message) -> VcxResult<u32> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V3(ref mut connection) => {
                connection.update_state(Some(&json!(message).to_string()))?;
                Ok(error::SUCCESS.code_num)
            }
        }
    })
        .or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}


pub fn update_state(handle: u32, message: Option<String>) -> VcxResult<u32> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V3(ref mut connection) => {
                connection.update_state(message.as_ref().map(String::as_str))?;
                Ok(error::SUCCESS.code_num)
            }
        }
    })
        .or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}


pub fn delete_connection(handle: u32) -> VcxResult<u32> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
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

pub fn connect(handle: u32, options: Option<String>) -> VcxResult<u32> {
    let options_obj: ConnectionOptions = ConnectionOptions::from_opt_str(options)?;

    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V3(ref mut connection) => {
                connection.connect()?;
                Ok(error::SUCCESS.code_num)
            }
        }
    })
}

impl Into<(Connection, ActorDidExchangeState)> for ConnectionV3 {
    fn into(self) -> (Connection, ActorDidExchangeState) {
        let data = Connection {
            source_id: self.source_id().clone(),
            pw_did: self.agent_info().pw_did.clone(),
            pw_verkey: self.agent_info().pw_vk.clone(),
            state: VcxStateType::from_u32(self.state()),
            uuid: String::new(),
            endpoint: String::new(),
            invite_detail: None,
            invite_url: None,
            agent_did: self.agent_info().agent_did.clone(),
            agent_vk: self.agent_info().agent_vk.clone(),
            their_pw_did: self.remote_did().unwrap_or_default(),
            their_pw_verkey: self.remote_vk().unwrap_or_default(),
            public_did: None,
            their_public_did: None,
            version: Some(ProtocolTypes::V2), // TODO check correctness
        };

        (data, self.state_object().to_owned())
    }
}

impl From<(Connection, ActorDidExchangeState)> for ConnectionV3 {
    fn from((connection, state): (Connection, ActorDidExchangeState)) -> ConnectionV3 {
        let agent_info = AgentInfo {
            pw_did: connection.pw_did,
            pw_vk: connection.pw_verkey,
            agent_did: connection.agent_did,
            agent_vk: connection.agent_vk
        };

        ConnectionV3::from_parts(connection.source_id, agent_info, state)
    }
}

pub fn to_string(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V3(ref connection) => {
                // let (data, state) = connection.to_owned().into();
                let (data, state) = connection.to_owned().into();
                let object = SerializableObjectWithState::V2 { data, state };

                ::serde_json::to_string(&object)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidState, format!("Cannot serialize Connection: {:?}", err)))
            }
        }
    })
}

pub fn from_string(connection_data: &str) -> VcxResult<u32> {
    let object: SerializableObjectWithState<Connection, ::v3::handlers::connection::states::ActorDidExchangeState> = ::serde_json::from_str(connection_data)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize Connection: {:?}", err)))?;

    let handle = match object {
        SerializableObjectWithState::V2 { data, state } => {
            CONNECTION_MAP.add(Connections::V3((data, state).into()))?
        }
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

pub fn get_invite_details(handle: u32, abbreviated: bool) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V3(ref connection) => {
                connection.get_invite_details()
            }
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn set_invite_details(handle: u32, invite_detail: &InviteDetail) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V3(_) => {
                Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
            }
        }
    })
        .or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
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


pub fn get_messages(handle: u32) -> VcxResult<HashMap<String, A2AMessage>> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V3(ref mut connection) => connection.get_messages(),
        }
    })
}

pub fn update_message_status(handle: u32, uid: String) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V3(ref mut connection) => connection.update_message_status(uid.clone()),
        }
    })
}

pub fn get_message_by_id(handle: u32, msg_id: String) -> VcxResult<A2AMessage> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V3(ref mut connection) => connection.get_message_by_id(&msg_id),
        }
    })
}

pub fn decode_message(handle: u32, message: Message) -> VcxResult<A2AMessage> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V3(ref mut connection) => connection.decode_message(&message),
        }
    })
}

pub fn send_message(handle: u32, message: A2AMessage) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
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
            Connections::V3(_) => Ok(true)
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn send_ping(connection_handle: u32, comment: Option<String>) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(connection_handle, |connection| {
        match connection {
            Connections::V3(ref mut connection) => connection.send_ping(comment.clone())
        }
    })
}

pub fn send_discovery_features(connection_handle: u32, query: Option<String>, comment: Option<String>) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(connection_handle, |connection| {
        match connection {
            Connections::V3(ref mut connection) => connection.send_discovery_features(query.clone(), comment.clone())
        }
    })
}

pub fn get_connection_info(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
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
    use utils::httpclient::AgencyMock;

    use super::*;

    pub fn build_test_connection() -> u32 {
        let handle = create_connection("alice").unwrap();
        connect(handle, Some("{}".to_string())).unwrap();
        handle
    }

    pub fn create_connected_connections() -> (u32, u32) {
        ::utils::devsetup::set_institution();

        let alice = create_connection("alice").unwrap();
        let my_public_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let options = json!({"use_public_did": true}).to_string();

        connect(alice, Some(options)).unwrap();
        let details = get_invite_details(alice, false).unwrap();

        //BE CONSUMER AND ACCEPT INVITE FROM INSTITUTION
        ::utils::devsetup::set_consumer();

        let faber = create_connection_with_invite("faber", &details).unwrap();

        assert_eq!(VcxStateType::VcxStateRequestReceived as u32, get_state(faber));

        connect(faber, Some("{}".to_string())).unwrap();
        let public_did = get_their_public_did(faber).unwrap().unwrap();
        assert_eq!(my_public_did, public_did);

        //BE INSTITUTION AND CHECK THAT INVITE WAS ACCEPTED
        ::utils::devsetup::set_institution();

        thread::sleep(Duration::from_millis(500));

        update_state(alice, None).unwrap();
        assert_eq!(VcxStateType::VcxStateAccepted as u32, get_state(alice));
        (faber, alice)
    }

    #[test]
    fn test_build_connection_failures_with_no_wallet() {
        let _setup = SetupDefaults::init();

        assert_eq!(create_connection("This Should Fail").unwrap_err().kind(), VcxErrorKind::InvalidWalletHandle);

        assert_eq!(create_connection_with_invite("This Should Fail", "BadDetailsFoobar").unwrap_err().kind(), VcxErrorKind::InvalidJson);
    }

    #[test]
    fn test_create_connection() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_create_connection").unwrap();

        assert_eq!(get_pw_did(handle).unwrap(), constants::DID);
        assert_eq!(get_pw_verkey(handle).unwrap(), constants::VERKEY);
        assert_eq!(get_state(handle), VcxStateType::VcxStateInitialized as u32);

        connect(handle, Some("{}".to_string())).unwrap();

        AgencyMock::set_next_response(GET_MESSAGES_INVITE_ACCEPTED_RESPONSE.to_vec());
        update_state(handle, None).unwrap();
        assert_eq!(get_state(handle), VcxStateType::VcxStateAccepted as u32);

        AgencyMock::set_next_response(DELETE_CONNECTION_RESPONSE.to_vec());
        assert_eq!(delete_connection(handle).unwrap(), 0);

        // This errors b/c we release handle in delete connection
        assert!(release(handle).is_err());
    }

    #[test]
    fn test_create_drop_create() {
        let _setup = SetupMocks::init();

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
    fn test_connection_release_fails() {
        let _setup = SetupEmpty::init();

        let rc = release(1);
        assert_eq!(rc.unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
    }

    #[test]
    fn test_get_state_fails() {
        let _setup = SetupEmpty::init();

        let state = get_state(1);
        assert_eq!(state, VcxStateType::VcxStateNone as u32);
    }

    #[test]
    fn test_get_string_fails() {
        let _setup = SetupEmpty::init();

        let rc = to_string(0);
        assert_eq!(rc.unwrap_err().kind(), VcxErrorKind::InvalidHandle);
    }

    #[test]
    fn test_get_qr_code_data() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_get_qr_code_data").unwrap();

        connect(handle, None).unwrap();

        let details = get_invite_details(handle, true).unwrap();
        assert!(details.contains("\"dp\":"));

        assert_eq!(get_invite_details(0, true).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
    }

    #[test]
    fn test_serialize_deserialize() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_serialize_deserialize").unwrap();

        let first_string = to_string(handle).unwrap();
        assert!(release(handle).is_ok());
        let handle = from_string(&first_string).unwrap();
        let second_string = to_string(handle).unwrap();

        assert_eq!(first_string, second_string);

        assert!(release(handle).is_ok());

        // Aries connection
        ::settings::set_config_value(::settings::COMMUNICATION_METHOD, "aries");

        let handle = create_connection("test_serialize_deserialize").unwrap();

        let first_string = to_string(handle).unwrap();
        assert!(release(handle).is_ok());
        let handle = from_string(&first_string).unwrap();
        let second_string = to_string(handle).unwrap();

        assert_eq!(first_string, second_string);

        assert!(release(handle).is_ok());
    }

    #[test]
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
    fn test_retry_connection() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_serialize_deserialize").unwrap();

        assert_eq!(get_state(handle), VcxStateType::VcxStateInitialized as u32);

        connect(handle, None).unwrap();
        connect(handle, None).unwrap();
    }

    #[test]
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
    fn test_process_acceptance_message() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_process_acceptance_message").unwrap();
        let message = serde_json::from_str(INVITE_ACCEPTED_RESPONSE).unwrap();
        assert_eq!(error::SUCCESS.code_num, update_state_with_message(handle, message).unwrap());
    }

    #[test]
    fn test_create_with_invalid_invite_details() {
        let _setup = SetupMocks::init();

        let bad_details = r#"{"id":"mtfjmda","s":{"d":"abc"},"l":"abc","n":"Evernym","v":"avc"},"sa":{"d":"abc","e":"abc","v":"abc"},"sc":"MS-101","sm":"message created","t":"there"}"#;
        let err = create_connection_with_invite("alice", &bad_details).unwrap_err();
        assert_eq!(err.kind(), VcxErrorKind::InvalidJson);
    }

    #[test]
    fn test_void_functions_actually_have_results() {
        let _setup = SetupDefaults::init();

        assert_eq!(set_their_pw_verkey(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_state(1, VcxStateType::VcxStateNone).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_pw_did(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_their_pw_did(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_uuid(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_endpoint(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_agent_verkey(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_pw_verkey(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
    }

    #[test]
    fn test_different_protocol_version() {
        let _setup = SetupMocks::init();

        let handle = create_connection_with_invite("alice", INVITE_DETAIL_V3_STRING).unwrap();

        CONNECTION_MAP.get_mut(handle, |connection| {
            match connection {
                Connections::V3(_) => Ok(()),
            }
        }).unwrap();

        let _serialized = to_string(handle).unwrap();
    }
}
