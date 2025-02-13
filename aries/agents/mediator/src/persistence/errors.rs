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
/// errorset!(ComposedError\[ErrorVariant1, ErrorVariant2\]);
macro_rules! error_compose {
    ($errorset_name:ident[$($error_name: ident),*]) => {
        #[derive(Error, Debug)]
        pub enum $errorset_name {
            $(
            #[error(transparent)]
            $error_name(#[from] $error_name),
            )*
            /// Generic error variant - display, backtrace passed onto source anyhow::Error
            /// Useful for chucking in errors from random sources. See usage of anyhow! macro.
            #[error(transparent)]
            ZFhOt01Rdb0Error(#[from] anyhow::Error),
        }
    };
}

// Manual declaration example
#[derive(Error, Debug)]
pub enum CreateAccountError {
    #[error("Failed to create account in storage layer")]
    StorageBackendError(#[from] StorageBackendError),
    #[error(transparent)]
    ZFhOt01Rdb0Error(#[from] anyhow::Error),
}
// Composed
error_compose!(GetAccountIdError[StorageBackendError, AccountNotFound]);
error_compose!(GetAccountDetailsError[StorageBackendError, AccountNotFound, DecodeError]);
error_compose!(ListAccountsError[StorageBackendError, DecodeError]);

error_compose!(AddRecipientError[StorageBackendError, AccountNotFound]);
// Expected to fail similarly
pub type RemoveRecipientError = AddRecipientError;
error_compose!(ListRecipientKeysError[StorageBackendError, AccountNotFound]);

error_compose!(PersistForwardMessageError[StorageBackendError, AccountNotFound]);
error_compose!(RetrievePendingMessageCountError[StorageBackendError, AccountNotFound]);
error_compose!(RetrievePendingMessagesError[StorageBackendError, AccountNotFound]);
