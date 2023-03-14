use std::{
    ffi::{CStr, CString},
    str::Utf8Error,
};

use libc::c_char;

pub struct CStringUtils {}

impl CStringUtils {
    pub fn c_str_to_string(cstr: *const c_char) -> Result<Option<String>, Utf8Error> {
        if cstr.is_null() {
            return Ok(None);
        }

        unsafe {
            match CStr::from_ptr(cstr).to_str() {
                Ok(str) => Ok(Some(str.to_string())),
                Err(err) => Err(err),
            }
        }
    }
    pub fn c_str_to_str<'a>(cstr: *const c_char) -> Result<Option<&'a str>, Utf8Error> {
        if cstr.is_null() {
            return Ok(None);
        }

        unsafe {
            match CStr::from_ptr(cstr).to_str() {
                Ok(s) => Ok(Some(s)),
                Err(err) => Err(err),
            }
        }
    }
    pub fn string_to_cstring(s: String) -> CString {
        CString::new(s).expect("Unexpected error converting to CString")
    }
}

macro_rules! check_useful_c_str {
    ($x:ident, $e:expr) => {
        let $x = match CStringUtils::c_str_to_string($x) {
            Ok(Some(val)) => val,
            _ => {
                let err = LibvcxError::from_msg($e, "Invalid pointer has been passed");
                set_current_error_vcx(&err);
                return err.into();
            }
        };

        if $x.is_empty() {
            let err = LibvcxError::from_msg($e, "Empty string has been passed");
            set_current_error_vcx(&err);
            return err.into();
        }
    };
}

macro_rules! check_useful_opt_c_str {
    ($x:ident, $e:expr) => {
        let $x = match CStringUtils::c_str_to_string($x) {
            Ok(opt_val) => opt_val,
            Err(_) => {
                let err = LibvcxError::from_msg($e, "Invalid pointer has been passed");
                set_current_error_vcx(&err);
                return err.into();
            }
        };
    };
}

macro_rules! check_useful_c_byte_array {
    ($ptr:ident, $len:expr, $err1:expr, $err2:expr) => {
        if $ptr.is_null() {
            let err = LibvcxError::from_msg($err1, "Invalid pointer has been passed");
            set_current_error_vcx(&err);
            return err.into();
        }
        if $len <= 0 {
            let err = LibvcxError::from_msg($err2, "Array length must be greater than 0");
            set_current_error_vcx(&err);
            return err.into();
        }
        let $ptr = unsafe { std::slice::from_raw_parts($ptr, $len as usize) };
        let $ptr = $ptr.to_vec();
    };
}

//Returnable pointer is valid only before first vector modification
pub fn vec_to_pointer(v: &Vec<u8>) -> (*const u8, u32) {
    let len = v.len() as u32;
    (v.as_ptr() as *const u8, len)
}
