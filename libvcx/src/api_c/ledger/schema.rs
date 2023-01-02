use std::ptr;

use futures::future::BoxFuture;
use libc::c_char;
use serde_json;

use crate::api_c::types::CommandHandle;

use crate::api_vcx::api_handle::schema;
use crate::errors::error;
use crate::errors::error::{LibvcxError, LibvcxErrorKind};

use crate::api_c::cutils::cstring::CStringUtils;
use crate::api_c::cutils::current_error::set_current_error_vcx;
use crate::api_c::cutils::runtime::{execute, execute_async};

/// Create a new Schema object and publish corresponding record on the ledger
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: Enterprise's personal identification for the user.
///
/// schema_name: Name of schema
///
/// version: Version of schema. A semver-compatible value like "1.0" is encouraged.
///
/// schema_data: A list of attributes that will make up the schema, represented
///    as a string containing a JSON array. The number of attributes should be
///    less or equal to 125, because larger arrays cause various downstream problems.
///    This limitation is an annoyance that we'd like to remove.
///
/// # Example schema_data -> "["attr1", "attr2", "attr3"]"
///
/// payment_handle: Reserved for future use (currently uses any address in the wallet)
///
/// cb: Callback that provides Schema handle and error status of request.
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_schema_create(
    command_handle: CommandHandle,
    source_id: *const c_char,
    schema_name: *const c_char,
    version: *const c_char,
    schema_data: *const c_char,
    _payment_handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, credentialdef_handle: u32)>,
) -> u32 {
    info!("vcx_schema_create >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(schema_name, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(version, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(schema_data, LibvcxErrorKind::InvalidOption);

    trace!(target: "vcx", "vcx_schema_create(command_handle: {}, source_id: {}, schema_name: {},  schema_data: {})",
           command_handle, source_id, schema_name, schema_data);

    // todo: schema::create_and_publish_schema must have method in api_vcx layer, should include issuer_did loading
    // - similar also for functions below
    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match schema::create_and_publish_schema(&source_id, schema_name, version, schema_data).await {
            Ok(err) => {
                trace!(target: "vcx", "vcx_schema_create_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                       command_handle, error::SUCCESS_ERR_CODE, err, source_id);
                cb(command_handle, error::SUCCESS_ERR_CODE, err);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_schema_create_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                    command_handle, err, 0, source_id
                );
                cb(command_handle, err.into(), 0);
            }
        };

        Ok(())
    }));

    error::SUCCESS_ERR_CODE
}

/// Create a new Schema object that will be published by Endorser later.
///
/// Note that Schema can't be used for credential issuing until it will be published on the ledger.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: Enterprise's personal identification for the user.
///
/// schema_name: Name of schema
///
/// version: version of schema
///
/// schema_data: list of attributes that will make up the schema (the number of attributes should be less or equal than 125)
///
/// endorser: DID of the Endorser that will submit the transaction.
///
/// # Example schema_data -> "["attr1", "attr2", "attr3"]"
///
/// cb: Callback that provides Schema handle and Schema transaction that should be passed to Endorser for publishing.
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_schema_prepare_for_endorser(
    command_handle: CommandHandle,
    source_id: *const c_char,
    schema_name: *const c_char,
    version: *const c_char,
    schema_data: *const c_char,
    endorser: *const c_char,
    cb: Option<
        extern "C" fn(xcommand_handle: CommandHandle, err: u32, schema_handle: u32, schema_transaction: *const c_char),
    >,
) -> u32 {
    info!("vcx_schema_prepare_for_endorser >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(schema_name, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(version, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(schema_data, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(endorser, LibvcxErrorKind::InvalidOption);

    trace!(target: "vcx", "vcx_schema_prepare_for_endorser(command_handle: {}, source_id: {}, schema_name: {},  schema_data: {},  endorser: {})",
           command_handle, source_id, schema_name, schema_data, endorser);

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match schema::prepare_schema_for_endorser(&source_id, schema_name, version, schema_data, endorser).await {
            Ok((handle, transaction)) => {
                trace!(target: "vcx", "vcx_schema_prepare_for_endorser(command_handle: {}, rc: {}, handle: {}, transaction: {}) source_id: {}",
                       command_handle, error::SUCCESS_ERR_CODE, handle, transaction, source_id);
                let transaction = CStringUtils::string_to_cstring(transaction);
                cb(command_handle, error::SUCCESS_ERR_CODE, handle, transaction.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!("vcx_schema_prepare_for_endorser(command_handle: {}, rc: {}, handle: {}, transaction: {}) source_id: {}", command_handle, err, 0, "", source_id);
                cb(command_handle, err.into(), 0, ptr::null_mut());
            }
        };

        Ok(())
    }));

    error::SUCCESS_ERR_CODE
}

/// Takes the schema object and returns a json string of all its attributes
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// schema_handle: Schema handle that was provided during creation. Used to access schema object
///
/// cb: Callback that provides json string of the schema's attributes and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_schema_serialize(
    command_handle: CommandHandle,
    schema_handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, schema_state: *const c_char)>,
) -> u32 {
    info!("vcx_schema_serialize >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    let source_id = schema::get_source_id(schema_handle).unwrap_or_default();
    trace!(
        "vcx_schema_serialize(command_handle: {}, schema_handle: {}) source_id: {}",
        command_handle,
        schema_handle,
        source_id
    );

    if !schema::is_valid_handle(schema_handle) {
        return LibvcxError::from_msg(
            LibvcxErrorKind::InvalidSchemaHandle,
            format!("Invalid schema handle {}", schema_handle),
        )
        .into();
    };

    execute(move || {
        match schema::to_string(schema_handle) {
            Ok(err) => {
                trace!(
                    "vcx_schema_serialize_cb(command_handle: {}, schema_handle: {}, rc: {}, state: {}) source_id: {}",
                    command_handle,
                    schema_handle,
                    error::SUCCESS_ERR_CODE,
                    err,
                    source_id
                );
                let msg = CStringUtils::string_to_cstring(err);
                cb(command_handle, error::SUCCESS_ERR_CODE, msg.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_schema_serialize_cb(command_handle: {}, schema_handle: {}, rc: {}, state: {}) source_id: {}",
                    command_handle, schema_handle, err, "null", source_id
                );
                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS_ERR_CODE
}

/// Takes a json string representing a schema object and recreates an object matching the json
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// schema_data: json string representing a schema object
///
/// cb: Callback that provides schema handle and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_schema_deserialize(
    command_handle: CommandHandle,
    schema_data: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, schema_handle: u32)>,
) -> u32 {
    info!("vcx_schema_deserialize >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(schema_data, LibvcxErrorKind::InvalidOption);

    trace!(
        "vcx_schema_deserialize(command_handle: {}, schema_data: {})",
        command_handle,
        schema_data
    );
    execute(move || {
        match schema::from_string(&schema_data) {
            Ok(err) => {
                trace!(
                    "vcx_schema_deserialize_cb(command_handle: {}, rc: {}, handle: {}), source_id: {}",
                    command_handle,
                    error::SUCCESS_ERR_CODE,
                    err,
                    schema::get_source_id(err).unwrap_or_default()
                );
                cb(command_handle, error::SUCCESS_ERR_CODE, err);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_schema_deserialize_cb(command_handle: {}, rc: {}, handle: {}), source_id: {}",
                    command_handle, err, 0, ""
                );
                cb(command_handle, err.into(), 0);
            }
        };

        Ok(())
    });

    error::SUCCESS_ERR_CODE
}

/// Releases the schema object by de-allocating memory
///
/// #Params
/// schema_handle: Schema handle that was provided during creation. Used to access schema object
///
/// #Returns
/// Success
#[no_mangle]
pub extern "C" fn vcx_schema_release(schema_handle: u32) -> u32 {
    info!("vcx_schema_release >>>");

    let source_id = schema::get_source_id(schema_handle).unwrap_or_default();
    match schema::release(schema_handle) {
        Ok(()) => {
            trace!(
                "vcx_schema_release(schema_handle: {}, rc: {}), source_id: {}",
                schema_handle,
                error::SUCCESS_ERR_CODE,
                source_id
            );
            error::SUCCESS_ERR_CODE
        }
        Err(err) => {
            set_current_error_vcx(&err);
            error!(
                "vcx_schema_release(schema_handle: {}, rc: {}), source_id: {}",
                schema_handle, err, source_id
            );
            err.into()
        }
    }
}

/// Retrieves schema's id
///
/// #Params
/// schema_handle: Schema handle that was provided during creation. Used to access proof object
///
/// cb: Callback that provides schema id and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_schema_get_schema_id(
    command_handle: CommandHandle,
    schema_handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, schema_id: *const c_char)>,
) -> u32 {
    info!("vcx_schema_get_schema_id >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    trace!(
        "vcx_schema_get_schema_id(command_handle: {}, schema_handle: {})",
        command_handle,
        schema_handle
    );
    if !schema::is_valid_handle(schema_handle) {
        return LibvcxError::from_msg(
            LibvcxErrorKind::InvalidSchemaHandle,
            format!("Invalid schema handle {}", schema_handle),
        )
        .into();
    }

    execute(move || {
        match schema::get_schema_id(schema_handle) {
            Ok(err) => {
                trace!(
                    "vcx_schema_get_schema_id(command_handle: {}, schema_handle: {}, rc: {}, schema_seq_no: {})",
                    command_handle,
                    schema_handle,
                    error::SUCCESS_ERR_CODE,
                    err
                );
                let msg = CStringUtils::string_to_cstring(err);
                cb(command_handle, error::SUCCESS_ERR_CODE, msg.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_schema_get_schema_id(command_handle: {}, schema_handle: {}, rc: {}, schema_seq_no: {})",
                    command_handle, schema_handle, err, ""
                );
                cb(command_handle, err.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS_ERR_CODE
}

/// Retrieves all of the data associated with a schema on the ledger.
///
/// #Params
/// source_id: Enterprise's personal identification for the user.
///
/// schema_id: id of schema given during the creation of the schema
///
/// cb: Callback contains the error status (if the schema cannot be found)
/// and it will also contain a json string representing all of the data of a
/// schema already on the ledger.
///
/// # Example
/// schema -> {"data":["height","name","sex","age"],"name":"test-licence","payment_txn":null,"schema_id":"2hoqvcwupRTUNkXn6ArYzs:2:test-licence:4.4.4","source_id":"Test Source ID","state":1,"version":"4.4.4"}
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_schema_get_attributes(
    command_handle: CommandHandle,
    source_id: *const c_char,
    schema_id: *const c_char,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, s_handle: u32, schema_attrs: *const c_char)>,
) -> u32 {
    info!("vcx_schema_get_attributes >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, LibvcxErrorKind::InvalidOption);
    check_useful_c_str!(schema_id, LibvcxErrorKind::InvalidOption);
    trace!(
        "vcx_schema_get_attributes(command_handle: {}, source_id: {}, schema_id: {})",
        command_handle,
        source_id,
        schema_id
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match schema::get_schema_attrs(source_id, schema_id).await {
            Ok((handle, data)) => {
                let data: serde_json::Value =
                    serde_json::from_str(&data).unwrap_or_else(|_| panic!("unexpected error deserializing data: {}", data));
                let data = data["data"].clone();
                trace!(
                    "vcx_schema_get_attributes_cb(command_handle: {}, rc: {}, handle: {}, attrs: {})",
                    command_handle,
                    error::SUCCESS_ERR_CODE,
                    handle,
                    data
                );
                let msg = CStringUtils::string_to_cstring(data.to_string());
                cb(command_handle, error::SUCCESS_ERR_CODE, handle, msg.as_ptr());
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_schema_get_attributes_cb(command_handle: {}, rc: {}, handle: {}, attrs: {})",
                    command_handle, err, 0, ""
                );
                cb(command_handle, err.into(), 0, ptr::null_mut());
            }
        };

        Ok(())
    }));

    error::SUCCESS_ERR_CODE
}

/// Checks if schema is published on the Ledger and updates the  state
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// schema_handle: Schema handle that was provided during creation. Used to access schema object
///
/// cb: Callback that provides most current state of the schema and error status of request
///     States:
///         0 = Built
///         1 = Published
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_schema_update_state(
    command_handle: CommandHandle,
    schema_handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, state: u32)>,
) -> u32 {
    info!("vcx_schema_update_state >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    let source_id = schema::get_source_id(schema_handle).unwrap_or_default();

    trace!(
        "vcx_schema_update_state(command_handle: {}, schema_handle: {}) source_id: {}",
        command_handle,
        schema_handle,
        source_id
    );

    execute_async::<BoxFuture<'static, Result<(), ()>>>(Box::pin(async move {
        match schema::update_state(schema_handle).await {
            Ok(state) => {
                trace!(
                    "vcx_schema_update_state(command_handle: {}, rc: {}, state: {})",
                    command_handle,
                    error::SUCCESS_ERR_CODE,
                    state
                );
                cb(command_handle, error::SUCCESS_ERR_CODE, state);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_schema_update_state(command_handle: {}, rc: {}, state: {})",
                    command_handle, err, 0
                );
                cb(command_handle, err.into(), 0);
            }
        };

        Ok(())
    }));

    error::SUCCESS_ERR_CODE
}

/// Get the current state of the schema object
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// schema_handle: Schema handle that was provided during creation. Used to access schema object
///
/// cb: Callback that provides most current state of the schema and error status of request
///     States:
///         0 = Built
///         1 = Published
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern "C" fn vcx_schema_get_state(
    command_handle: CommandHandle,
    schema_handle: u32,
    cb: Option<extern "C" fn(xcommand_handle: CommandHandle, err: u32, state: u32)>,
) -> u32 {
    info!("vcx_schema_get_state >>>");

    check_useful_c_callback!(cb, LibvcxErrorKind::InvalidOption);

    let source_id = schema::get_source_id(schema_handle).unwrap_or_default();
    trace!(
        "vcx_schema_get_state(command_handle: {}, schema_handle: {}) source_id: {}",
        command_handle,
        schema_handle,
        source_id
    );

    execute(move || {
        match schema::get_state(schema_handle) {
            Ok(state) => {
                trace!(
                    "vcx_schema_get_state(command_handle: {}, rc: {}, state: {})",
                    command_handle,
                    error::SUCCESS_ERR_CODE,
                    state
                );
                cb(command_handle, error::SUCCESS_ERR_CODE, state);
            }
            Err(err) => {
                set_current_error_vcx(&err);
                error!(
                    "vcx_schema_get_state(command_handle: {}, rc: {}, state: {})",
                    command_handle, err, 0
                );
                cb(command_handle, err.into(), 0);
            }
        };

        Ok(())
    });

    error::SUCCESS_ERR_CODE
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use aries_vcx::common::primitives::credential_definition::PublicEntityStateType;
    use aries_vcx::global::settings::CONFIG_INSTITUTION_DID;
    use aries_vcx::utils;
    use aries_vcx::utils::constants::{DEFAULT_SCHEMA_ID, SCHEMA_ID, SCHEMA_WITH_VERSION};
    use aries_vcx::utils::devsetup::SetupMocks;

    use crate::api_c::cutils::return_types_u32;
    use crate::api_c::cutils::timeout::TimeoutUtils;
    use crate::api_vcx::api_global::settings;
    use crate::api_vcx::api_global::settings::get_config_value;
    use crate::api_vcx::api_handle::schema::prepare_schema_for_endorser;
    use crate::api_vcx::api_handle::schema::tests::prepare_schema_data;
    use crate::errors::error;

    use super::*;

    fn vcx_schema_create_c_closure(name: &str, version: &str, data: &str) -> Result<u32, u32> {
        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        let rc = vcx_schema_create(
            cb.command_handle,
            CString::new("Test Source ID").unwrap().into_raw(),
            CString::new(name).unwrap().into_raw(),
            CString::new(version).unwrap().into_raw(),
            CString::new(data).unwrap().into_raw(),
            0,
            Some(cb.get_callback()),
        );
        if rc != error::SUCCESS_ERR_CODE {
            return Err(rc);
        }

        let handle = cb.receive(TimeoutUtils::some_medium()).unwrap();
        Ok(handle)
    }

    fn vcx_schema_serialize_c_closure(handle: u32) -> String {
        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(
            vcx_schema_serialize(cb.command_handle, handle, Some(cb.get_callback())),
            error::SUCCESS_ERR_CODE
        );
        let schema_json = cb.receive(TimeoutUtils::some_short()).unwrap().unwrap();
        schema_json
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_create_schema_success() {
        let _setup = SetupMocks::init();

        let (_, schema_name, schema_version, data) = prepare_schema_data();
        let handle = vcx_schema_create_c_closure(&schema_name, &schema_version, &data).unwrap();
        assert!(handle > 0)
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_schema_serialize() {
        let _setup = SetupMocks::init();

        let (_, schema_name, schema_version, data) = prepare_schema_data();
        let handle = vcx_schema_create_c_closure(&schema_name, &schema_version, &data).unwrap();

        let _schema_json = vcx_schema_serialize_c_closure(handle);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_schema_deserialize_succeeds() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        let err = vcx_schema_deserialize(
            cb.command_handle,
            CString::new(SCHEMA_WITH_VERSION).unwrap().into_raw(),
            Some(cb.get_callback()),
        );
        assert_eq!(err, error::SUCCESS_ERR_CODE);
        let schema_handle = cb.receive(TimeoutUtils::some_short()).unwrap();
        assert!(schema_handle > 0);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_schema_get_schema_id_succeeds() {
        let _setup = SetupMocks::init();

        let (_, schema_name, schema_version, data) = prepare_schema_data();
        let schema_handle = vcx_schema_create_c_closure(&schema_name, &schema_version, &data).unwrap();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(
            vcx_schema_get_schema_id(cb.command_handle, schema_handle, Some(cb.get_callback())),
            error::SUCCESS_ERR_CODE
        );
        let id = cb.receive(TimeoutUtils::some_short()).unwrap().unwrap();
        assert_eq!(DEFAULT_SCHEMA_ID, &id);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_schema_get_attrs() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32_U32_STR::new().unwrap();
        let data = r#"["height","name","sex","age"]"#;
        assert_eq!(
            vcx_schema_get_attributes(
                cb.command_handle,
                CString::new("Test Source ID").unwrap().into_raw(),
                CString::new(SCHEMA_ID).unwrap().into_raw(),
                Some(cb.get_callback()),
            ),
            error::SUCCESS_ERR_CODE
        );
        let (_handle, schema_data_as_string) = cb.receive(TimeoutUtils::some_short()).unwrap();
        let schema_data_as_string = schema_data_as_string.unwrap();
        let schema_as_json: serde_json::Value = serde_json::from_str(&schema_data_as_string).unwrap();
        assert_eq!(schema_as_json["data"].to_string(), data);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_schema_release() {
        let _setup = SetupMocks::init();

        let (_, schema_name, schema_version, data) = prepare_schema_data();
        let handle = vcx_schema_create_c_closure(&schema_name, &schema_version, &data).unwrap();

        let unknown_handle = handle + 1;
        assert_eq!(
            vcx_schema_release(unknown_handle),
            u32::from(LibvcxErrorKind::InvalidSchemaHandle)
        );
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_vcx_prepare_schema_success() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32_U32_STR::new().unwrap();
        assert_eq!(
            vcx_schema_prepare_for_endorser(
                cb.command_handle,
                CString::new("Test Source ID").unwrap().into_raw(),
                CString::new("Test Schema").unwrap().into_raw(),
                CString::new("0.0").unwrap().into_raw(),
                CString::new("[\"attr\", \"att2\"]").unwrap().into_raw(),
                CString::new("V4SGRU86Z58d6TV7PBUe6f").unwrap().into_raw(),
                Some(cb.get_callback()),
            ),
            error::SUCCESS_ERR_CODE
        );
        let (_handle, schema_transaction) = cb.receive(TimeoutUtils::some_short()).unwrap();
        let schema_transaction = schema_transaction.unwrap();
        let schema_transaction: serde_json::Value = serde_json::from_str(&schema_transaction).unwrap();
        let expected_schema_transaction: serde_json::Value =
            serde_json::from_str(utils::constants::REQUEST_WITH_ENDORSER).unwrap();
        assert_eq!(expected_schema_transaction, schema_transaction);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_vcx_schema_get_state() {
        let _setup = SetupMocks::init();

        let did = get_config_value(CONFIG_INSTITUTION_DID).unwrap();
        let (handle, _) = prepare_schema_for_endorser(
            "testid",
            "name".to_string(),
            "1.0".to_string(),
            "[\"name\",\"gender\"]".to_string(),
            "V4SGRU86Z58d6TV7PBUe6f".to_string(),
        )
        .await
        .unwrap();
        {
            let cb = return_types_u32::Return_U32_U32::new().unwrap();
            let _rc = vcx_schema_get_state(cb.command_handle, handle, Some(cb.get_callback()));
            assert_eq!(
                cb.receive(TimeoutUtils::some_medium()).unwrap(),
                PublicEntityStateType::Built as u32
            )
        }
        {
            let cb = return_types_u32::Return_U32_U32::new().unwrap();
            let _rc = vcx_schema_update_state(cb.command_handle, handle, Some(cb.get_callback()));
            assert_eq!(
                cb.receive(TimeoutUtils::some_medium()).unwrap(),
                PublicEntityStateType::Published as u32
            );
        }
        {
            let cb = return_types_u32::Return_U32_U32::new().unwrap();
            let _rc = vcx_schema_get_state(cb.command_handle, handle, Some(cb.get_callback()));
            assert_eq!(
                cb.receive(TimeoutUtils::some_medium()).unwrap(),
                PublicEntityStateType::Published as u32
            )
        }
    }
}
