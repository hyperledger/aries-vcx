use std::ptr;

use error::prelude::*;
use utils::error;
use indy_sys::CommandHandle;
use libc::c_char;
use filters;
use utils::threadpool::spawn;
use utils::cstring::CStringUtils;


/// Filters proof requests based on name selected by verifier when creating the request.
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// requests: Serialized array of proof requests JSONs.
///
/// # Example 
///
/// match_name: Name of the request to match.
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_filter_proof_requests_by_name(command_handle: CommandHandle,
                                                requests: *const c_char,
                                                match_name: *const c_char,
                                                cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, requests: *const c_char)>) -> u32 {
    info!("vcx_filter_proof_requests_by_name >>>");

    check_useful_c_str!(requests, VcxErrorKind::InvalidOption);
    check_useful_c_str!(match_name, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_filter_proof_requests_by_name(command_handle: {}, requests: {}, match_name: {})",
           command_handle, requests, match_name);

    spawn(move || {
        match filters::filter_proof_requests_by_name(&requests, &match_name) {
            Ok(x) => {
                trace!("vcx_filter_proof_requests_by_name_cb(command_handle: {}, requests: {}, rc: {}, requests: {})",
                       command_handle, requests, error::SUCCESS.message, x);
                let x = CStringUtils::string_to_cstring(x);
                cb(command_handle,error::SUCCESS.code_num, x.as_ptr());
            }
            Err(err) => {
                error!("vcx_filter_proof_requests_by_name_cb(command_handle: {}, rc: {}, msg: {})",
                       command_handle, error::SUCCESS.message, err);
                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_filter_proof_requests_by_name() {}

}
