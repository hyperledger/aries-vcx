use futures::future::FutureExt;

use aries_vcx::error::{VcxError, VcxErrorKind, VcxResult};
use aries_vcx::handlers::connection::public_agent::PublicAgent;

use crate::api_lib::api_handle::object_cache_async::ObjectCacheAsync;

lazy_static! {
    pub static ref PUBLIC_AGENT_MAP: ObjectCacheAsync<PublicAgent> = ObjectCacheAsync::<PublicAgent>::new("public-agent-cache");
}

pub async fn is_valid_handle(handle: u32) -> bool {
    PUBLIC_AGENT_MAP.has_handle(handle).await
}

async fn store_public_agent(agent: PublicAgent) -> VcxResult<u32> {
    PUBLIC_AGENT_MAP.add(agent).await
        .or(Err(VcxError::from(VcxErrorKind::CreatePublicAgent)))
}

pub async fn create_public_agent(source_id: &str, institution_did: &str) -> VcxResult<u32> {
    trace!("create_public_agent >>> source_id: {}, institution_did: {}", source_id, institution_did);
    let agent = PublicAgent::create(source_id, institution_did).await?;
    store_public_agent(agent).await
}

pub async fn download_connection_requests(agent_handle: u32, uids: Option<&Vec<String>>) -> VcxResult<String> {
    trace!("download_connection_requests >>> agent_handle: {}, uids: {:?}", agent_handle, uids);
    PUBLIC_AGENT_MAP.get(agent_handle, |agent, []| async move {
        let requests = agent.download_connection_requests(uids.map(|v| v.clone())).await?;
        let requests = serde_json::to_string(&requests)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to serialize dowloaded connection requests {:?}, err: {:?}", requests, err)))?;
        Ok(requests)
    }.boxed()).await
}

pub async fn download_message(agent_handle: u32, uid: &str) -> VcxResult<String> {
    trace!("download_message >>> agent_handle: {}, uid: {:?}", agent_handle, uid);
    PUBLIC_AGENT_MAP.get(agent_handle, |agent, []| async move {
        let msg = agent.download_message(uid).await?;
        serde_json::to_string(&msg)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to serialize dowloaded message {:?}, err: {:?}", msg, err)))
    }.boxed()).await
}

pub async fn get_service(handle: u32) -> VcxResult<String> {
    PUBLIC_AGENT_MAP.get(handle, |agent, []| async move {
        let service = agent.service()?;
        serde_json::to_string(&service)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to serialize agent service {:?}, err: {:?}", service, err)))
    }.boxed()).await
}

pub async fn to_string(handle: u32) -> VcxResult<String> {
    PUBLIC_AGENT_MAP.get(handle, |agent, []| async move {
        agent.to_string().map_err(|err| err.into())
    }.boxed()).await
}

pub async fn from_string(agent_data: &str) -> VcxResult<u32> {
    let agent = PublicAgent::from_string(agent_data)?;
    PUBLIC_AGENT_MAP.add(agent).await.map_err(|err| err.into())
}

pub fn release(handle: u32) -> VcxResult<()> {
    PUBLIC_AGENT_MAP.release(handle)
        .or(Err(VcxError::from(VcxErrorKind::InvalidHandle)))
}
