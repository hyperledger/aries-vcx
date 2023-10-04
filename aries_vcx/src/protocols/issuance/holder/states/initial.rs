use crate::errors::error::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitialHolderState;

impl InitialHolderState {
    pub fn is_revokable(&self) -> VcxResult<bool> {
        Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidState,
            "Revocation information not available in the initial state",
        ))
    }
}
