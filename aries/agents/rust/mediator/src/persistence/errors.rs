// DatabaseOperationnError
// AccountNotFoundError

// AccountCreationError {
//     source:
// }
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreateAccountError {
    #[error("Failed to create account in storage layer")]
    StorageBackendError { source: anyhow::Error },
    #[error("Possibly created account, but failed to retrieve created account's ID")]
    GetAccountDetailsError(#[from] GetAccountDetailsError),
    #[error(transparent)]
    hkpLXwHUQError(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum GetAccountIdError {
    #[error("No account found matching given input")]
    NotFound,
    #[error(transparent)]
    hkpLXwHUQError(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum ListAccountsError {
    #[error(transparent)]
    hkpLXwHUQError(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum GetAccountDetailsError {
    #[error("No account found matching given input")]
    NotFound,
    #[error("Couldn't retrieve or decode expected account details: {0}")]
    DecodeError(String),
    #[error(transparent)]
    hkpLXwHUQError(#[from] anyhow::Error),
}
