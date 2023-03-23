use std::cell::RefCell;
use std::error::Error;
use std::ffi::CString;
use std::ptr;

use libc::c_char;

use libvcx_core::errors::error::LibvcxError;

use crate::api_c::cutils::cstring::CStringUtils;

thread_local! {
    pub static CURRENT_ERROR_C_JSON: RefCell<Option<CString>> = RefCell::new(None);
}

pub fn reset_current_error() {
    CURRENT_ERROR_C_JSON.with(|error| {
        error.replace(None);
    })
}

pub fn set_current_error_vcx(err: &LibvcxError) {
    CURRENT_ERROR_C_JSON
        .try_with(|error| {
            let error_json = json!({
                "error": err.kind().to_string(),
                "message": err.to_string(),
                "cause": err.find_root_cause(),
                // TODO: Put back once https://github.com/rust-lang/rust/issues/99301 is stabilized
                // "backtrace": err.backtrace()
            })
            .to_string();
            error.replace(Some(CStringUtils::string_to_cstring(error_json)));
        })
        .map_err(|err| error!("Thread local variable access failed with: {:?}", err))
        .ok();
}

pub fn set_current_error(err: &dyn Error) {
    CURRENT_ERROR_C_JSON
        .try_with(|error| {
            let error_json = json!({
                "message": err.to_string()
            })
            .to_string();
            error.replace(Some(CStringUtils::string_to_cstring(error_json)));
        })
        .map_err(|err| error!("Thread local variable access failed with: {:?}", err))
        .ok();
}

pub fn get_current_error_c_json() -> *const c_char {
    let mut value = ptr::null();

    CURRENT_ERROR_C_JSON
        .try_with(|err| err.borrow().as_ref().map(|err| value = err.as_ptr()))
        .map_err(|err| error!("Thread local variable access failed with: {:?}", err))
        .ok();

    value
}
