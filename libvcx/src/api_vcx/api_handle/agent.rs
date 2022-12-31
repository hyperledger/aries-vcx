use aries_vcx::handlers::connection::public_agent::PublicAgent;

use crate::api_vcx::api_global::agency_client::get_main_agency_client;
use crate::api_vcx::api_global::profile::get_main_profile;
use crate::api_vcx::api_handle::object_cache::ObjectCache;
use crate::errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult};
lazy_static! {
    pub static ref PUBLIC_AGENT_MAP: ObjectCache<PublicAgent> = ObjectCache::<PublicAgent>::new("public-agent-cache");
}

pub async fn is_valid_handle(handle: u32) -> bool {
    PUBLIC_AGENT_MAP.has_handle(handle)
}

fn store_public_agent(agent: PublicAgent) -> LibvcxResult<u32> {
    PUBLIC_AGENT_MAP
        .add(agent)
        .map_err(|e| LibvcxError::from_msg(LibvcxErrorKind::CreatePublicAgent, e.to_string()))
}

pub async fn create_public_agent(source_id: &str, institution_did: &str) -> LibvcxResult<u32> {
    trace!(
        "create_public_agent >>> source_id: {}, institution_did: {}",
        source_id,
        institution_did
    );
    let profile = get_main_profile()?;
    let agent = PublicAgent::create(&profile, &get_main_agency_client().unwrap(), source_id, institution_did).await?;
    store_public_agent(agent)
}

pub async fn download_connection_requests(handle: u32, uids: Option<&Vec<String>>) -> LibvcxResult<String> {
    trace!("download_connection_requests >>> handle: {}, uids: {:?}", handle, uids);
    let agent = PUBLIC_AGENT_MAP.get_cloned(handle)?;
    let requests = agent
        .download_connection_requests(&get_main_agency_client().unwrap(), uids.cloned())
        .await?;
    let requests = serde_json::to_string(&requests).map_err(|err| {
        LibvcxError::from_msg(
            LibvcxErrorKind::SerializationError,
            format!(
                "Failed to serialize dowloaded connection requests {:?}, err: {:?}",
                requests, err
            ),
        )
    })?;
    Ok(requests)
}

pub async fn download_message(handle: u32, uid: &str) -> LibvcxResult<String> {
    trace!("download_message >>> handle: {}, uid: {:?}", handle, uid);
    let agent = PUBLIC_AGENT_MAP.get_cloned(handle)?;
    let msg = agent.download_message(&get_main_agency_client().unwrap(), uid).await?;
    serde_json::to_string(&msg).map_err(|err| {
        LibvcxError::from_msg(
            LibvcxErrorKind::SerializationError,
            format!("Failed to serialize dowloaded message {:?}, err: {:?}", msg, err),
        )
    })
}

pub fn get_service(handle: u32) -> LibvcxResult<String> {
    PUBLIC_AGENT_MAP.get(handle, |agent| {
        let service = agent.service(&get_main_agency_client().unwrap())?;
        serde_json::to_string(&service).map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::SerializationError,
                format!("Failed to serialize agent service {:?}, err: {:?}", service, err),
            )
        })
    })
}

pub fn to_string(handle: u32) -> LibvcxResult<String> {
    PUBLIC_AGENT_MAP.get(handle, |agent| agent.to_string().map_err(|err| err.into()))
}

pub fn from_string(agent_data: &str) -> LibvcxResult<u32> {
    let agent = PublicAgent::from_string(agent_data)?;
    PUBLIC_AGENT_MAP.add(agent)
}

pub fn release(handle: u32) -> LibvcxResult<()> {
    PUBLIC_AGENT_MAP
        .release(handle)
        .map_err(|e| LibvcxError::from_msg(LibvcxErrorKind::InvalidHandle, e.to_string()))
}
