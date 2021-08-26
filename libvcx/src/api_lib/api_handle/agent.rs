use crate::aries_vcx::handlers::connection::public_agent::PublicAgent;
use crate::api_lib::api_handle::object_cache::ObjectCache;
use crate::error::prelude::*;

lazy_static! {
    pub static ref PUBLIC_AGENT_MAP: ObjectCache<PublicAgent> = ObjectCache::<PublicAgent>::new("public-agent-cache");
}

pub fn is_valid_handle(handle: u32) -> bool {
    PUBLIC_AGENT_MAP.has_handle(handle)
}

fn store_public_agent(agent: PublicAgent) -> VcxResult<u32> {
    PUBLIC_AGENT_MAP.add(agent)
        .or(Err(VcxError::from(VcxErrorKind::CreatePublicAgent)))
}

pub fn create_public_agent(institution_did: &str) -> VcxResult<u32> {
    trace!("create_public_agent >>> institution_did: {}", institution_did);
    let agent = PublicAgent::create(institution_did)?;
    return store_public_agent(agent);
}

pub fn generate_public_invite(agent_handle: u32, label: &str) -> VcxResult<String> {
    trace!("generate_public_invite >>> agent_handle: {}, label: {}", agent_handle, label);
    PUBLIC_AGENT_MAP.get(agent_handle, |agent| {
        let invite = agent.generate_public_invite(label)?;
        let invite = serde_json::to_string(&invite)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to serialize public invite {:?}, err: {:?}", invite, err)))?;
        Ok(invite)
    })
}

pub fn to_string(handle: u32) -> VcxResult<String> {
    PUBLIC_AGENT_MAP.get(handle, |agent| {
        agent.to_string().map_err(|err| err.into())
    })
}

pub fn from_string(agent_data: &str) -> VcxResult<u32> {
    let agent = PublicAgent::from_string(agent_data)?;
    PUBLIC_AGENT_MAP.add(agent).map_err(|err| err.into())
}

pub fn release(handle: u32) -> VcxResult<()> {
    PUBLIC_AGENT_MAP.release(handle)
        .or(Err(VcxError::from(VcxErrorKind::InvalidHandle)))
}
