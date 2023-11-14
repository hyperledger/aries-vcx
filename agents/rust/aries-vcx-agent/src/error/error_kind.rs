#[derive(Copy, Clone, Eq, PartialEq, Debug, thiserror::Error)]
pub enum AgentErrorKind {
    #[error("AriesVCX error")]
    GenericAriesVcxError,
    #[error("Failed to get invite details")]
    InviteDetails,
    #[error("No object found with specified ID")]
    NotFound,
    #[error("Unable to lock storage")]
    LockError,
    #[error("Serialization error")]
    SerializationError,
    #[error("Invalid arguments passed")]
    InvalidArguments,
    #[error("Credential definition already exists on the ledger")]
    CredDefAlreadyCreated,
    #[error("Mediated connections not configured")]
    MediatedConnectionServiceUnavailable,
    #[error("Failed to submit http request")]
    PostMessageFailed,
}
