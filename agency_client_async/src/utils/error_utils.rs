use std::collections::HashMap;
use std::ffi::CString;
use std::fmt;

pub static SUCCESS: Error = Error { code_num: 0, message: "Success" };
pub static UNKNOWN_ERROR: Error = Error { code_num: 1001, message: "Unknown Error" };
pub static INVALID_CONFIGURATION: Error = Error { code_num: 1004, message: "Invalid Configuration" };
pub static NOT_READY: Error = Error { code_num: 1005, message: "Object not ready for specified action" };
pub static INVALID_OPTION: Error = Error { code_num: 1007, message: "Invalid Option" };
pub static INVALID_DID: Error = Error { code_num: 1008, message: "Invalid DID" };
pub static INVALID_VERKEY: Error = Error { code_num: 1009, message: "Invalid VERKEY" };
pub static POST_MSG_FAILURE: Error = Error { code_num: 1010, message: "Message failed in post" };
pub static INVALID_URL: Error = Error { code_num: 1013, message: "Invalid URL" };
pub static NOT_BASE58: Error = Error { code_num: 1014, message: "Value needs to be base58" };
pub static INVALID_JSON: Error = Error { code_num: 1016, message: "Invalid JSON string" };
pub static CREATE_POOL_CONFIG: Error = Error { code_num: 1026, message: "Formatting for Pool Config are incorrect." };
pub static INVALID_HTTP_RESPONSE: Error = Error { code_num: 1033, message: "Invalid HTTP response." };
pub static UNKNOWN_LIBINDY_ERROR: Error = Error { code_num: 1035, message: "Unknown libindy error" };
pub static TIMEOUT_LIBINDY_ERROR: Error = Error { code_num: 1038, message: "Waiting for callback timed out" };
pub static CREDENTIAL_DEF_ALREADY_CREATED: Error = Error { code_num: 1039, message: "Can't create, Credential Def already exists in wallet" };
pub static INVALID_OBJ_HANDLE: Error = Error { code_num: 1048, message: "Obj was not found with handle" };
pub static SERIALIZATION_ERROR: Error = Error { code_num: 1050, message: "Unable to serialize" };
pub static WALLET_ALREADY_EXISTS: Error = Error { code_num: 1051, message: "Indy wallet already exists" };
pub static WALLET_ALREADY_OPEN: Error = Error { code_num: 1052, message: "Indy wallet already open" };
pub static INVALID_WALLET_HANDLE: Error = Error { code_num: 1057, message: "Invalid Wallet or Search Handle" };
pub static CANNOT_DELETE_CONNECTION: Error = Error { code_num: 1060, message: "Cannot Delete Connection. Check status of connection is appropriate to be deleted from agency." };
pub static INSUFFICIENT_TOKEN_AMOUNT: Error = Error { code_num: 1064, message: "Insufficient amount of tokens to process request" };
pub static INVALID_LIBINDY_PARAM: Error = Error { code_num: 1067, message: "Parameter passed to libindy was invalid" };
pub static MISSING_WALLET_KEY: Error = Error { code_num: 1069, message: "Configuration is missing wallet key" };
pub static DUPLICATE_WALLET_RECORD: Error = Error { code_num: 1072, message: "Record already exists in the wallet" };
pub static WALLET_RECORD_NOT_FOUND: Error = Error { code_num: 1073, message: "Wallet record not found" };
pub static IOERROR: Error = Error { code_num: 1074, message: "IO Error, possibly creating a backup wallet" };
pub static WALLET_NOT_FOUND: Error = Error { code_num: 1079, message: "Wallet Not Found" };
pub static LIBINDY_INVALID_STRUCTURE: Error = Error { code_num: 1080, message: "Object (json, config, key, credential and etc...) passed to libindy has invalid structure" };
pub static INVALID_STATE: Error = Error { code_num: 1081, message: "Object is in invalid state for requested operation" };
pub static INVALID_MSGPACK: Error = Error { code_num: 1019, message: "Invalid MessagePack" };
pub static DUPLICATE_MASTER_SECRET: Error = Error { code_num: 1084, message: "Attempted to add a Master Secret that already existed in wallet" };
pub static DID_ALREADY_EXISTS_IN_WALLET: Error = Error { code_num: 1083, message: "Attempted to add a DID to wallet when that DID already exists in wallet" };
pub static CREATE_AGENT: Error = Error { code_num: 2000, message: "Failed to create agency client" };

lazy_static! {
    static ref ERROR_C_MESSAGES: HashMap<u32, CString> = {
       let mut m = HashMap::new();
        insert_c_message(&mut m, &UNKNOWN_ERROR);
        insert_c_message(&mut m, &INVALID_CONFIGURATION);
        insert_c_message(&mut m, &NOT_READY);
        insert_c_message(&mut m, &INVALID_OPTION);
        insert_c_message(&mut m, &INVALID_DID);
        insert_c_message(&mut m, &INVALID_VERKEY);
        insert_c_message(&mut m, &POST_MSG_FAILURE);
        insert_c_message(&mut m, &INVALID_URL);
        insert_c_message(&mut m, &NOT_BASE58);
        insert_c_message(&mut m, &INVALID_JSON);
        insert_c_message(&mut m, &INVALID_MSGPACK);
        insert_c_message(&mut m, &CREATE_POOL_CONFIG);
        insert_c_message(&mut m, &INVALID_HTTP_RESPONSE);
        insert_c_message(&mut m, &UNKNOWN_LIBINDY_ERROR);
        insert_c_message(&mut m, &TIMEOUT_LIBINDY_ERROR);
        insert_c_message(&mut m, &CREDENTIAL_DEF_ALREADY_CREATED);
        insert_c_message(&mut m, &INVALID_OBJ_HANDLE);
        insert_c_message(&mut m, &SERIALIZATION_ERROR);
        insert_c_message(&mut m, &WALLET_ALREADY_EXISTS);
        insert_c_message(&mut m, &WALLET_ALREADY_OPEN);
        insert_c_message(&mut m, &INVALID_WALLET_HANDLE);
        insert_c_message(&mut m, &CANNOT_DELETE_CONNECTION);
        insert_c_message(&mut m, &INSUFFICIENT_TOKEN_AMOUNT);
        insert_c_message(&mut m, &INVALID_LIBINDY_PARAM);
        insert_c_message(&mut m, &MISSING_WALLET_KEY);
        insert_c_message(&mut m, &DUPLICATE_WALLET_RECORD);
        insert_c_message(&mut m, &WALLET_RECORD_NOT_FOUND);
        insert_c_message(&mut m, &IOERROR);
        insert_c_message(&mut m, &WALLET_NOT_FOUND);
        insert_c_message(&mut m, &LIBINDY_INVALID_STRUCTURE);
        insert_c_message(&mut m, &INVALID_STATE);
        insert_c_message(&mut m, &DID_ALREADY_EXISTS_IN_WALLET);
        insert_c_message(&mut m, &DUPLICATE_MASTER_SECRET);

        m
    };
}

// ******* END *******

// Helper function for static defining of error messages. Does limited checking that it can.
fn insert_c_message(map: &mut HashMap<u32, CString>, error: &Error) {
    if map.contains_key(&error.code_num) {
        panic!("Error Code number was repeated which is not allowed! (likely a copy/paste error)")
    }
    map.insert(error.code_num, CString::new(error.message).unwrap());
}

#[derive(Clone, Copy)]
pub struct Error {
    pub code_num: u32,
    pub message: &'static str,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = error_message(&self.code_num);
        write!(f, "{}: (Error Num:{})", msg, &self.code_num)
    }
}

pub fn error_message(code_num: &u32) -> String {
    match ERROR_C_MESSAGES.get(code_num) {
        Some(msg) => msg.to_str().unwrap().to_string(),
        None => error_message(&UNKNOWN_ERROR.code_num),
    }
}
