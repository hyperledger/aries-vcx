use std::collections::HashMap;
use std::ffi::CString;
use std::fmt;
use aries_vcx::error::VcxErrorKind;

// **** DEFINE NEW ERRORS HERE ****
// STEP 1: create new public static instance of Error, assign it a new unused number and
// give it a human readable error message
// STEP 2: Add Error to the static MAP (used for getting messages to wrappers)
// STEP 3: create a test making sure that your message can be retrieved


#[derive(Copy, Clone, Eq, PartialEq, Debug, thiserror::Error)]
pub enum LibvcxErrorKind {
    // Common
    #[error("Object is in invalid state for requested operation")]
    InvalidState,
    #[error("Invalid Configuration")]
    InvalidConfiguration,
    #[error("Obj was not found with handle")]
    InvalidHandle,
    #[error("Invalid JSON string")]
    InvalidJson,
    #[error("Invalid Option")]
    InvalidOption,
    #[error("Invalid MessagePack")]
    InvalidMessagePack,
    #[error("Object cache error")]
    ObjectCacheError,
    #[error("Object not ready for specified action")]
    NotReady,
    #[error("IO Error, possibly creating a backup wallet")]
    IOError,
    #[error("Object (json, config, key, credential and etc...) passed to libindy has invalid structure")]
    LibindyInvalidStructure,
    #[error("Waiting for callback timed out")]
    TimeoutLibindy,
    #[error("Parameter passed to libindy was invalid")]
    InvalidLibindyParam,
    #[error("Library already initialized")]
    AlreadyInitialized,
    #[error("Action is not supported")]
    ActionNotSupported,
    #[error("Invalid input parameter")]
    InvalidInput,
    #[error("Unimplemented feature")]
    UnimplementedFeature,

    // Connection
    #[error("Could not create connection")]
    CreateConnection,
    #[error("Invalid Connection Handle")]
    InvalidConnectionHandle,
    #[error("Invalid invite details structure")]
    InvalidInviteDetail,
    #[error("Invalid redirect details structure")]
    InvalidRedirectDetail,
    #[error("Cannot Delete Connection. Check status of connection is appropriate to be deleted from agency.")]
    DeleteConnection,
    #[error("Error with Connection")]
    GeneralConnectionError,

    // Payment
    #[error("No payment information associated with object")]
    NoPaymentInformation,
    #[error("Insufficient amount of tokens to process request")]
    InsufficientTokenAmount,
    #[error("Invalid payment address")]
    InvalidPaymentAddress,

    // Credential Definition error
    #[error("Call to create Credential Definition failed")]
    CreateCredDef,
    #[error("Can't create, Credential Def already on ledger")]
    CredDefAlreadyCreated,
    #[error("Invalid Credential Definition handle")]
    InvalidCredDefHandle,
    #[error("No revocation delta found in storage for this revocation registry. Were any credentials locally revoked?")]
    RevDeltaNotFound,
    #[error("Failed to clean stored revocation delta")]
    RevDeltaFailedToClear,

    // Revocation
    #[error("Failed to create Revocation Registration Definition")]
    CreateRevRegDef,
    #[error("Invalid Revocation Details")]
    InvalidRevocationDetails,
    #[error("Unable to Update Revocation Delta On Ledger")]
    InvalidRevocationEntry,
    #[error("Invalid Credential Revocation timestamp")]
    InvalidRevocationTimestamp,
    #[error("No revocation definition found")]
    RevRegDefNotFound,

    // Credential
    #[error("Invalid credential handle")]
    InvalidCredentialHandle,
    #[error("could not create credential request")]
    CreateCredentialRequest,

    // Issuer Credential
    #[error("Invalid Credential Issuer Handle")]
    InvalidIssuerCredentialHandle,
    #[error("Invalid Credential Request")]
    InvalidCredentialRequest,
    #[error("Invalid credential json")]
    InvalidCredential,
    #[error("Attributes provided to Credential Offer are not correct, possibly malformed")]
    InvalidAttributesStructure,

    // Proof
    #[error("Invalid proof handle")]
    InvalidProofHandle,
    #[error("Obj was not found with handle")]
    InvalidDisclosedProofHandle,
    #[error("Proof had invalid format")]
    InvalidProof,
    #[error("Schema was invalid or corrupt")]
    InvalidSchema,
    #[error("The Proof received does not have valid credentials listed.")]
    InvalidProofCredentialData,
    #[error("Could not create proof")]
    CreateProof,
    #[error("Proof Request Passed into Libindy Call Was Invalid")]
    InvalidProofRequest,

    // Schema
    #[error("Could not create schema")]
    CreateSchema,
    #[error("Invalid Schema Handle")]
    InvalidSchemaHandle,
    #[error("No Schema for that schema sequence number")]
    InvalidSchemaSeqNo,
    #[error("Duplicate Schema: Ledger Already Contains Schema For Given DID, Version, and Name Combination")]
    DuplicationSchema,
    #[error("Unknown Rejection of Schema Creation, refer to libindy documentation")]
    UnknownSchemaRejection,

    // Public agent
    #[error("Could not create public agent")]
    CreatePublicAgent,

    // Out of Band
    #[error("Could not create out of band message.")]
    CreateOutOfBand,

    // Pool
    #[error("Invalid genesis transactions path.")]
    InvalidGenesisTxnPath,
    #[error("Formatting for Pool Config are incorrect.")]
    CreatePoolConfig,
    #[error("Connection to Pool Ledger.")]
    PoolLedgerConnect,
    #[error("Ledger rejected submitted request.")]
    InvalidLedgerResponse,
    #[error("No Pool open. Can't return handle.")]
    NoPoolOpen,
    #[error("Message failed in post")]
    PostMessageFailed,

    // Wallet
    #[error("Error Creating a wallet")]
    WalletCreate,
    #[error("Missing wallet name in config")]
    MissingWalletName,
    #[error("Missing exported wallet path in config")]
    MissingExportedWalletPath,
    #[error("Missing exported backup key in config")]
    MissingBackupKey,
    #[error("Attempt to open wallet with invalid credentials")]
    WalletAccessFailed,
    #[error("Invalid Wallet or Search Handle")]
    InvalidWalletHandle,
    #[error("Indy wallet already exists")]
    DuplicationWallet,
    #[error("Wallet record not found")]
    WalletRecordNotFound,
    #[error("Record already exists in the wallet")]
    DuplicationWalletRecord,
    #[error("Wallet not found")]
    WalletNotFound,
    #[error("Indy wallet already open")]
    WalletAlreadyOpen,
    #[error("Configuration is missing wallet key")]
    MissingWalletKey,
    #[error("Attempted to add a Master Secret that already existed in wallet")]
    DuplicationMasterSecret,
    #[error("Attempted to add a DID to wallet when that DID already exists in wallet")]
    DuplicationDid,

    // Logger
    #[error("Logging Error")]
    LoggingError,

    // Validation
    #[error("Could not encode string to a big integer.")]
    EncodeError,
    #[error("Unknown Error")]
    UnknownError,
    #[error("Invalid DID")]
    InvalidDid,
    #[error("Invalid VERKEY")]
    InvalidVerkey,
    #[error("Invalid NONCE")]
    InvalidNonce,
    #[error("Invalid URL")]
    InvalidUrl,
    #[error("Configuration is missing the Payment Method parameter")]
    MissingPaymentMethod,
    #[error("Unable to serialize")]
    SerializationError,
    #[error("Value needs to be base58")]
    NotBase58,
    #[error("Could not parse a value")]
    ParsingError,

    // A2A
    #[error("Invalid HTTP response.")]
    InvalidHttpResponse,
    #[error("No Endpoint set for Connection Object")]
    NoEndpoint,
    #[error("Error Retrieving messages from API")]
    InvalidMessages,
    #[error("Error creating agent in agency")]
    CreateAgent,

    #[error("Common error {}", 0)]
    Common(u32),
    #[error("Libndy error {}", 0)]
    LibndyError(u32),
    #[error("Unknown libindy error")]
    UnknownLibndyError,
    #[error("No Agent pairwise information")]
    NoAgentInformation,

    #[error("Invalid message format")]
    InvalidMessageFormat,

    #[error("Attempted to unlock poisoned lock")]
    PoisonedLock,
}

pub static INDY_WALLET_RECORD_NOT_FOUND: LibvcxError = LibvcxError {
    kind: LibvcxErrorKind::WalletRecordNotFound,
    code_num: 212,
};

pub static INDY_DUPLICATE_WALLET_RECORD: LibvcxError = LibvcxError {
    kind: LibvcxErrorKind::DuplicationWalletRecord,
    code_num: 213,
};

pub static UNKNOWN_ERROR: LibvcxError = LibvcxError {
    kind: LibvcxErrorKind::UnknownError,
    code_num: 1001,
};

// todo: this
pub static CONNECTION_ERROR: LibvcxError = LibvcxError {
    kind: LibvcxErrorKind::UnknownError,
    code_num: 1002,
};

pub static INVALID_CONNECTION_HANDLE: LibvcxError = LibvcxError {
    kind: LibvcxErrorKind::InvalidConnectionHandle,
    code_num: 1003,
};

pub static INVALID_CONFIGURATION: LibvcxError = LibvcxError {
    code_num: 1004,
};

pub static NOT_READY: LibvcxError = LibvcxError {
    code_num: 1005,
};

pub static NO_ENDPOINT: LibvcxError = LibvcxError {
    code_num: 1006,
};

pub static INVALID_OPTION: LibvcxError = LibvcxError {
    code_num: 1007,
};

pub static INVALID_DID: LibvcxError = LibvcxError {
    code_num: 1008,
};

pub static INVALID_VERKEY: LibvcxError = LibvcxError {
    code_num: 1009,
};

pub static POST_MSG_FAILURE: LibvcxError = LibvcxError {
    code_num: 1010,
};

pub static INVALID_NONCE: LibvcxError = LibvcxError {
    code_num: 1011,
};

pub static INVALID_KEY_DELEGATE: LibvcxError = LibvcxError {
    code_num: 1012,
};

pub static INVALID_URL: LibvcxError = LibvcxError {
    code_num: 1013,
};

pub static NOT_BASE58: LibvcxError = LibvcxError {
    code_num: 1014,
};

pub static INVALID_ISSUER_CREDENTIAL_HANDLE: LibvcxError = LibvcxError {
    code_num: 1015,
};

pub static INVALID_JSON: LibvcxError = LibvcxError {
    code_num: 1016,
};

pub static INVALID_PROOF_HANDLE: LibvcxError = LibvcxError {
    code_num: 1017,
};

pub static INVALID_CREDENTIAL_REQUEST: LibvcxError = LibvcxError {
    code_num: 1018,
};

pub static INVALID_MSGPACK: LibvcxError = LibvcxError {
    code_num: 1019,
};
//todo: code_num: 1020,

pub static INVALID_ATTRIBUTES_STRUCTURE: LibvcxError = LibvcxError {
    code_num: 1021,
};

pub static BIG_NUMBER_ERROR: LibvcxError = LibvcxError {
    code_num: 1022,
};

pub static INVALID_PROOF: LibvcxError = LibvcxError {
    code_num: 1023,
};

pub static INVALID_GENESIS_TXN_PATH: LibvcxError = LibvcxError {
    code_num: 1024,
};

pub static POOL_LEDGER_CONNECT: LibvcxError = LibvcxError {
    code_num: 1025,
};

pub static CREATE_POOL_CONFIG: LibvcxError = LibvcxError {
    code_num: 1026,
};

pub static INVALID_PROOF_CREDENTIAL_DATA: LibvcxError = LibvcxError {
    code_num: 1027,
};
pub static INDY_SUBMIT_REQUEST_ERR: LibvcxError = LibvcxError {
    code_num: 1028,
};
pub static BUILD_CREDENTIAL_DEF_REQ_ERR: LibvcxError = LibvcxError {
    code_num: 1029,
};
pub static NO_POOL_OPEN: LibvcxError = LibvcxError {
    code_num: 1030,
};
pub static INVALID_SCHEMA: LibvcxError = LibvcxError {
    code_num: 1031,
};
pub static FAILED_PROOF_COMPLIANCE: LibvcxError = LibvcxError {
    code_num: 1032,
};
pub static INVALID_HTTP_RESPONSE: LibvcxError = LibvcxError {
    code_num: 1033,
};
pub static CREATE_CREDENTIAL_DEF_ERR: LibvcxError = LibvcxError {
    code_num: 1034,
};
pub static UNKNOWN_LIBINDY_ERROR: LibvcxError = LibvcxError {
    code_num: 1035,
};
pub static INVALID_CREDENTIAL_DEF_JSON: LibvcxError = LibvcxError {
    code_num: 1036,
};
pub static INVALID_CREDENTIAL_DEF_HANDLE: LibvcxError = LibvcxError {
    code_num: 1037,
};
pub static TIMEOUT_LIBINDY_ERROR: LibvcxError = LibvcxError {
    code_num: 1038,
};
pub static CREDENTIAL_DEF_ALREADY_CREATED: LibvcxError = LibvcxError {
    code_num: 1039,
};
pub static INVALID_SCHEMA_SEQ_NO: LibvcxError = LibvcxError {
    code_num: 1040,
};
pub static INVALID_SCHEMA_CREATION: LibvcxError = LibvcxError {
    code_num: 1041,
};
pub static INVALID_SCHEMA_HANDLE: LibvcxError = LibvcxError {
    code_num: 1042,
};
pub static INVALID_MASTER_SECRET: LibvcxError = LibvcxError {
    code_num: 1043,
};
pub static ALREADY_INITIALIZED: LibvcxError = LibvcxError {
    code_num: 1044,
};
pub static INVALID_INVITE_DETAILS: LibvcxError = LibvcxError {
    code_num: 1045,
};
pub static INVALID_SELF_ATTESTED_VAL: LibvcxError = LibvcxError {
    code_num: 1046,
};
pub static INVALID_PREDICATE: LibvcxError = LibvcxError {
    code_num: 1047,
};
pub static INVALID_OBJ_HANDLE: LibvcxError = LibvcxError {
    code_num: 1048,
};
pub static INVALID_DISCLOSED_PROOF_HANDLE: LibvcxError = LibvcxError {
    code_num: 1049,
};
pub static SERIALIZATION_ERROR: LibvcxError = LibvcxError {
    code_num: 1050,
};
pub static WALLET_ALREADY_EXISTS: LibvcxError = LibvcxError {
    code_num: 1051,
};
pub static WALLET_ALREADY_OPEN: LibvcxError = LibvcxError {
    code_num: 1052,
};
pub static INVALID_CREDENTIAL_HANDLE: LibvcxError = LibvcxError {
    code_num: 1053,
};
pub static INVALID_CREDENTIAL_JSON: LibvcxError = LibvcxError {
    code_num: 1054,
};
pub static CREATE_CREDENTIAL_REQUEST_ERROR: LibvcxError = LibvcxError {
    code_num: 1055,
};
pub static CREATE_PROOF_ERROR: LibvcxError = LibvcxError {
    code_num: 1056,
};
pub static INVALID_WALLET_HANDLE: LibvcxError = LibvcxError {
    code_num: 1057,
};
pub static INVALID_WALLET_CREATION: LibvcxError = LibvcxError {
    code_num: 1058,
};
pub static INVALID_POOL_NAME: LibvcxError = LibvcxError {
    code_num: 1059,
};
pub static CANNOT_DELETE_CONNECTION: LibvcxError = LibvcxError {
    code_num: 1060,
};
pub static CREATE_CONNECTION_ERROR: LibvcxError = LibvcxError {
    code_num: 1061,
};
pub static INVALID_WALLET_SETUP: LibvcxError = LibvcxError {
    code_num: 1062,
};
pub static COMMON_ERROR: LibvcxError = LibvcxError {
    code_num: 1063,
};
pub static INSUFFICIENT_TOKEN_AMOUNT: LibvcxError = LibvcxError {
    code_num: 1064,
};
pub static UNKNOWN_TXN_TYPE: LibvcxError = LibvcxError {
    code_num: 1065,
};
pub static INVALID_PAYMENT_ADDRESS: LibvcxError = LibvcxError {
    code_num: 1066,
};
pub static INVALID_LIBINDY_PARAM: LibvcxError = LibvcxError {
    code_num: 1067,
};
pub static INVALID_PAYMENT: LibvcxError = LibvcxError {
    code_num: 1068,
};
pub static MISSING_WALLET_KEY: LibvcxError = LibvcxError {
    code_num: 1069,
};
pub static OBJECT_CACHE_ERROR: LibvcxError = LibvcxError {
    code_num: 1070,
};
pub static NO_PAYMENT_INFORMATION: LibvcxError = LibvcxError {
    code_num: 1071,
};
pub static DUPLICATE_WALLET_RECORD: LibvcxError = LibvcxError {
    code_num: 1072,
};
pub static WALLET_RECORD_NOT_FOUND: LibvcxError = LibvcxError {
    code_num: 1073,
};
pub static IOERROR: LibvcxError = LibvcxError {
    code_num: 1074,
};
pub static WALLET_ACCESS_FAILED: LibvcxError = LibvcxError {
    code_num: 1075,
};
pub static MISSING_WALLET_NAME: LibvcxError = LibvcxError {
    code_num: 1076,
};
pub static MISSING_EXPORTED_WALLET_PATH: LibvcxError = LibvcxError {
    code_num: 1077,
};
pub static MISSING_BACKUP_KEY: LibvcxError = LibvcxError {
    code_num: 1078,
};
pub static WALLET_NOT_FOUND: LibvcxError = LibvcxError {
    code_num: 1079,
};
pub static LIBINDY_INVALID_STRUCTURE: LibvcxError = LibvcxError {
    code_num: 1080,
};
pub static INVALID_STATE: LibvcxError = LibvcxError {
    code_num: 1081,
};
pub static INVALID_LEDGER_RESPONSE: LibvcxError = LibvcxError {
    code_num: 1082,
};
pub static DID_ALREADY_EXISTS_IN_WALLET: LibvcxError = LibvcxError {
    code_num: 1083,
};
pub static DUPLICATE_MASTER_SECRET: LibvcxError = LibvcxError {
    code_num: 1084,
};
pub static THREAD_ERROR: LibvcxError = LibvcxError {
    code_num: 1085,
};
pub static INVALID_PROOF_REQUEST: LibvcxError = LibvcxError {
    code_num: 1086,
};
pub static MISSING_PAYMENT_METHOD: LibvcxError = LibvcxError {
    code_num: 1087,
};
pub static DUPLICATE_SCHEMA: LibvcxError = LibvcxError {
    code_num: 1088,
};
pub static UKNOWN_LIBINDY_TRANSACTION_REJECTION: LibvcxError = LibvcxError {
    code_num: 1089,
};
pub static LOGGING_ERROR: LibvcxError = LibvcxError {
    code_num: 1090,
};
pub static INVALID_REVOCATION_DETAILS: LibvcxError = LibvcxError {
    code_num: 1091,
};
pub static INVALID_REV_ENTRY: LibvcxError = LibvcxError {
    code_num: 1092,
};
pub static INVALID_REVOCATION_TIMESTAMP: LibvcxError = LibvcxError {
    code_num: 1093,
};
pub static UNKNOWN_SCHEMA_REJECTION: LibvcxError = LibvcxError {
    code_num: 1094,
};
pub static INVALID_REV_REG_DEF_CREATION: LibvcxError = LibvcxError {
    code_num: 1095,
};
/* EC 1096 - 1099 are reserved for proprietary forks of libVCX */
pub static INVALID_ATTACHMENT_ENCODING: LibvcxError = LibvcxError {
    code_num: 1100,
};
pub static UNKNOWN_ATTACHMENT_ENCODING: LibvcxError = LibvcxError {
    code_num: 1101,
};
pub static UNKNOWN_MIME_TYPE: LibvcxError = LibvcxError {
    code_num: 1102,
};
pub static ACTION_NOT_SUPPORTED: LibvcxError = LibvcxError {
    code_num: 1103,
};
pub static INVALID_REDIRECT_DETAILS: LibvcxError = LibvcxError {
    code_num: 1104,
};
/* EC 1105 is reserved for proprietary forks of libVCX */
pub static NO_AGENT_INFO: LibvcxError = LibvcxError {
    code_num: 1106,
};
pub static REV_REG_DEF_NOT_FOUND: LibvcxError = LibvcxError {
    code_num: 1107,
};
pub static REV_DELTA_NOT_FOUND: LibvcxError = LibvcxError {
    code_num: 1108,
};
pub static POISONED_LOCK: LibvcxError = LibvcxError {
    code_num: 1109,
};
pub static CREATE_PUBLIC_AGENT: LibvcxError = LibvcxError {
    code_num: 1110,
};
// todo:
// code_num: 1111,
// };
pub static CREATE_OUT_OF_BAND: LibvcxError = LibvcxError {
    code_num: 1112,
};
pub static CREATE_AGENT: LibvcxError = LibvcxError {
    code_num: 1113,
};
pub static REV_DELTA_FAILED_TO_CLEAR: LibvcxError = LibvcxError {
    code_num: 1114,
};
pub static INVALID_INPUT: LibvcxError = LibvcxError {
    code_num: 1115,
};

pub static PARSING: LibvcxError = LibvcxError {
    code_num: 1116,
};

pub static UNIMPLEMENTED_FEATURE: LibvcxError = LibvcxError {
    code_num: 1117,
};


#[derive(Clone, Copy)]
pub struct LibvcxError {
    pub kind: LibvcxErrorKind,
    pub code_num: u32,
}

// todo: probably don't need this?
// impl LibvcxError {
//     pub fn get_code(&self) -> u32 {
//         self.code_num
//     }
// }

impl fmt::Display for LibvcxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // todo::
        // let msg = error_message(&self.code_num);
        // write!(f, "{}: (Error Num:{})", msg, &self.code_num)
        write!("foobar")
    }
}
