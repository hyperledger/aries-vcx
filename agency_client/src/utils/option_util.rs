use crate::utils::error::{AgencyClientErrorKind, AgencyClientResult, AgencyClientError};

pub fn get_or_default(config: &Option<String>, default: &str) -> String {
    config.to_owned().unwrap_or(default.to_string())
}

pub fn get_or_err(config: &Option<String>, err: Option<AgencyClientErrorKind>) -> AgencyClientResult<String> {
    let e = AgencyClientError::from(err.unwrap_or(AgencyClientErrorKind::InvalidOption));
    config.to_owned().ok_or(e)
}
