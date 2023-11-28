// DatabaseOperationnError
// AccountNotFoundError

// AccountCreationError {
//     source:
// }
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreateAccountError {
    #[error("Possibly created account, but failed to retrieve created account ID")]
    GetAccountIdError(#[from] GetAccountIdError),
    #[error(transparent)]
    LowerLayerError(#[from] Box<dyn std::error::Error>),
}

#[derive(Error, Debug)]
pub enum GetAccountIdError {
    #[error("No account found matching given input")]
    NotFound,
    #[error(transparent)]
    LowerLayerError(#[from] Box<dyn std::error::Error>),
}

#[derive(Error, Debug)]
pub enum ListAccountsError {
    #[error(transparent)]
    LowerLayerError(#[from] Box<dyn std::error::Error>),
}

#[derive(Error, Debug)]
pub enum GetAccountDetailsError {
    #[error("No account found matching given input")]
    NotFound,
    #[error("Couldn't decode retrieved data to expected account details structure: {0}")]
    DecodeError(String),
    #[error(transparent)]
    LowerLayerError(#[from] Box<dyn std::error::Error>),
}
