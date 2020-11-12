use agency_client::utils::error::prelude::*;

pub fn get_or_default(config: &Option<String>, default: &str) -> String {
    config.to_owned().unwrap_or(default.to_string())
}

pub fn get_or_err(config: &Option<String>, err: Option<AgencyCommErrorKind>) -> VcxResult<String> {
    let e = AgencyCommError::from(err.unwrap_or(AgencyCommErrorKind::InvalidOption));
    config.to_owned().ok_or(e)
}
