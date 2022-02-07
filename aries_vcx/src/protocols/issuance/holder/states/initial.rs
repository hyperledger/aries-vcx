use crate::error::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitialHolderState {}

impl InitialHolderState {
    pub fn new() -> Self {
        Self {}
    }

    pub fn is_revokable(&self) -> VcxResult<bool> {
        Err(VcxError::from_msg(VcxErrorKind::InvalidState, "Revocation information not available in the initial state"))
    }
}
