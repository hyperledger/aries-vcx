use aries_vcx::{common::ledger::transactions::into_did_doc, handlers::connection::connection::Connection};

use crate::{
    api_vcx::{api_global::profile::get_main_profile, api_handle::object_cache::ObjectCache},
    errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult},
};

lazy_static! {
    pub static ref CONNECTION_MAP: ObjectCache<Connection> =
        ObjectCache::<Connection>::new("nonmediated-connections-cache");
}

fn store_connection(connection: Connection) -> LibvcxResult<u32> {
    CONNECTION_MAP
        .add(connection)
        .map_err(|e| LibvcxError::from_msg(LibvcxErrorKind::IOError, e.to_string()))
}

fn deserialize<T>(data: &str) -> LibvcxResult<T>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_str(data)
        .map_err(|err| LibvcxError::from_msg(LibvcxErrorKind::InvalidJson, format!("Deserialization failed: {}", err)))
}

pub async fn create_inviter() -> LibvcxResult<u32> {
    trace!("create_inviter >>>");
    store_connection(Connection::create_inviter(&get_main_profile()?).await?)
}

pub async fn create_invitee(invitation: &str) -> LibvcxResult<u32> {
    trace!("create_invitee >>>");
    let profile = get_main_profile()?;
    store_connection(
        Connection::create_invitee(&profile, into_did_doc(&profile, &deserialize(invitation)?).await?).await?,
    )
}

pub fn process_invite(handle: u32, invitation: &str) -> LibvcxResult<u32> {
    trace!("process_invite >>>");
    store_connection(
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
) -> LibvcxResult<u32> {
    trace!("process_request >>>");
    store_connection(
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

pub async fn process_response(handle: u32, response: &str) -> LibvcxResult<u32> {
    trace!("process_response >>>");
    store_connection(
        CONNECTION_MAP
            .get_cloned(handle)?
            .process_response(&get_main_profile()?, deserialize(response)?, None)
            .await?,
    )
}

pub async fn process_ack(handle: u32, message: &str) -> LibvcxResult<u32> {
    trace!("process_ack >>>");
    store_connection(
        CONNECTION_MAP
            .get_cloned(handle)?
            .process_ack(deserialize(message)?)
            .await?,
    )
}

pub async fn send_response(handle: u32) -> LibvcxResult<u32> {
    trace!("send_response >>>");
    store_connection(
        CONNECTION_MAP
            .get_cloned(handle)?
            .send_response(&get_main_profile()?, None)
            .await?,
    )
}

pub async fn send_request(handle: u32, service_endpoint: String, routing_keys: Vec<String>) -> LibvcxResult<u32> {
    trace!("send_request >>>");
    store_connection(
        CONNECTION_MAP
            .get_cloned(handle)?
            .send_request(&get_main_profile()?, service_endpoint, routing_keys, None)
            .await?,
    )
}

pub async fn send_ack(handle: u32) -> LibvcxResult<u32> {
    trace!("send_ack >>>");
    store_connection(
        CONNECTION_MAP
            .get_cloned(handle)?
            .send_ack(&get_main_profile()?, None)
            .await?,
    )
}

pub async fn create_invite(handle: u32, service_endpoint: String, routing_keys: Vec<String>) -> LibvcxResult<u32> {
    trace!("create_invite >>>");
    store_connection(
        CONNECTION_MAP
            .get_cloned(handle)?
            .create_invite(service_endpoint, routing_keys)
            .await?,
    )
}

pub fn from_string(connection_data: &str) -> LibvcxResult<u32> {
    trace!("from_string >>>");
    store_connection(Connection::from_string(connection_data)?)
}

pub fn to_string(handle: u32) -> LibvcxResult<String> {
    trace!("to_string >>>");
    CONNECTION_MAP.get(handle, |connection| connection.to_string().map_err(|err| err.into()))
}

pub fn release(handle: u32) -> LibvcxResult<()> {
    trace!("release >>>");
    CONNECTION_MAP.release(handle)
}

pub fn release_all() {
    trace!("release_all >>>");
    CONNECTION_MAP.drain().ok();
}
