use thiserror::Error;

#[derive(Error, Debug)]
#[error("No account found matching given input: {0}")]
pub struct AccountNotFound(pub String);

/// Error closely related to the storage backend
#[derive(Error, Debug)]
#[error(transparent)]
pub struct StorageBackendError {
    #[from]
    pub source: Box<dyn std::error::Error>,
}

/// Error parsing values from backend into expected structures
#[derive(Error, Debug)]
#[error("Couldn't retrieve or decode expected data: {0}")]
pub struct DecodeError(#[from] pub Box<dyn std::error::Error>);

/// Creates an error enum composed of individual error items given as list.  
/// The enum variants are named identically to the error variants provided.
/// From<> impls (to the composition) and Display are automatically derived
/// with the help of thiserror.
/// Usage:
/// errorset!(ComposedError[ErrorVariant1, ErrorVariant2]);

macro_rules! errorset {
    ($errorset_name:ident[$($error_name: ident),*]) => {
        #[derive(Error, Debug)]
        pub enum $errorset_name {
            $(
            #[error("{0}")]
            $error_name(#[from] $error_name),
            )*
            /// Generic error variant - display, backtrace passed onto source anyhow::Error
            /// Useful for chucking in errors from random sources. See usage of anyhow! macro.
            #[error(transparent)]
            ZFhOt01Rdb0Error(#[from] anyhow::Error),
        }
    };
}

errorset!(GetAccountIdError[StorageBackendError, AccountNotFound]);
errorset!(ListAccountsError[StorageBackendError, DecodeError]);

errorset!(GetAccountDetailsError[StorageBackendError, AccountNotFound, DecodeError]);
errorset!(RetrievePendingMessageCountError[StorageBackendError, AccountNotFound]);
errorset!(RetrievePendingMessagesError[StorageBackendError, AccountNotFound]);
errorset!(AddRecipientError[StorageBackendError, AccountNotFound]);
// Same error modes as AddRecipientError
pub type RemoveRecipientError = AddRecipientError;
errorset!(ListRecipientKeysError[StorageBackendError, AccountNotFound]);
errorset!(PersistForwardMessageError[StorageBackendError, AccountNotFound]);

// Manual declaration example
#[derive(Error, Debug)]
pub enum CreateAccountError {
    #[error("Failed to create account in storage layer")]
    StorageBackendError(#[from] StorageBackendError),
    #[error(transparent)]
    ZFhOt01Rdb0Error(#[from] anyhow::Error),
}
