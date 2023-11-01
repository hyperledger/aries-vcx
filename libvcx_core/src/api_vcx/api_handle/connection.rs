use std::{any::type_name, collections::HashMap, str::FromStr, sync::RwLock};

use aries_vcx::{
    errors::error::{AriesVcxError, VcxResult},
    messages::msg_fields::protocols::connection::request::Request,
    protocols::connection::{
        invitee::InviteeConnection, inviter::InviterConnection, pairwise_info::PairwiseInfo,
        Connection, GenericConnection, State, ThinState,
    },
    transport::Transport,
};
use async_trait::async_trait;
use rand::Rng;
use shared_vcx::http_client::post_message;
use url::Url;

use crate::{
    api_vcx::api_global::profile::{get_main_ledger_read, get_main_wallet},
    errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult},
};

type Map = HashMap<u32, GenericConnection>;
type Cache = RwLock<Map>;

lazy_static! {
    pub static ref CONNECTION_MAP: Cache = RwLock::new(HashMap::new());
}

pub struct HttpClient;

#[async_trait]
impl Transport for HttpClient {
    async fn send_message(&self, msg: Vec<u8>, service_endpoint: Url) -> VcxResult<()> {
        post_message(msg, service_endpoint).await?;
        Ok(())
    }
}

fn new_handle() -> LibvcxResult<u32> {
    loop {
        let handle = rand::thread_rng().gen::<u32>();
        if !CONNECTION_MAP.read()?.contains_key(&handle) {
            break Ok(handle);
        }
    }
}

pub fn get_cloned_generic_connection(handle: &u32) -> LibvcxResult<GenericConnection> {
    CONNECTION_MAP.write()?.get(handle).cloned().ok_or_else(|| {
        LibvcxError::from_msg(
            LibvcxErrorKind::ObjectAccessError,
            format!("No connection found for handle: {}", handle),
        )
    })
}

fn get_con_attribute_with_closure<F, T>(handle: &u32, closure: F) -> LibvcxResult<T>
where
    F: Fn(&GenericConnection) -> LibvcxResult<T>,
{
    let lock = CONNECTION_MAP.read()?;
    let con = lock.get(handle).ok_or_else(|| {
        LibvcxError::from_msg(
            LibvcxErrorKind::ObjectAccessError,
            format!("No connection found for handle: {}", handle),
        )
    })?;

    closure(con)
}

fn get_cloned_connection<I, S>(handle: &u32) -> LibvcxResult<Connection<I, S>>
where
    Connection<I, S>: TryFrom<GenericConnection, Error = AriesVcxError>,
{
    let con = CONNECTION_MAP
        .write()?
        .get(handle)
        .ok_or_else(|| {
            LibvcxError::from_msg(
                LibvcxErrorKind::ObjectAccessError,
                format!("No connection found for handle: {}", handle),
            )
        })?
        .clone()
        .try_into()?;

    Ok(con)
}

fn add_connection<I, S>(connection: Connection<I, S>) -> LibvcxResult<u32>
where
    GenericConnection: From<Connection<I, S>>,
{
    let handle = new_handle()?;
    insert_connection(handle, connection)?;
    Ok(handle)
}

pub fn insert_connection<I, S>(handle: u32, connection: Connection<I, S>) -> LibvcxResult<()>
where
    GenericConnection: From<Connection<I, S>>,
{
    trace!(
        "Inserting connection; Handle: {} - Type: {}",
        &handle,
        type_name::<Connection<I, S>>()
    );

    CONNECTION_MAP.write()?.insert(handle, connection.into());
    Ok(())
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
    serde_json::from_str(data).map_err(|err| {
        LibvcxError::from_msg(
            LibvcxErrorKind::InvalidJson,
            format!("Deserialization failed: {}", err),
        )
    })
}

// ----------------------------- CONSTRUCTORS ------------------------------------
pub async fn create_inviter(pw_info: Option<PairwiseInfo>) -> LibvcxResult<u32> {
    trace!("create_inviter >>>");
    let pw_info = pw_info.unwrap_or(PairwiseInfo::create(get_main_wallet()?.as_ref()).await?);
    let con = InviterConnection::new_inviter("".to_owned(), pw_info);
    add_connection(con)
}

pub async fn create_invitee(_invitation: &str) -> LibvcxResult<u32> {
    trace!("create_invitee >>>");
    let pairwise_info = PairwiseInfo::create(get_main_wallet()?.as_ref()).await?;
    let con = InviteeConnection::new_invitee("".to_owned(), pairwise_info);
    add_connection(con)
}

// ----------------------------- GETTERS ------------------------------------
pub fn get_thread_id(handle: u32) -> LibvcxResult<String> {
    trace!("get_thread_id >>> handle: {}", handle);

    let closure = |con: &GenericConnection| {
        GenericConnection::thread_id(con)
            .map(ToOwned::to_owned)
            .ok_or_else(|| {
                LibvcxError::from_msg(
                    LibvcxErrorKind::ObjectAccessError,
                    format!("No thread ID for connection with handle: {}", &handle),
                )
            })
    };

    get_con_attribute_with_closure(&handle, closure)
}

pub fn get_pairwise_info(handle: u32) -> LibvcxResult<String> {
    trace!("get_pairwise_info >>> handle: {}", handle);

    let closure = |con: &GenericConnection| serialize(GenericConnection::pairwise_info(con));

    get_con_attribute_with_closure(&handle, closure)
}

pub fn get_remote_did(handle: u32) -> LibvcxResult<String> {
    trace!("get_remote_did >>> handle: {}", handle);

    let closure = |con: &GenericConnection| {
        GenericConnection::remote_did(con)
            .map(ToOwned::to_owned)
            .ok_or_else(|| {
                LibvcxError::from_msg(
                    LibvcxErrorKind::ObjectAccessError,
                    format!("No remote DID for connection with handle: {}", handle),
                )
            })
    };

    get_con_attribute_with_closure(&handle, closure)
}

pub fn get_remote_vk(handle: u32) -> LibvcxResult<String> {
    trace!("get_remote_vk >>> handle: {}", handle);

    let closure = |con: &GenericConnection| GenericConnection::remote_vk(con).map_err(From::from);

    get_con_attribute_with_closure(&handle, closure)
}

pub fn get_state(handle: u32) -> LibvcxResult<u32> {
    trace!("get_state >>> handle: {}", handle);

    let closure = |con: &GenericConnection| Ok(GenericConnection::state(con));
    let state = get_con_attribute_with_closure(&handle, closure)?;

    let state_id = match state {
        ThinState::Invitee(s) => s as u32,
        ThinState::Inviter(s) => s as u32,
    };

    Ok(state_id)
}

pub fn get_invitation(handle: u32) -> LibvcxResult<String> {
    trace!("get_invitation >>> handle: {}", handle);

    let closure = |con: &GenericConnection| {
        let invitation = GenericConnection::invitation(con).ok_or_else(|| {
            LibvcxError::from_msg(
                LibvcxErrorKind::ObjectAccessError,
                format!("No invitation for connection with handle: {}", handle),
            )
        })?;

        serialize(invitation)
    };

    get_con_attribute_with_closure(&handle, closure)
}

// ----------------------------- MSG PROCESSING ------------------------------------
pub async fn process_invite(handle: u32, invitation: &str) -> LibvcxResult<()> {
    trace!("process_invite >>>");

    let ledger = get_main_ledger_read()?;
    let invitation = deserialize(invitation)?;
    let con = get_cloned_connection(&handle)?
        .accept_invitation(ledger.as_ref(), invitation)
        .await?;

    insert_connection(handle, con)
}

pub async fn process_request(
    handle: u32,
    request: &str,
    service_endpoint: String,
    routing_keys: Vec<String>,
) -> LibvcxResult<()> {
    trace!("process_request >>>");

    let con = get_cloned_generic_connection(&handle)?;
    let wallet = get_main_wallet()?;
    let request: Request = deserialize(request)?;

    let con = match con.state() {
        ThinState::Inviter(State::Initial) => Connection::try_from(con)
            .map_err(From::from)
            .map(|c| c.into_invited_by_request(&request)),
        ThinState::Inviter(State::Invited) => Connection::try_from(con).map_err(From::from),
        s => Err(LibvcxError::from_msg(
            LibvcxErrorKind::ObjectAccessError,
            format!(
                "Connection with handle {} cannot process a request; State: {:?}",
                handle, s
            ),
        )),
    }?;

    let con = con
        .handle_request(
            wallet.as_ref(),
            request,
            Url::from_str(&service_endpoint).map_err(|err| {
                LibvcxError::from_msg(LibvcxErrorKind::InvalidUrl, err.to_string())
            })?,
            routing_keys,
        )
        .await?;

    insert_connection(handle, con)
}

pub async fn process_response(handle: u32, response: &str) -> LibvcxResult<()> {
    trace!("process_response >>>");

    let con = get_cloned_connection(&handle)?;
    let response = deserialize(response)?;
    let con = con
        .handle_response(get_main_wallet()?.as_ref(), response)
        .await?;

    insert_connection(handle, con)
}

pub async fn process_ack(handle: u32, message: &str) -> LibvcxResult<()> {
    trace!("process_ack >>>");

    let con = get_cloned_connection(&handle)?;
    let msg = deserialize(message)?;
    let con = con.acknowledge_connection(&msg)?;

    insert_connection(handle, con)
}

// In the old implementation this only consumed the ProblemReport without doing
// anything with it and returned the connection in the initial state.
//
// We'll emulate that for backwards compatibility.
pub fn process_problem_report(handle: u32, _problem_report: &str) -> LibvcxResult<()> {
    trace!("process_problem_report >>>");
    let con = get_cloned_generic_connection(&handle)?;
    match con.state() {
        ThinState::Invitee(_) => insert_connection(
            handle,
            Connection::new_invitee("".to_owned(), con.pairwise_info().to_owned()),
        ),
        ThinState::Inviter(_) => insert_connection(
            handle,
            Connection::new_inviter("".to_owned(), con.pairwise_info().to_owned()),
        ),
    }
}

pub async fn send_response(handle: u32) -> LibvcxResult<()> {
    trace!("send_response >>>");

    let con = get_cloned_connection(&handle)?;
    let response = con.get_connection_response_msg();
    con.send_message(get_main_wallet()?.as_ref(), &response.into(), &HttpClient)
        .await?;
    insert_connection(handle, con)
}

pub async fn send_request(
    handle: u32,
    service_endpoint: String,
    routing_keys: Vec<String>,
) -> LibvcxResult<()> {
    trace!("send_request >>>");

    let con = get_cloned_connection(&handle)?;
    let url = Url::from_str(&service_endpoint)
        .map_err(|err| LibvcxError::from_msg(LibvcxErrorKind::InvalidUrl, err.to_string()))?;
    let con = con.prepare_request(url, routing_keys).await?;
    let request = con.get_request().clone();
    con.send_message(get_main_wallet()?.as_ref(), &request.into(), &HttpClient)
        .await?;

    insert_connection(handle, con)
}

pub async fn send_ack(handle: u32) -> LibvcxResult<()> {
    trace!("send_ack >>>");

    let con = get_cloned_connection(&handle)?;
    con.send_message(
        get_main_wallet()?.as_ref(),
        &con.get_ack().into(),
        &HttpClient,
    )
    .await?;
    Ok(())
}

pub async fn send_generic_message(handle: u32, content: String) -> LibvcxResult<()> {
    trace!("send_generic_message >>>");

    let message = serde_json::from_str(&content)?;
    let con = get_cloned_generic_connection(&handle)?;
    con.send_message(get_main_wallet()?.as_ref(), &message, &HttpClient)
        .await?;
    Ok(())
}

pub async fn create_invite(
    handle: u32,
    service_endpoint: String,
    routing_keys: Vec<String>,
) -> LibvcxResult<()> {
    trace!("create_invite >>>");

    let con = get_cloned_connection(&handle)?;
    let con = con.create_invitation(
        routing_keys,
        Url::from_str(&service_endpoint)
            .map_err(|err| LibvcxError::from_msg(LibvcxErrorKind::InvalidUrl, err.to_string()))?,
    );

    insert_connection(handle, con)
}

// // ------------------------- (DE)SERIALIZATION ----------------------------------
pub fn to_string(handle: u32) -> LibvcxResult<String> {
    trace!("to_string >>>");

    CONNECTION_MAP
        .read()?
        .get(&handle)
        .ok_or_else(|| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidHandle,
                format!(
                    "[Connection Cache] get >> Object not found for handle: {}",
                    handle
                ),
            )
        })
        .and_then(serialize)
}

pub fn from_string(connection_data: &str) -> LibvcxResult<u32> {
    trace!("from_string >>>");

    let connection = deserialize(connection_data)?;
    let handle = new_handle()?;
    CONNECTION_MAP.write()?.insert(handle, connection);

    Ok(handle)
}

// --------------------------------------- CLEANUP ---------------------------------------
pub fn release(handle: u32) -> LibvcxResult<()> {
    trace!("release >>>");

    CONNECTION_MAP
        .write()
        .map(|mut map| map.remove(&handle))
        .ok();
    Ok(())
}

pub fn release_all() {
    trace!("release_all >>>");
    CONNECTION_MAP
        .write()
        .map(|mut map| map.drain().for_each(drop))
        .ok();
}
