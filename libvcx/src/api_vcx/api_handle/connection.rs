use aries_vcx::{
    common::ledger::transactions::into_did_doc, handlers::connection::connection::Connection,
    protocols::connection::pairwise_info::PairwiseInfo,
};

use crate::{
    api_vcx::{api_global::profile::get_main_profile, api_handle::object_cache::ObjectCache},
    errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult},
};

lazy_static! {
    static ref CONNECTION_MAP: ObjectCache<Connection> =
        ObjectCache::<Connection>::new("nonmediated-connections-cache");
}

fn store_connection(connection: Connection) -> LibvcxResult<u32> {
    CONNECTION_MAP
        .add(connection)
        .map_err(|e| LibvcxError::from_msg(LibvcxErrorKind::IOError, e.to_string()))
}

fn serialize<T>(data: &T) -> LibvcxResult<String>
where
    T: serde::ser::Serialize,
{
    serde_json::to_string(data).map_err(|err| {
        LibvcxError::from_msg(
            LibvcxErrorKind::SerializationError,
            format!("Serialization failed: {}", err),
        )
    })
}

fn deserialize<T>(data: &str) -> LibvcxResult<T>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_str(data)
        .map_err(|err| LibvcxError::from_msg(LibvcxErrorKind::InvalidJson, format!("Deserialization failed: {}", err)))
}

// ----------------------------- CONSTRUCTORS ------------------------------------
pub async fn create_inviter(pw_info: Option<PairwiseInfo>) -> LibvcxResult<u32> {
    trace!("create_inviter >>>");
    store_connection(Connection::create_inviter(&get_main_profile()?, pw_info).await?)
}

pub async fn create_invitee(invitation: &str) -> LibvcxResult<u32> {
    trace!("create_invitee >>>");
    let profile = get_main_profile()?;
    store_connection(
        Connection::create_invitee(&profile, into_did_doc(&profile, &deserialize(invitation)?).await?).await?,
    )
}

// ----------------------------- GETTERS ------------------------------------
pub fn get_thread_id(handle: u32) -> LibvcxResult<String> {
    trace!("get_thread_id >>> handle: {}", handle);
    CONNECTION_MAP.get(handle, |connection| Ok(connection.get_thread_id()))
}

pub fn get_pairwise_info(handle: u32) -> LibvcxResult<String> {
    trace!("get_pairwise_info >>> handle: {}", handle);
    CONNECTION_MAP.get(handle, |connection| serialize(connection.pairwise_info()))
}

pub fn get_remote_did(handle: u32) -> LibvcxResult<String> {
    trace!("get_remote_did >>> handle: {}", handle);
    CONNECTION_MAP.get(handle, |connection| connection.remote_did().map_err(|e| e.into()))
}

pub fn get_remote_vk(handle: u32) -> LibvcxResult<String> {
    trace!("get_remote_vk >>> handle: {}", handle);
    CONNECTION_MAP.get(handle, |connection| connection.remote_vk().map_err(|e| e.into()))
}

pub fn get_state(handle: u32) -> LibvcxResult<u32> {
    trace!("get_state >>> handle: {}", handle);
    CONNECTION_MAP.get(handle, |connection| Ok(connection.get_state().into()))
}

pub fn get_invitation(handle: u32) -> LibvcxResult<String> {
    trace!("get_invitation >>> handle: {}", handle);
    CONNECTION_MAP.get(handle, |connection| {
        serialize(connection.get_invite_details().ok_or(LibvcxError::from_msg(
            LibvcxErrorKind::ActionNotSupported,
            "Invitation is not available for the connection.",
        ))?)
    })
}

// ----------------------------- MSG PROCESSING ------------------------------------
pub fn process_invite(handle: u32, invitation: &str) -> LibvcxResult<()> {
    trace!("process_invite >>>");
    CONNECTION_MAP.insert(
        handle,
        CONNECTION_MAP
            .get_cloned(handle)?
            .process_invite(deserialize(invitation)?)?,
    )
}

pub async fn process_request(
    handle: u32,
    request: &str,
    service_endpoint: String,
    routing_keys: Vec<String>,
) -> LibvcxResult<()> {
    trace!("process_request >>>");
    CONNECTION_MAP.insert(
        handle,
        CONNECTION_MAP
            .get_cloned(handle)?
            .process_request(
                &get_main_profile()?,
                deserialize(request)?,
                service_endpoint,
                routing_keys,
                None,
            )
            .await?,
    )
}

pub async fn process_response(handle: u32, response: &str) -> LibvcxResult<()> {
    trace!("process_response >>>");
    CONNECTION_MAP.insert(
        handle,
        CONNECTION_MAP
            .get_cloned(handle)?
            .process_response(&get_main_profile()?, deserialize(response)?, None)
            .await?,
    )
}

pub async fn process_ack(handle: u32, message: &str) -> LibvcxResult<()> {
    trace!("process_ack >>>");
    CONNECTION_MAP.insert(
        handle,
        CONNECTION_MAP
            .get_cloned(handle)?
            .process_ack(deserialize(message)?)
            .await?,
    )
}

pub async fn send_response(handle: u32) -> LibvcxResult<()> {
    trace!("send_response >>>");
    CONNECTION_MAP.insert(
        handle,
        CONNECTION_MAP
            .get_cloned(handle)?
            .send_response(&get_main_profile()?, None)
            .await?,
    )
}

pub async fn send_request(handle: u32, service_endpoint: String, routing_keys: Vec<String>) -> LibvcxResult<()> {
    trace!("send_request >>>");
    CONNECTION_MAP.insert(
        handle,
        CONNECTION_MAP
            .get_cloned(handle)?
            .send_request(&get_main_profile()?, service_endpoint, routing_keys, None)
            .await?,
    )
}

pub async fn send_ack(handle: u32) -> LibvcxResult<()> {
    trace!("send_ack >>>");
    CONNECTION_MAP.insert(
        handle,
        CONNECTION_MAP
            .get_cloned(handle)?
            .send_ack(&get_main_profile()?, None)
            .await?,
    )
}

pub async fn create_invite(handle: u32, service_endpoint: String, routing_keys: Vec<String>) -> LibvcxResult<()> {
    trace!("create_invite >>>");
    CONNECTION_MAP.insert(
        handle,
        CONNECTION_MAP
            .get_cloned(handle)?
            .create_invite(service_endpoint, routing_keys)
            .await?,
    )
}

// ------------------------- (DE)SERIALIZATION ----------------------------------
pub fn to_string(handle: u32) -> LibvcxResult<String> {
    trace!("to_string >>>");
    CONNECTION_MAP.get(handle, |connection| connection.to_string().map_err(|err| err.into()))
}

pub fn from_string(connection_data: &str) -> LibvcxResult<u32> {
    trace!("from_string >>>");
    store_connection(Connection::from_string(connection_data)?)
}

// ------------------------------ CLEANUP ---------------------------------------
pub fn release(handle: u32) -> LibvcxResult<()> {
    trace!("release >>>");
    CONNECTION_MAP.release(handle)
}

pub fn release_all() {
    trace!("release_all >>>");
    CONNECTION_MAP.drain().ok();
}
