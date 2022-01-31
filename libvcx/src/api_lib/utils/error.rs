use std::cell::RefCell;
use std::error::Error;
use std::ffi::CString;
use std::ptr;

use failure::Fail;
use libc::c_char;
use aries_vcx::agency_client::error::AgencyClientError;

use crate::api_lib::utils::cstring::CStringUtils;
use aries_vcx::error::{VcxError, VcxErrorKind, VcxResult};
use aries_vcx::utils::error;

thread_local! {
    pub static CURRENT_ERROR_C_JSON: RefCell<Option<CString>> = RefCell::new(None);
}

pub fn reset_current_error() {
    CURRENT_ERROR_C_JSON.with(|error| {
        error.replace(None);
    })
}

pub fn set_current_error_agency(err: &AgencyClientError) {
    CURRENT_ERROR_C_JSON.try_with(|error| {
        let error_json = json!({
            "error": err.kind().to_string(),
            "message": err.to_string(),
            "cause": <dyn Fail>::find_root_cause(err).to_string(),
            "backtrace": err.backtrace().map(|bt| bt.to_string())
        }).to_string();
        error.replace(Some(CStringUtils::string_to_cstring(error_json)));
    })
        .map_err(|err| error!("Thread local variable access failed with: {:?}", err)).ok();
}

pub fn set_current_error(err: &VcxError) {
    CURRENT_ERROR_C_JSON.try_with(|error| {
        let error_json = json!({
            "error": err.kind().to_string(),
            "message": err.to_string(),
            "cause": <dyn Fail>::find_root_cause(err).to_string(),
            "backtrace": err.backtrace().map(|bt| bt.to_string())
        }).to_string();
        error.replace(Some(CStringUtils::string_to_cstring(error_json)));
    })
        .map_err(|err| error!("Thread local variable access failed with: {:?}", err)).ok();
}

pub fn set_current_error_2(err: &Error) {
    CURRENT_ERROR_C_JSON.try_with(|error| {
        let error_json = json!({
            "message": err.to_string()
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
