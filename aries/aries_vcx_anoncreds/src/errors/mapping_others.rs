use super::error::VcxAnoncredsError;

impl From<serde_json::Error> for VcxAnoncredsError {
    fn from(value: serde_json::Error) -> Self {
        Self::InvalidJson(value.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for VcxAnoncredsError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        VcxAnoncredsError::InvalidState(err.to_string())
    }
}

impl From<anoncreds_types::Error> for VcxAnoncredsError {
    fn from(err: anoncreds_types::Error) -> Self {
        VcxAnoncredsError::InvalidState(err.to_string())
    }
}
