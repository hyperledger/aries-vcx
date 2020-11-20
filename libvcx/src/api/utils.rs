use crate::error::VcxResult;
use crate::utils::provision::connect_register_provision;

pub fn provision_agent(config: &str) -> VcxResult<String>  {
    connect_register_provision(&config)
}
