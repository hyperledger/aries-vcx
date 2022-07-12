use std::ptr;

use libc::c_char;

use aries_vcx::error::{VcxError, VcxErrorKind};
use aries_vcx::indy_sys::CommandHandle;
use aries_vcx::utils::error;
use aries_vcx::utils::filters;

use crate::api_lib::utils::cstring::CStringUtils;
use crate::api_lib::utils::error::set_current_error_vcx;
use crate::api_lib::utils::runtime::execute;

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

    execute(move || {
        match filters::filter_proof_requests_by_name(&requests, &match_name) {
            Ok(err) => {
                trace!("vcx_filter_proof_requests_by_name_cb(command_handle: {}, requests: {}, rc: {}, requests: {})",
                       command_handle, requests, error::SUCCESS.message, err);
                let err = CStringUtils::string_to_cstring(err);
                cb(command_handle, error::SUCCESS.code_num, err.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
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
    use std::ffi::CString;

    use aries_vcx::agency_client::testing::mocking::AgencyMockDecrypted;
    use aries_vcx::utils::{constants::GET_MESSAGES_DECRYPTED_RESPONSE, devsetup::*, error, mockdata::mockdata_proof};

    use crate::api_lib::api_c::filters::vcx_filter_proof_requests_by_name;
    use crate::api_lib::api_handle::connection;
    use crate::api_lib::api_handle::disclosed_proof::get_proof_request_messages;
    use crate::api_lib::utils::return_types_u32;
    use crate::api_lib::utils::timeout::TimeoutUtils;

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_vcx_filter_proof_requests_by_name() {
        let _setup = SetupMocks::init();

        let connection_h = connection::tests::build_test_connection_inviter_requested().await;

        AgencyMockDecrypted::set_next_decrypted_response(GET_MESSAGES_DECRYPTED_RESPONSE);
        AgencyMockDecrypted::set_next_decrypted_message(mockdata_proof::PRESENTATION_REQUEST_MESSAGE_1);
        AgencyMockDecrypted::set_next_decrypted_message(mockdata_proof::PRESENTATION_REQUEST_MESSAGE_2);

        let messages = get_proof_request_messages(connection_h).await.unwrap();
        let requests = CString::new(messages).unwrap().into_raw();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        let match_name = CString::new("request2".to_string()).unwrap().into_raw();
        assert_eq!(vcx_filter_proof_requests_by_name(cb.command_handle, requests, match_name, Some(cb.get_callback())), error::SUCCESS.code_num);
        let request = cb.receive(TimeoutUtils::some_short()).unwrap().unwrap();
        let value = serde_json::from_str::<serde_json::Value>(&request).unwrap();
        let requests = value.as_array().unwrap();
        assert_eq!(requests.len(), 1);
    }
}
