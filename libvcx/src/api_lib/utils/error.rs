use std::cell::RefCell;
use std::ffi::CString;
use std::fmt;
use std::ptr;
use std::sync;

use failure::{Backtrace, Context, Fail};
use libc::c_char;

use crate::api_lib::utils::cstring::CStringUtils;
use crate::error::{VcxError, VcxErrorKind};
use aries_vcx::utils;
use aries_vcx::utils::error;

impl From<VcxError> for u32 {
    fn from(code: VcxError) -> u32 {
        set_current_error(&code);
        code.kind().into()
    }
}

impl From<VcxErrorKind> for u32 {
    fn from(code: VcxErrorKind) -> u32 {
        match code {
            VcxErrorKind::InvalidState => error::INVALID_STATE.code_num,
            VcxErrorKind::InvalidConfiguration => error::INVALID_CONFIGURATION.code_num,
            VcxErrorKind::InvalidHandle => error::INVALID_OBJ_HANDLE.code_num,
            VcxErrorKind::InvalidJson => error::INVALID_JSON.code_num,
            VcxErrorKind::InvalidOption => error::INVALID_OPTION.code_num,
            VcxErrorKind::InvalidMessagePack => error::INVALID_MSGPACK.code_num,
            VcxErrorKind::ObjectCacheError => error::OBJECT_CACHE_ERROR.code_num,
            VcxErrorKind::NoPaymentInformation => error::NO_PAYMENT_INFORMATION.code_num,
            VcxErrorKind::NotReady => error::NOT_READY.code_num,
            VcxErrorKind::InvalidRevocationDetails => error::INVALID_REVOCATION_DETAILS.code_num,
            VcxErrorKind::GeneralConnectionError => error::CONNECTION_ERROR.code_num,
            VcxErrorKind::IOError => error::IOERROR.code_num,
            VcxErrorKind::LibindyInvalidStructure => error::LIBINDY_INVALID_STRUCTURE.code_num,
            VcxErrorKind::TimeoutLibindy => error::TIMEOUT_LIBINDY_ERROR.code_num,
            VcxErrorKind::InvalidLibindyParam => error::INVALID_LIBINDY_PARAM.code_num,
            VcxErrorKind::AlreadyInitialized => error::ALREADY_INITIALIZED.code_num,
            VcxErrorKind::CreateConnection => error::CREATE_CONNECTION_ERROR.code_num,
            VcxErrorKind::InvalidConnectionHandle => error::INVALID_CONNECTION_HANDLE.code_num,
            VcxErrorKind::InvalidInviteDetail => error::INVALID_INVITE_DETAILS.code_num,
            VcxErrorKind::InvalidRedirectDetail => error::INVALID_REDIRECT_DETAILS.code_num,
            VcxErrorKind::DeleteConnection => error::CANNOT_DELETE_CONNECTION.code_num,
            VcxErrorKind::CreateCredDef => error::CREATE_CREDENTIAL_DEF_ERR.code_num,
            VcxErrorKind::CredDefAlreadyCreated => error::CREDENTIAL_DEF_ALREADY_CREATED.code_num,
            VcxErrorKind::InvalidCredDefHandle => error::INVALID_CREDENTIAL_DEF_HANDLE.code_num,
            VcxErrorKind::InvalidRevocationEntry => error::INVALID_REV_ENTRY.code_num,
            VcxErrorKind::CreateRevRegDef => error::INVALID_REV_REG_DEF_CREATION.code_num,
            VcxErrorKind::InvalidCredentialHandle => error::INVALID_CREDENTIAL_HANDLE.code_num,
            VcxErrorKind::CreateCredentialRequest => error::CREATE_CREDENTIAL_REQUEST_ERROR.code_num,
            VcxErrorKind::InvalidIssuerCredentialHandle => error::INVALID_ISSUER_CREDENTIAL_HANDLE.code_num,
            VcxErrorKind::InvalidCredentialRequest => error::INVALID_CREDENTIAL_REQUEST.code_num,
            VcxErrorKind::InvalidCredential => error::INVALID_CREDENTIAL_JSON.code_num,
            VcxErrorKind::InsufficientTokenAmount => error::INSUFFICIENT_TOKEN_AMOUNT.code_num,
            VcxErrorKind::InvalidProofHandle => error::INVALID_PROOF_HANDLE.code_num,
            VcxErrorKind::InvalidDisclosedProofHandle => error::INVALID_DISCLOSED_PROOF_HANDLE.code_num,
            VcxErrorKind::InvalidProof => error::INVALID_PROOF.code_num,
            VcxErrorKind::InvalidSchema => error::INVALID_SCHEMA.code_num,
            VcxErrorKind::InvalidProofCredentialData => error::INVALID_PROOF_CREDENTIAL_DATA.code_num,
            VcxErrorKind::CreateProof => error::CREATE_PROOF_ERROR.code_num,
            VcxErrorKind::InvalidRevocationTimestamp => error::INVALID_REVOCATION_TIMESTAMP.code_num,
            VcxErrorKind::CreateSchema => error::INVALID_SCHEMA_CREATION.code_num,
            VcxErrorKind::InvalidSchemaHandle => error::INVALID_SCHEMA_HANDLE.code_num,
            VcxErrorKind::InvalidSchemaSeqNo => error::INVALID_SCHEMA_SEQ_NO.code_num,
            VcxErrorKind::DuplicationSchema => error::DUPLICATE_SCHEMA.code_num,
            VcxErrorKind::UnknownSchemaRejection => error::UNKNOWN_SCHEMA_REJECTION.code_num,
            VcxErrorKind::WalletCreate => error::INVALID_WALLET_CREATION.code_num,
            VcxErrorKind::MissingWalletName => error::MISSING_WALLET_NAME.code_num,
            VcxErrorKind::WalletAccessFailed => error::WALLET_ACCESS_FAILED.code_num,
            VcxErrorKind::InvalidWalletHandle => error::INVALID_WALLET_HANDLE.code_num,
            VcxErrorKind::DuplicationWallet => error::WALLET_ALREADY_EXISTS.code_num,
            VcxErrorKind::WalletNotFound => error::WALLET_NOT_FOUND.code_num,
            VcxErrorKind::WalletRecordNotFound => error::WALLET_RECORD_NOT_FOUND.code_num,
            VcxErrorKind::PoolLedgerConnect => error::POOL_LEDGER_CONNECT.code_num,
            VcxErrorKind::InvalidGenesisTxnPath => error::INVALID_GENESIS_TXN_PATH.code_num,
            VcxErrorKind::CreatePoolConfig => error::CREATE_POOL_CONFIG.code_num,
            VcxErrorKind::DuplicationWalletRecord => error::DUPLICATE_WALLET_RECORD.code_num,
            VcxErrorKind::WalletAlreadyOpen => error::WALLET_ALREADY_OPEN.code_num,
            VcxErrorKind::DuplicationMasterSecret => error::DUPLICATE_MASTER_SECRET.code_num,
            VcxErrorKind::DuplicationDid => error::DID_ALREADY_EXISTS_IN_WALLET.code_num,
            VcxErrorKind::InvalidLedgerResponse => error::INVALID_LEDGER_RESPONSE.code_num,
            VcxErrorKind::InvalidAttributesStructure => error::INVALID_ATTRIBUTES_STRUCTURE.code_num,
            VcxErrorKind::InvalidPaymentAddress => error::INVALID_PAYMENT_ADDRESS.code_num,
            VcxErrorKind::NoEndpoint => error::NO_ENDPOINT.code_num,
            VcxErrorKind::InvalidProofRequest => error::INVALID_PROOF_REQUEST.code_num,
            VcxErrorKind::NoPoolOpen => error::NO_POOL_OPEN.code_num,
            VcxErrorKind::PostMessageFailed => error::POST_MSG_FAILURE.code_num,
            VcxErrorKind::LoggingError => error::LOGGING_ERROR.code_num,
            VcxErrorKind::EncodeError => error::BIG_NUMBER_ERROR.code_num,
            VcxErrorKind::UnknownError => error::UNKNOWN_ERROR.code_num,
            VcxErrorKind::InvalidDid => error::INVALID_DID.code_num,
            VcxErrorKind::InvalidVerkey => error::INVALID_VERKEY.code_num,
            VcxErrorKind::InvalidNonce => error::INVALID_NONCE.code_num,
            VcxErrorKind::InvalidUrl => error::INVALID_URL.code_num,
            VcxErrorKind::MissingWalletKey => error::MISSING_WALLET_KEY.code_num,
            VcxErrorKind::MissingPaymentMethod => error::MISSING_PAYMENT_METHOD.code_num,
            VcxErrorKind::SerializationError => error::SERIALIZATION_ERROR.code_num,
            VcxErrorKind::NotBase58 => error::NOT_BASE58.code_num,
            VcxErrorKind::InvalidHttpResponse => error::INVALID_HTTP_RESPONSE.code_num,
            VcxErrorKind::InvalidMessages => error::INVALID_MESSAGES.code_num,
            VcxErrorKind::MissingExportedWalletPath => error::MISSING_EXPORTED_WALLET_PATH.code_num,
            VcxErrorKind::MissingBackupKey => error::MISSING_BACKUP_KEY.code_num,
            VcxErrorKind::UnknownLibndyError => error::UNKNOWN_LIBINDY_ERROR.code_num,
            VcxErrorKind::ActionNotSupported => error::ACTION_NOT_SUPPORTED.code_num,
            VcxErrorKind::Common(num) => num,
            VcxErrorKind::LibndyError(num) => num,
            VcxErrorKind::NoAgentInformation => error::NO_AGENT_INFO.code_num,
            VcxErrorKind::RevRegDefNotFound => error::REV_REG_DEF_NOT_FOUND.code_num,
            VcxErrorKind::RevDeltaNotFound => error::REV_DELTA_NOT_FOUND.code_num,
            VcxErrorKind::PoisonedLock => error::POISONED_LOCK.code_num,
            VcxErrorKind::CreatePublicAgent => error::CREATE_PUBLIC_AGENT.code_num
        }
    }
}

impl From<u32> for VcxErrorKind {
    fn from(code: u32) -> VcxErrorKind {
        match code {
            _ if { error::INVALID_STATE.code_num == code } => VcxErrorKind::InvalidState,
            _ if { error::INVALID_CONFIGURATION.code_num == code } => VcxErrorKind::InvalidConfiguration,
            _ if { error::INVALID_OBJ_HANDLE.code_num == code } => VcxErrorKind::InvalidHandle,
            _ if { error::INVALID_JSON.code_num == code } => VcxErrorKind::InvalidJson,
            _ if { error::INVALID_OPTION.code_num == code } => VcxErrorKind::InvalidOption,
            _ if { error::INVALID_MSGPACK.code_num == code } => VcxErrorKind::InvalidMessagePack,
            _ if { error::OBJECT_CACHE_ERROR.code_num == code } => VcxErrorKind::ObjectCacheError,
            _ if { error::NO_PAYMENT_INFORMATION.code_num == code } => VcxErrorKind::NoPaymentInformation,
            _ if { error::NOT_READY.code_num == code } => VcxErrorKind::NotReady,
            _ if { error::INVALID_REVOCATION_DETAILS.code_num == code } => VcxErrorKind::InvalidRevocationDetails,
            _ if { error::CONNECTION_ERROR.code_num == code } => VcxErrorKind::GeneralConnectionError,
            _ if { error::IOERROR.code_num == code } => VcxErrorKind::IOError,
            _ if { error::LIBINDY_INVALID_STRUCTURE.code_num == code } => VcxErrorKind::LibindyInvalidStructure,
            _ if { error::TIMEOUT_LIBINDY_ERROR.code_num == code } => VcxErrorKind::TimeoutLibindy,
            _ if { error::INVALID_LIBINDY_PARAM.code_num == code } => VcxErrorKind::InvalidLibindyParam,
            _ if { error::ALREADY_INITIALIZED.code_num == code } => VcxErrorKind::AlreadyInitialized,
            _ if { error::CREATE_CONNECTION_ERROR.code_num == code } => VcxErrorKind::CreateConnection,
            _ if { error::INVALID_CONNECTION_HANDLE.code_num == code } => VcxErrorKind::InvalidConnectionHandle,
            _ if { error::INVALID_INVITE_DETAILS.code_num == code } => VcxErrorKind::InvalidInviteDetail,
            _ if { error::INVALID_REDIRECT_DETAILS.code_num == code } => VcxErrorKind::InvalidRedirectDetail,
            _ if { error::CANNOT_DELETE_CONNECTION.code_num == code } => VcxErrorKind::DeleteConnection,
            _ if { error::CREATE_CREDENTIAL_DEF_ERR.code_num == code } => VcxErrorKind::CreateCredDef,
            _ if { error::CREDENTIAL_DEF_ALREADY_CREATED.code_num == code } => VcxErrorKind::CredDefAlreadyCreated,
            _ if { error::INVALID_CREDENTIAL_DEF_HANDLE.code_num == code } => VcxErrorKind::InvalidCredDefHandle,
            _ if { error::INVALID_REV_ENTRY.code_num == code } => VcxErrorKind::InvalidRevocationEntry,
            _ if { error::INVALID_REV_REG_DEF_CREATION.code_num == code } => VcxErrorKind::CreateRevRegDef,
            _ if { error::INVALID_CREDENTIAL_HANDLE.code_num == code } => VcxErrorKind::InvalidCredentialHandle,
            _ if { error::CREATE_CREDENTIAL_REQUEST_ERROR.code_num == code } => VcxErrorKind::CreateCredentialRequest,
            _ if { error::INVALID_ISSUER_CREDENTIAL_HANDLE.code_num == code } => VcxErrorKind::InvalidIssuerCredentialHandle,
            _ if { error::INVALID_CREDENTIAL_REQUEST.code_num == code } => VcxErrorKind::InvalidCredentialRequest,
            _ if { error::INVALID_CREDENTIAL_JSON.code_num == code } => VcxErrorKind::InvalidCredential,
            _ if { error::INSUFFICIENT_TOKEN_AMOUNT.code_num == code } => VcxErrorKind::InsufficientTokenAmount,
            _ if { error::INVALID_PROOF_HANDLE.code_num == code } => VcxErrorKind::InvalidProofHandle,
            _ if { error::INVALID_DISCLOSED_PROOF_HANDLE.code_num == code } => VcxErrorKind::InvalidDisclosedProofHandle,
            _ if { error::INVALID_PROOF.code_num == code } => VcxErrorKind::InvalidProof,
            _ if { error::INVALID_SCHEMA.code_num == code } => VcxErrorKind::InvalidSchema,
            _ if { error::INVALID_PROOF_CREDENTIAL_DATA.code_num == code } => VcxErrorKind::InvalidProofCredentialData,
            _ if { error::CREATE_PROOF_ERROR.code_num == code } => VcxErrorKind::CreateProof,
            _ if { error::INVALID_REVOCATION_TIMESTAMP.code_num == code } => VcxErrorKind::InvalidRevocationTimestamp,
            _ if { error::INVALID_SCHEMA_CREATION.code_num == code } => VcxErrorKind::CreateSchema,
            _ if { error::INVALID_SCHEMA_HANDLE.code_num == code } => VcxErrorKind::InvalidSchemaHandle,
            _ if { error::INVALID_SCHEMA_SEQ_NO.code_num == code } => VcxErrorKind::InvalidSchemaSeqNo,
            _ if { error::DUPLICATE_SCHEMA.code_num == code } => VcxErrorKind::DuplicationSchema,
            _ if { error::UNKNOWN_SCHEMA_REJECTION.code_num == code } => VcxErrorKind::UnknownSchemaRejection,
            _ if { error::INVALID_WALLET_CREATION.code_num == code } => VcxErrorKind::WalletCreate,
            _ if { error::MISSING_WALLET_NAME.code_num == code } => VcxErrorKind::MissingWalletName,
            _ if { error::WALLET_ACCESS_FAILED.code_num == code } => VcxErrorKind::WalletAccessFailed,
            _ if { error::INVALID_WALLET_HANDLE.code_num == code } => VcxErrorKind::InvalidWalletHandle,
            _ if { error::WALLET_ALREADY_EXISTS.code_num == code } => VcxErrorKind::DuplicationWallet,
            _ if { error::WALLET_NOT_FOUND.code_num == code } => VcxErrorKind::WalletNotFound,
            _ if { error::WALLET_RECORD_NOT_FOUND.code_num == code } => VcxErrorKind::WalletRecordNotFound,
            _ if { error::POOL_LEDGER_CONNECT.code_num == code } => VcxErrorKind::PoolLedgerConnect,
            _ if { error::INVALID_GENESIS_TXN_PATH.code_num == code } => VcxErrorKind::InvalidGenesisTxnPath,
            _ if { error::CREATE_POOL_CONFIG.code_num == code } => VcxErrorKind::CreatePoolConfig,
            _ if { error::DUPLICATE_WALLET_RECORD.code_num == code } => VcxErrorKind::DuplicationWalletRecord,
            _ if { error::WALLET_ALREADY_OPEN.code_num == code } => VcxErrorKind::WalletAlreadyOpen,
            _ if { error::DUPLICATE_MASTER_SECRET.code_num == code } => VcxErrorKind::DuplicationMasterSecret,
            _ if { error::DID_ALREADY_EXISTS_IN_WALLET.code_num == code } => VcxErrorKind::DuplicationDid,
            _ if { error::INVALID_LEDGER_RESPONSE.code_num == code } => VcxErrorKind::InvalidLedgerResponse,
            _ if { error::INVALID_ATTRIBUTES_STRUCTURE.code_num == code } => VcxErrorKind::InvalidAttributesStructure,
            _ if { error::INVALID_PAYMENT_ADDRESS.code_num == code } => VcxErrorKind::InvalidPaymentAddress,
            _ if { error::NO_ENDPOINT.code_num == code } => VcxErrorKind::NoEndpoint,
            _ if { error::INVALID_PROOF_REQUEST.code_num == code } => VcxErrorKind::InvalidProofRequest,
            _ if { error::NO_POOL_OPEN.code_num == code } => VcxErrorKind::NoPoolOpen,
            _ if { error::POST_MSG_FAILURE.code_num == code } => VcxErrorKind::PostMessageFailed,
            _ if { error::LOGGING_ERROR.code_num == code } => VcxErrorKind::LoggingError,
            _ if { error::BIG_NUMBER_ERROR.code_num == code } => VcxErrorKind::EncodeError,
            _ if { error::UNKNOWN_ERROR.code_num == code } => VcxErrorKind::UnknownError,
            _ if { error::INVALID_DID.code_num == code } => VcxErrorKind::InvalidDid,
            _ if { error::INVALID_VERKEY.code_num == code } => VcxErrorKind::InvalidVerkey,
            _ if { error::INVALID_NONCE.code_num == code } => VcxErrorKind::InvalidNonce,
            _ if { error::INVALID_URL.code_num == code } => VcxErrorKind::InvalidUrl,
            _ if { error::MISSING_WALLET_KEY.code_num == code } => VcxErrorKind::MissingWalletKey,
            _ if { error::MISSING_PAYMENT_METHOD.code_num == code } => VcxErrorKind::MissingPaymentMethod,
            _ if { error::SERIALIZATION_ERROR.code_num == code } => VcxErrorKind::SerializationError,
            _ if { error::NOT_BASE58.code_num == code } => VcxErrorKind::NotBase58,
            _ if { error::INVALID_HTTP_RESPONSE.code_num == code } => VcxErrorKind::InvalidHttpResponse,
            _ if { error::INVALID_MESSAGES.code_num == code } => VcxErrorKind::InvalidMessages,
            _ if { error::MISSING_EXPORTED_WALLET_PATH.code_num == code } => VcxErrorKind::MissingExportedWalletPath,
            _ if { error::MISSING_BACKUP_KEY.code_num == code } => VcxErrorKind::MissingBackupKey,
            _ if { error::UNKNOWN_LIBINDY_ERROR.code_num == code } => VcxErrorKind::UnknownLibndyError,
            _ if { error::ACTION_NOT_SUPPORTED.code_num == code } => VcxErrorKind::ActionNotSupported,
            _ if { error::NO_AGENT_INFO.code_num == code } => VcxErrorKind::NoAgentInformation,
            _ if { error::REV_REG_DEF_NOT_FOUND.code_num == code } => VcxErrorKind::RevRegDefNotFound,
            _ if { error::REV_DELTA_NOT_FOUND.code_num == code } => VcxErrorKind::RevDeltaNotFound,
            _ if { error::CREATE_PUBLIC_AGENT.code_num == code } => VcxErrorKind::CreatePublicAgent,
            _ => VcxErrorKind::UnknownError,
        }
    }
}

thread_local! {
    pub static CURRENT_ERROR_C_JSON: RefCell<Option<CString>> = RefCell::new(None);
}

pub fn reset_current_error() {
    CURRENT_ERROR_C_JSON.with(|error| {
        error.replace(None);
    })
}

pub fn set_current_error(err: &VcxError) {
    CURRENT_ERROR_C_JSON.try_with(|error| {
        let error_json = json!({
            "error": err.kind().to_string(),
            "message": err.to_string(),
            "cause": Fail::find_root_cause(err).to_string(),
            "backtrace": err.backtrace().map(|bt| bt.to_string())
        }).to_string();
        error.replace(Some(CStringUtils::string_to_cstring(error_json)));
    })
        .map_err(|err| error!("Thread local variable access failed with: {:?}", err)).ok();
}

pub fn get_current_error_c_json() -> *const c_char {
    let mut value = ptr::null();

    CURRENT_ERROR_C_JSON.try_with(|err|
        err.borrow().as_ref().map(|err| value = err.as_ptr())
    )
        .map_err(|err| error!("Thread local variable access failed with: {:?}", err)).ok();

    value
}
