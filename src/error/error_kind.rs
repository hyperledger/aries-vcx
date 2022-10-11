use failure::{Backtrace, Context, Fail};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum AgentErrorKind {
    #[fail(display = "AriesVCX error")]
    GenericAriesVcxError,
    #[fail(display = "Failed to get invite details")]
    InviteDetails,
    #[fail(display = "No object found with specified ID")]
    NotFound,
    #[fail(display = "Unable to lock storage")]
    LockError,
    #[fail(display = "Serialization error")]
    SerializationError,
    #[fail(display = "Invalid arguments passed")]
    InvalidArguments,
}
