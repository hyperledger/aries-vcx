use crate::aries_vcx::handlers::connection::public_agent::PublicAgent;
use crate::api_lib::api_handle::object_cache::ObjectCache;
use crate::error::prelude::*;

lazy_static! {
    static ref PUBLIC_AGENT_MAP: ObjectCache<PublicAgent> = ObjectCache::<PublicAgent>::new("public-agent-cache");
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
