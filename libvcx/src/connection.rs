use std::collections::HashMap;

use rmp_serde;
use serde_json;


use api::VcxStateType;
use error::prelude::*;
use messages;
use messages::{GeneralMessage, MessageStatusCode, RemoteMessageType, SerializableObjectWithState};
use messages::invite::{InviteDetail, RedirectDetail, SenderDetail, Payload as ConnectionPayload};
use messages::payload::{Payloads, PayloadKinds};
use messages::thread::Thread;
use messages::send_message::SendMessageOptions;
use messages::get_message::{Message, MessagePayload};
use object_cache::ObjectCache;
use settings;
use utils::error;
use utils::libindy::signus::create_and_store_my_did;
use utils::libindy::crypto;

use utils::json::KeyMatch;

use v3::handlers::connection::connection::Connection as ConnectionV3;
use v3::handlers::connection::states::ActorDidExchangeState;
use v3::handlers::connection::agent::AgentInfo;
use v3::messages::connection::invite::Invitation as InvitationV3;
use v3::messages::connection::did_doc::DidDoc;
use v3::messages::a2a::A2AMessage;
use settings::ProtocolTypes;

lazy_static! {
    static ref CONNECTION_MAP: ObjectCache<Connections> = Default::default();
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
    redirect_detail: Option<RedirectDetail>,
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

impl Connection {
    fn _connect_send_invite(&mut self, options: &ConnectionOptions) -> VcxResult<u32> {
        debug!("sending invite for connection {}", self.source_id);

        let (invite, url) =
            messages::send_invite()
                .to(&self.pw_did)?
                .to_vk(&self.pw_verkey)?
                .phone_number(options.phone.as_ref().map(String::as_str))?
                .agent_did(&self.agent_did)?
                .agent_vk(&self.agent_vk)?
                .public_did(self.public_did.as_ref().map(String::as_str))?
                .thread(&Thread::new())?
                .version(&self.version)?
                .send_secure()
                .map_err(|err| err.extend("Cannot send invite"))?;

        self.state = VcxStateType::VcxStateOfferSent;
        self.invite_detail = Some(invite);
        self.invite_url = Some(url);

        Ok(error::SUCCESS.code_num)
    }

    pub fn delete_connection(&mut self) -> VcxResult<u32> {
        trace!("Connection::delete_connection >>>");

        messages::delete_connection()
            .to(&self.pw_did)?
            .to_vk(&self.pw_verkey)?
            .agent_did(&self.agent_did)?
            .agent_vk(&self.agent_vk)?
            .version(&self.version)?
            .send_secure()
            .map_err(|err| err.extend("Cannot delete connection"))?;

        self.state = VcxStateType::VcxStateNone;

        Ok(error::SUCCESS.code_num)
    }

    fn _connect_accept_invite(&mut self) -> VcxResult<u32> {
        debug!("accepting invite for connection {}", self.source_id);

        let details: &InviteDetail = self.invite_detail.as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::GeneralConnectionError, format!("Invite details not found for: {}", self.source_id)))?;

        messages::accept_invite()
            .to(&self.pw_did)?
            .to_vk(&self.pw_verkey)?
            .agent_did(&self.agent_did)?
            .agent_vk(&self.agent_vk)?
            .sender_details(&details.sender_detail)?
            .sender_agency_details(&details.sender_agency_detail)?
            .answer_status_code(&MessageStatusCode::Accepted)?
            .reply_to(&details.conn_req_id)?
            .thread(&self._build_thread(&details))?
            .version(self.version.clone())?
            .send_secure()
            .map_err(|err| err.extend("Cannot accept invite"))?;

        self.state = VcxStateType::VcxStateAccepted;

        Ok(error::SUCCESS.code_num)
    }

    fn _build_thread(&self, invite_detail: &InviteDetail) -> Thread {
        let mut received_orders = HashMap::new();
        received_orders.insert(invite_detail.sender_detail.did.clone(), 0);
        Thread {
            thid: invite_detail.thread_id.clone(),
            pthid: None,
            sender_order: 0,
            received_orders,
        }
    }

    fn connect(&mut self, options: &ConnectionOptions) -> VcxResult<u32> {
        trace!("Connection::connect >>> options: {:?}", options);
        match self.state {
            VcxStateType::VcxStateInitialized
            | VcxStateType::VcxStateOfferSent => self._connect_send_invite(options),
            VcxStateType::VcxStateRequestReceived => self._connect_accept_invite(),
            _ => {
                warn!("connection {} in state {} not ready to connect", self.source_id, self.state as u32);
                // TODO: Refactor Error
                // TODO: Implement Correct Error
                Err(VcxError::from_msg(VcxErrorKind::GeneralConnectionError, format!("Connection {} in state {} not ready to connect", self.source_id, self.state as u32)))
            }
        }
    }

    fn get_state(&self) -> u32 {
        trace!("Connection::get_state >>>");
        self.state as u32
    }
    fn set_state(&mut self, state: VcxStateType) {
        trace!("Connection::set_state >>> state: {:?}", state);
        self.state = state;
    }

    fn get_pw_did(&self) -> &String { &self.pw_did }
    fn set_pw_did(&mut self, did: &str) { self.pw_did = did.to_string(); }

    fn get_their_pw_did(&self) -> &String { &self.their_pw_did }
    fn set_their_pw_did(&mut self, did: &str) { self.their_pw_did = did.to_string(); }

    fn set_their_public_did(&mut self, did: &str) { self.their_public_did = Some(did.to_string()); }
    fn get_their_public_did(&self) -> Option<String> { self.their_public_did.clone() }

    fn get_agent_did(&self) -> &String { &self.agent_did }
    fn set_agent_did(&mut self, did: &str) { self.agent_did = did.to_string(); }

    fn get_pw_verkey(&self) -> &String { &self.pw_verkey }
    fn set_pw_verkey(&mut self, verkey: &str) { self.pw_verkey = verkey.to_string(); }

    fn get_their_pw_verkey(&self) -> &String { &self.their_pw_verkey }
    fn set_their_pw_verkey(&mut self, verkey: &str) { self.their_pw_verkey = verkey.to_string(); }

    fn get_agent_verkey(&self) -> &String { &self.agent_vk }
    fn set_agent_verkey(&mut self, verkey: &str) { self.agent_vk = verkey.to_string(); }

    fn get_uuid(&self) -> &String { &self.uuid }
    fn set_uuid(&mut self, uuid: &str) { self.uuid = uuid.to_string(); }

    fn get_endpoint(&self) -> &String { &self.endpoint }
    fn set_endpoint(&mut self, endpoint: &str) { self.endpoint = endpoint.to_string(); }

    fn get_invite_detail(&self) -> &Option<InviteDetail> { &self.invite_detail }
    fn set_invite_detail(&mut self, id: InviteDetail) {
        self.version = match id.version.is_some() {
            true => Some(settings::ProtocolTypes::from(id.version.clone().unwrap())),
            false => Some(settings::get_connecting_protocol_version()),
        };
        self.invite_detail = Some(id);
    }

    fn get_version(&self) -> Option<settings::ProtocolTypes> { self.version.clone() }

    fn get_source_id(&self) -> &String { &self.source_id }

    fn create_agent_pairwise(&mut self) -> VcxResult<u32> {
        debug!("creating pairwise keys on agent for connection {}", self.source_id);

        let (for_did, for_verkey) = messages::create_keys()
            .for_did(&self.pw_did)?
            .for_verkey(&self.pw_verkey)?
            .version(&self.version)?
            .send_secure()
            .map_err(|err| err.extend("Cannot create pairwise keys"))?;

        debug!("create key for connection: {} with did {:?}, vk: {:?}", self.source_id, for_did, for_verkey);
        self.set_agent_did(&for_did);
        self.set_agent_verkey(&for_verkey);

        Ok(error::SUCCESS.code_num)
    }

    fn update_agent_profile(&mut self, options: &ConnectionOptions) -> VcxResult<u32> {
        debug!("updating agent config for connection {}", self.source_id);

        if let Some(true) = options.use_public_did {
            self.public_did = Some(settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?);
        };

        let webhook_url = settings::get_config_value(settings::CONFIG_WEBHOOK_URL).ok();

        if let Ok(name) = settings::get_config_value(settings::CONFIG_INSTITUTION_NAME) {
            messages::update_data()
                .to(&self.pw_did)?
                .name(&name)?
                .logo_url(&settings::get_config_value(settings::CONFIG_INSTITUTION_LOGO_URL)?)?
                .webhook_url(&webhook_url)?
                .use_public_did(&self.public_did)?
                .version(&self.version)?
                .send_secure()
                .map_err(|err| err.extend("Cannot update agent profile"))?;
        }

        Ok(error::SUCCESS.code_num)
    }

    pub fn send_generic_message(&self, message: &str, msg_options: &str) -> VcxResult<String> {
        if self.state != VcxStateType::VcxStateAccepted {
            return Err(VcxError::from(VcxErrorKind::NotReady));
        }

        let msg_options: SendMessageOptions = serde_json::from_str(msg_options).map_err(|_| {
            error!("Invalid SendMessage msg_options");
            VcxError::from(VcxErrorKind::InvalidConfiguration)
        })?;

        let response =
            ::messages::send_message()
                .to(&self.get_pw_did())?
                .to_vk(&self.get_pw_verkey())?
                .msg_type(&RemoteMessageType::Other(msg_options.msg_type.clone()))?
                .edge_agent_payload(&self.get_pw_verkey(), &self.get_their_pw_verkey(), &message, PayloadKinds::Other(msg_options.msg_type.clone()), None)?
                .agent_did(&self.get_agent_did())?
                .agent_vk(&self.get_agent_verkey())?
                .set_title(&msg_options.msg_title)?
                .set_detail(&msg_options.msg_title)?
                .ref_msg_id(msg_options.ref_msg_id.clone())?
                .status_code(&MessageStatusCode::Accepted)?
                .send_secure()?;

        let msg_uid = response.get_msg_uid()?;
        return Ok(msg_uid);
    }
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
            Connections::V1(ref mut connection) => Ok(connection.set_agent_did(did)),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_agent_did(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(ref connection) => Ok(connection.get_agent_did().clone()),
            Connections::V3(ref connection) => Ok(connection.agent_info().agent_did.to_string())
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_pw_did(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(ref connection) => Ok(connection.get_pw_did().to_string()),
            Connections::V3(ref connection) => Ok(connection.agent_info().pw_did.to_string())
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_ver_str(handle: u32) -> VcxResult<Option<String>> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(ref connection) => Ok(connection.get_version().as_ref().map(ProtocolTypes::to_string)),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn set_pw_did(handle: u32, did: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => Ok(connection.set_pw_did(did)),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_their_pw_did(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(ref connection) => Ok(connection.get_their_pw_did().to_string()),
            Connections::V3(ref connection) => connection.remote_did()
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn set_their_pw_did(handle: u32, did: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => Ok(connection.set_their_pw_did(did)),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn set_their_public_did(handle: u32, did: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => Ok(connection.set_their_public_did(did)),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_their_public_did(handle: u32) -> VcxResult<Option<String>> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(ref connection) => Ok(connection.get_their_public_did()),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_their_pw_verkey(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(ref connection) => Ok(connection.get_their_pw_verkey().to_string()),
            Connections::V3(ref connection) => connection.remote_vk()
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn set_their_pw_verkey(handle: u32, did: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |cxn| {
        match cxn {
            Connections::V1(ref mut connection) => Ok(connection.set_their_pw_verkey(did)),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_uuid(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(ref connection) => Ok(connection.get_uuid().to_string()),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn set_uuid(handle: u32, uuid: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |cxn| {
        match cxn {
            Connections::V1(ref mut connection) => Ok(connection.set_uuid(uuid)),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

// TODO: Add NO_ENDPOINT error to connection error
pub fn get_endpoint(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(ref connection) => Ok(connection.get_endpoint().to_string()),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::NoEndpoint)))
}

pub fn set_endpoint(handle: u32, endpoint: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |cxn| {
        match cxn {
            Connections::V1(ref mut connection) => Ok(connection.set_endpoint(endpoint)),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_agent_verkey(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(ref connection) => Ok(connection.get_agent_verkey().clone()),
            Connections::V3(ref connection) => Ok(connection.agent_info().agent_vk.clone())
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_version(handle: u32) -> VcxResult<Option<ProtocolTypes>> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(ref connection) => Ok(connection.get_version()),
            Connections::V3(_) => Ok(Some(settings::get_protocol_type()))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn set_agent_verkey(handle: u32, verkey: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |cxn| {
        match cxn {
            Connections::V1(ref mut connection) => Ok(connection.set_agent_verkey(verkey).clone()),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::ActionNotSupported))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_pw_verkey(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(ref connection) => Ok(connection.get_pw_verkey().clone()),
            Connections::V3(ref connection) => Ok(connection.agent_info().pw_vk.clone())
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn set_pw_verkey(handle: u32, verkey: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |cxn| {
        match cxn {
            Connections::V1(ref mut connection) => Ok(connection.set_pw_verkey(verkey).clone()),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_state(handle: u32) -> u32 {
    trace!("get_state >>> handle = {:?}", handle);
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(ref connection) => Ok(connection.get_state()),
            Connections::V3(ref connection) => Ok(connection.state())
        }
    }).unwrap_or(0)
}


pub fn set_state(handle: u32, state: VcxStateType) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |cxn| {
        match cxn {
            Connections::V1(ref mut connection) => Ok(connection.set_state(state)),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_source_id(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(ref connection) => Ok(connection.get_source_id().clone()),
            Connections::V3(ref connection) => Ok(connection.get_source_id())
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

fn store_connection(connection: Connections) -> VcxResult<u32> {
    CONNECTION_MAP.add(connection)
        .or(Err(VcxError::from(VcxErrorKind::CreateConnection)))
}

fn create_connection_v1(source_id: &str) -> VcxResult<Connection> {
    let method_name = settings::get_config_value(settings::CONFIG_DID_METHOD).ok();

    let (pw_did, pw_verkey) = create_and_store_my_did(None, method_name.as_ref().map(String::as_str))?;

    Ok(Connection {
        source_id: source_id.to_string(),
        pw_did,
        pw_verkey,
        state: VcxStateType::VcxStateInitialized,
        uuid: String::new(),
        endpoint: String::new(),
        invite_detail: None,
        redirect_detail: None,
        invite_url: None,
        agent_did: String::new(),
        agent_vk: String::new(),
        their_pw_did: String::new(),
        their_pw_verkey: String::new(),
        public_did: None,
        their_public_did: None,
        version: Some(settings::get_connecting_protocol_version()),
    })
}

pub fn create_connection(source_id: &str) -> VcxResult<u32> {
    trace!("create_connection >>> source_id: {}", source_id);

    // Initiate connection of new format -- redirect to v3 folder
    if settings::is_aries_protocol_set() {
        warn!("Creating aries connection");
        let connection = Connections::V3(ConnectionV3::create(source_id));
        return store_connection(connection);
    }
    error!("Creating V1 connection");
    if ::std::env::var("DISALLOW_V1").unwrap_or("true".to_string()) == "true"
    {
        panic!("Trying to create non-aries connection!");
    }

    let connection = create_connection_v1(source_id)?;

    store_connection(Connections::V1(connection))
}

pub fn create_connection_with_invite(source_id: &str, details: &str) -> VcxResult<u32> {
    debug!("create connection {} with invite {}", source_id, details);

    if let Some(invitation) = serde_json::from_str::<InvitationV3>(details).ok() {
        let connection = Connections::V3(ConnectionV3::create_with_invite(source_id, invitation)?);
        store_connection(connection)
    } else {
        Err(VcxError::from(VcxErrorKind::ActionNotSupported))
    }
}

pub fn send_generic_message(connection_handle: u32, msg: &str, msg_options: &str) -> VcxResult<String> {
    CONNECTION_MAP.get(connection_handle, |connection| {
        match connection {
            Connections::V1(ref connection) => connection.send_generic_message(&msg, &msg_options),
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
            Connections::V1(ref mut connection) => {
                connection.delete_connection()
            }
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
            Connections::V1(ref mut connection) => {
                debug!("establish connection {}", connection.source_id);
                connection.update_agent_profile(&options_obj)?;
                connection.create_agent_pairwise()?;
                connection.connect(&options_obj)
            }
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
            Connections::V1(ref connection) => {
                let object: SerializableObjectWithState<Connection, ConnectionV3> = SerializableObjectWithState::V1 { data: connection.to_owned() };

                ::serde_json::to_string(&object)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidState, format!("Cannot serialize Connection: {:?}", err)))
            }
            Connections::V3(ref connection) => {
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
        SerializableObjectWithState::V1 { data, .. } => {
            CONNECTION_MAP.add(Connections::V1(data))?
        }
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

pub fn get_invite_details(handle: u32, _abbreviated: bool) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::ActionNotSupported)),
            Connections::V3(ref connection) => {
                connection.get_invite_details()
            }
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
            redirect_detail: None,
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
            pw_did: connection.get_pw_did().to_string(),
            pw_vk: connection.get_pw_verkey().to_string(),
            agent_did: connection.get_agent_did().to_string(),
            agent_vk: connection.get_agent_verkey().to_string(),
        };

        ConnectionV3::from_parts(connection.get_source_id().to_string(), agent_info, state)
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

    use super::*;
    use utils::devsetup::*;
    use utils::httpclient::AgencyMockDecrypted;
    use utils::constants;

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
        connect(alice_to_faber,  None).unwrap();
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
        AgencyMockDecrypted::set_next_decrypted_message(constants::CONNECTION_REQUEST);
        update_state(handle, None).unwrap();
        assert_eq!(get_state(handle), VcxStateType::VcxStateRequestReceived as u32);

        AgencyMockDecrypted::set_next_decrypted_response(constants::GET_MESSAGES_DECRYPTED_RESPONSE);
        AgencyMockDecrypted::set_next_decrypted_message(constants::ACK_RESPONSE);
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

        let handle = create_connection_with_invite("alice", constants::INVITE_DETAIL_V3_STRING).unwrap();
        connect(handle, None).unwrap();

        let handle_2 = create_connection_with_invite("alice", constants::INVITE_DETAIL_V3_STRING).unwrap();
        connect(handle_2, None).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_process_acceptance_message() {
        let _setup = SetupAriesMocks::init();

        let handle = create_connection("test_process_acceptance_message").unwrap();
        let message = serde_json::from_str(constants::CONNECTION_REQUEST).unwrap();
        assert_eq!(error::SUCCESS.code_num, update_state_with_message(handle, message).unwrap());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_with_legacy_invite_details() {
        let _setup = SetupAriesMocks::init();

        let err = create_connection_with_invite("alice", constants::INVITE_DETAIL_V1_STRING).unwrap_err();
        assert_eq!(err.kind(), VcxErrorKind::ActionNotSupported);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_void_functions_actually_have_results() {
        let _setup = SetupAriesMocks::init();

        assert_eq!(set_their_pw_verkey(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_state(1, VcxStateType::VcxStateNone).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_pw_did(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_their_pw_did(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_uuid(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_endpoint(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_pw_verkey(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_different_protocol_version() {
        let _setup = SetupMocks::init();
        let err = create_connection_with_invite("alice", INVITE_DETAIL_V1_STRING).unwrap_err();
        assert_eq!(err.kind(), VcxErrorKind::ActionNotSupported);

        let _setup = SetupMocks::init();
        let handle = create_connection_with_invite("alice", INVITE_DETAIL_V3_STRING).unwrap();

        CONNECTION_MAP.get_mut(handle, |connection| {
            match connection {
                Connections::V1(_) => Err(VcxError::from_msg(VcxErrorKind::InvalidState, "It is suppose to be V3")),
                Connections::V3(_) => Ok(()),
            }
        }).unwrap();

        let _serialized = to_string(handle).unwrap();
    }
}
