use std::ptr;
use libc::{c_char, c_void};
use async_std::sync::{Arc, Mutex};
use async_std::task::block_on;

use indy_api_types::{CommandHandle, ErrorCode, errors::prelude::*, validation::Validatable, WalletHandle};
use indy_utils::ctypes;

use crate::Locator;
use crate::services::CommandMetric;
use crate::domain::{
    cache::GetCacheOptions,
    vdr::{
        taa_config::TAAConfig,
        namespaces::Namespaces,
    },
};
use crate::controllers::vdr::{
    VDR,
    VDRBuilder,
};

/// Create a Builder object for Verifiable Data Registry which provides a unified interface for interactions with supported Ledgers.
///
/// EXPERIMENTAL
///
/// #Params
/// vdr_builder_p: pointer to store VDRBuilder object
///
/// #Returns
/// Error Code
#[no_mangle]
pub extern "C" fn vdr_builder_create(
    vdr_builder_p: *mut *const c_void,
) -> ErrorCode {
    debug!("vdr_builder_create >");

    let vdr_builder = Arc::new(Mutex::new(VDRBuilder::create()));

    unsafe {
        *vdr_builder_p = Box::into_raw(Box::new(vdr_builder)) as *const c_void;
    }

    let res = ErrorCode::Success;
    debug!("vdr_builder_create > {:?}", res);
    res
}

/// Register Indy Ledger in the VDR object.
/// Associate registered Indy Ledger with the list of specified namespaces that will be used for
/// the resolution of public entities referencing by fully qualified identifiers.
///
/// EXPERIMENTAL
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// vdr_builder: pointer to VDRBuilder object
/// namespace_list: list of namespaces to associated with Ledger ('["namespace_1", "namespace_2"]')
/// genesis_txn_data: genesis transactions for Indy Ledger (Note that node transactions must be located in separate lines)
/// taa_config: accepted transaction author agreement data:
///     {
///         text and version - (optional) raw data about TAA from ledger.
///                             These parameters should be passed together.
///                             These parameters are required if taa_digest parameter is omitted.
///         taa_digest - (optional) digest on text and version.
///                             Digest is sha256 hash calculated on concatenated strings: version || text.
///                             This parameter is required if text and version parameters are omitted.
///         acc_mech_type - mechanism how user has accepted the TAA
///         time - UTC timestamp when user has accepted the TAA. Note that the time portion will be discarded to avoid a privacy risk.
///     }
///
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: com

#[no_mangle]
pub extern "C" fn vdr_builder_register_indy_ledger(
    command_handle: CommandHandle,
    vdr_builder: *const c_void,
    namespace_list: *const c_char,
    genesis_txn_data: *const c_char,
    taa_config: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode)>,
) -> ErrorCode {
    debug!(
        "vdr_builder_register_indy_ledger > vdr_builder {:?} namespace_list {:?} genesis_txn_data {:?} taa_config {:?}",
        vdr_builder, namespace_list, genesis_txn_data, taa_config
    );

    check_useful_c_reference!(vdr_builder, Arc<Mutex<VDRBuilder>>, ErrorCode::CommonInvalidParam1);
    check_useful_validatable_json!(namespace_list, ErrorCode::CommonInvalidParam3, Namespaces);
    check_useful_c_str!(genesis_txn_data, ErrorCode::CommonInvalidParam4);
    check_useful_opt_json!(taa_config, ErrorCode::CommonInvalidParam5, TAAConfig);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    debug!(
        "vdr_builder_register_indy_ledger ? namespace_list {:?} genesis_txn_data {:?} taa_config {:?}",
        namespace_list, genesis_txn_data, taa_config
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .vdr_controller
            .register_indy_ledger(vdr_builder.clone(), namespace_list, genesis_txn_data, taa_config)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        debug!("vdr_builder_register_indy_ledger <<<");

        let err = prepare_result!(res);

        debug!("vdr_builder_register_indy_ledger ? err {:?} ", err);

        cb(command_handle, err)
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::VdrCommandRegisterIndyLedger, action, cb);

    let res = ErrorCode::Success;
    debug!("vdr_builder_register_indy_ledger > {:?}", res);
    res
}

/// Finalize building of VDR object and receive a pointer to VDR providing a unified interface for interactions with supported Ledgers.
///
/// EXPERIMENTAL
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// vdr_builder: pointer to VDRBuilder object
/// vdr_p: pointer to store VDR object
///
/// #Returns
/// Error Code
#[no_mangle]
pub extern "C" fn vdr_builder_finalize(
    vdr_builder: *const c_void,
    vdr_p: *mut *const c_void,
) -> ErrorCode {
    debug!("vdr_builder_finalize >");

    check_useful_c_ptr!(vdr_builder, ErrorCode::CommonInvalidParam2);

    debug!("vdr_builder_finalize ?");

    let vdr_builder = unsafe { Box::from_raw(vdr_builder as *mut Arc<Mutex<VDRBuilder>>) };

    block_on(async {
        let vdr_builder = vdr_builder.lock().await;
        let vdr = vdr_builder.finalize();
        unsafe {
            *vdr_p = Box::into_raw(Box::new(vdr)) as *const c_void;
        }
    });

    let res = ErrorCode::Success;
    debug!("vdr_builder_finalize > {:?}", res);
    res
}

/// Ping Ledgers registered in the VDR.
///
/// NOTE: This function MUST be called for Indy Ledgers before sending any request.
///
/// Indy Ledger: The function performs sync with the ledger and returns the most recent nodes state.
/// Cheqd Ledger: The function query network information.
///
/// EXPERIMENTAL
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// vdr: pointer to VDR object
/// namespace_list: list of namespaces to ping
/// cb: Callback that takes command result as parameter
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: command handle to map callback to caller context.
/// - err: Error code.
#[no_mangle]
pub extern "C" fn vdr_ping(
    command_handle: CommandHandle,
    vdr: *const c_void,
    namespace_list: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, status_list: *const c_char)>,
) -> ErrorCode {
    debug!("vdr_ping > namespace_list {:?}", namespace_list);

    check_useful_c_reference!(vdr, VDR, ErrorCode::CommonInvalidParam1);
    check_useful_validatable_json!(namespace_list, ErrorCode::CommonInvalidParam3, Namespaces);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    debug!("vdr_ping ? namespace_list {:?} ", namespace_list);

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .vdr_controller
            .ping(vdr, namespace_list)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, status_list) = prepare_result!(res, String::new());

        debug!("vdr_ping ? err {:?} status_list {:?}", err, status_list);

        let status_list = ctypes::string_to_cstring(status_list);

        cb(command_handle, err, status_list.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::VdrCommandPing, action, cb);

    let res = ErrorCode::Success;
    debug!("vdr_ping > {:?}", res);
    res
}

/// Drop VDR object and associated Ledger connections.
///
/// EXPERIMENTAL
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// vdr: pointer to VDR object
/// cb: Callback that takes command result as parameter
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: command handle to map callback to caller context.
/// - err: Error code.

#[no_mangle]
pub extern "C" fn vdr_cleanup(
    command_handle: CommandHandle,
    vdr: *const c_void,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode)>,
) -> ErrorCode {
    debug!("vdr_cleanup >");

    check_useful_c_ptr!(vdr, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    debug!("vdr_cleanup ?");

    let mut vdr = unsafe { Box::from_raw(vdr as *mut VDR) };

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .vdr_controller
            .cleanup(&mut vdr)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let err = prepare_result!(res);

        debug!("vdr_cleanup ? err {:?} ", err);

        cb(command_handle, err)
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::VdrCommandCleanup, action, cb);

    let res = ErrorCode::Success;
    debug!("vdr_cleanup > {:?}", res);
    res
}

/// Resolve DID information for specified fully-qualified DID.
///
/// EXPERIMENTAL
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// vdr: pointer to VDR object
/// fqdid: fully-qualified DID of the target DID on the Ledger
/// cb: Callback that takes command result as parameter
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: command handle to map callback to caller context.
/// - err: Error code.
/// - diddoc: Resolved DID information.
///           Note that the format of the value depends on the Ledger type:
///     Indy:    {
///             "did": string
///             "verkey": string
///         }
#[no_mangle]
pub extern "C" fn vdr_resolve_did(
    command_handle: CommandHandle,
    vdr: *const c_void,
    fqdid: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, diddoc: *const c_char)>,
) -> ErrorCode {
    debug!(
        "vdr_resolve_did > fqdid {:?}",
        fqdid
    );

    check_useful_c_reference!(vdr, VDR, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(fqdid, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    debug!(
        "vdr_resolve_did ? fqdid {:?}",
        fqdid
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .vdr_controller
            .resolve_did(vdr, &fqdid)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, diddoc) = prepare_result!(res, String::new());

        debug!("vdr_resolve_did ? err {:?} diddoc {:?}", err, diddoc);

        let diddoc = ctypes::string_to_cstring(diddoc);

        cb(command_handle, err, diddoc.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::VdrCommandResolveDid, action, cb);

    let res = ErrorCode::Success;
    debug!("vdr_resolve_did > {:?}", res);
    res
}

/// Resolve DID information for specified fully-qualified DID with using of wallet cache.
///
/// If data is present inside of wallet cache, cached data is returned.
/// Otherwise data is fetched from the associated Ledger and stored inside of cache for future use.
///
/// EXPERIMENTAL
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// vdr: pointer to VDR object
/// wallet_handle: handle pointing to an opened wallet (returned by indy_open_wallet)
/// fqdid: fully-qualified DID of the target DID on the Ledger
/// cache_options: caching options
///     {
///         forceUpdate: (optional, false by default) Force update of record in cache from the ledger,
///     }
/// cb: Callback that takes command result as parameter
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: command handle to map callback to caller context.
/// - err: Error code.
/// - diddoc: Resolved DID information.
///           Note that the format of the value depends on the Ledger type:
///     Indy:    {
///             "did": string
///             "verkey": string
///         }
#[no_mangle]
pub extern "C" fn vdr_resolve_did_with_cache(
    command_handle: CommandHandle,
    vdr: *const c_void,
    wallet_handle: WalletHandle,
    fqdid: *const c_char,
    cache_options: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, diddoc: *const c_char)>,
) -> ErrorCode {
    debug!(
        "vdr_resolve_did_with_cache > wallet_handle {:?} fqdid {:?} cache_options {:?}",
        wallet_handle, fqdid, cache_options
    );

    check_useful_c_reference!(vdr, VDR, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(fqdid, ErrorCode::CommonInvalidParam4);
    check_useful_json!(cache_options, ErrorCode::CommonInvalidParam5, GetCacheOptions);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    debug!(
        "vdr_resolve_did_with_cache ? wallet_handle {:?} fqdid {:?} cache_options {:?}",
        wallet_handle, fqdid, cache_options
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .vdr_controller
            .resolve_did_with_cache(vdr, wallet_handle, &fqdid, &cache_options)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, diddoc) = prepare_result!(res, String::new());

        debug!("vdr_resolve_did_with_cache ? err {:?} diddoc {:?}", err, diddoc);

        let diddoc = ctypes::string_to_cstring(diddoc);

        cb(command_handle, err, diddoc.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::VdrCommandResolveDidWithCache, action, cb);

    let res = ErrorCode::Success;
    debug!("vdr_resolve_did_with_cache > {:?}", res);
    res
}

/// Resolve Schema for specified fully-qualified ID.
///
/// EXPERIMENTAL
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// vdr: pointer to VDR object
/// fqschema: fully-qualified Schema ID of the target Schema on the Ledger
/// cb: Callback that takes command result as parameter
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: command handle to map callback to caller context.
/// - err: Error code.
/// - schema: Resolved Schema
///     {
///         id: identifier of schema
///         attrNames: array of attribute name strings
///         name: Schema's name string
///         version: Schema's version string
///         ver: Version of the Schema json
///     }
#[no_mangle]
pub extern "C" fn vdr_resolve_schema(
    command_handle: CommandHandle,
    vdr: *const c_void,
    fqschema: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, schema: *const c_char)>,
) -> ErrorCode {
    debug!(
        "vdr_resolve_schema > fqschema {:?}",
        fqschema,
    );

    check_useful_c_reference!(vdr, VDR, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(fqschema, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    debug!(
        "vdr_resolve_schema ? fqschema {:?}",
        fqschema,
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .vdr_controller
            .resolve_schema(vdr, &fqschema)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, schema) = prepare_result!(res, String::new());

        debug!("vdr_resolve_schema ? err {:?} schema {:?}", err, schema);

        let schema = ctypes::string_to_cstring(schema);

        cb(command_handle, err, schema.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::VdrCommandResolveSchema, action, cb);

    let res = ErrorCode::Success;
    debug!("vdr_resolve_schema > {:?}", res);
    res
}

/// Resolve Schema for specified fully-qualified ID with using of wallet cache.
///
/// If data is present inside of wallet cache, cached data is returned.
/// Otherwise data is fetched from the associated Ledger and stored inside of cache for future use.
///
/// EXPERIMENTAL
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// vdr: pointer to VDR object
/// wallet_handle: handle pointing to an opened wallet (returned by indy_open_wallet)
/// fqschema: fully-qualified Schema ID of the target Schema on the Ledger
/// cache_options: caching options
///     {
///         forceUpdate: (optional, false by default) Force update of record in cache from the ledger,
///     }
/// cb: Callback that takes command result as parameter
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: command handle to map callback to caller context.
/// - err: Error code.
/// - schema: Resolved Schema
///     {
///         id: identifier of schema
///         attrNames: array of attribute name strings
///         name: Schema's name string
///         version: Schema's version string
///         ver: Version of the Schema json
///     }
#[no_mangle]
pub extern "C" fn vdr_resolve_schema_with_cache(
    command_handle: CommandHandle,
    vdr: *const c_void,
    wallet_handle: WalletHandle,
    fqschema: *const c_char,
    cache_options: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, schema: *const c_char)>,
) -> ErrorCode {
    debug!(
        "vdr_resolve_schema_with_cache > wallet_handle {:?} fqschema {:?} cache_options {:?}",
        wallet_handle, fqschema, cache_options
    );

    check_useful_c_reference!(vdr, VDR, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(fqschema, ErrorCode::CommonInvalidParam4);
    check_useful_json!(cache_options, ErrorCode::CommonInvalidParam5, GetCacheOptions);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    debug!(
        "vdr_resolve_schema_with_cache ? wallet_handle {:?} fqschema {:?} cache_options {:?}",
        wallet_handle, fqschema, cache_options
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .vdr_controller
            .resolve_schema_with_cache(vdr, wallet_handle, &fqschema, &cache_options)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, schema) = prepare_result!(res, String::new());

        debug!("vdr_resolve_schema_with_cache ? err {:?} schema {:?}", err, schema);

        let schema = ctypes::string_to_cstring(schema);

        cb(command_handle, err, schema.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::VdrCommandResolveSchema, action, cb);

    let res = ErrorCode::Success;
    debug!("vdr_resolve_schema_with_cache > {:?}", res);
    res
}

/// Resolve Credential Definition for specified fully-qualified ID.
///
/// EXPERIMENTAL
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// vdr: pointer to VDR object
/// fqcreddef: fully-qualified CredDef ID of the target CredentialDefinition on the Ledger
/// cb: Callback that takes command result as parameter
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: command handle to map callback to caller context.
/// - err: Error code.
/// - credential_definition: Resolved Credential Definition
///     {
///         id: string - identifier of credential definition
///         schemaId: string - identifier of stored in ledger schema
///         type: string - type of the credential definition. CL is the only supported type now.
///         tag: string - allows to distinct between credential definitions for the same issuer and schema
///         value: Dictionary with Credential Definition's data: {
///             primary: primary credential public key,
///             Optional<revocation>: revocation credential public key
///         },
///         ver: Version of the Credential Definition json
///     }
#[no_mangle]
pub extern "C" fn vdr_resolve_cred_def(
    command_handle: CommandHandle,
    vdr: *const c_void,
    fqcreddef: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, cred_def: *const c_char)>,
) -> ErrorCode {
    debug!(
        "vdr_resolve_cred_def > fqcreddef {:?}",
        fqcreddef,
    );

    check_useful_c_reference!(vdr, VDR, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(fqcreddef, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    debug!(
        "vdr_resolve_cred_def ? fqcreddef {:?}",
        fqcreddef,
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .vdr_controller
            .resolve_creddef(vdr, &fqcreddef)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, cred_def) = prepare_result!(res, String::new());

        debug!("vdr_resolve_cred_def ? err {:?} cred_def {:?}", err, cred_def);

        let cred_def = ctypes::string_to_cstring(cred_def);

        cb(command_handle, err, cred_def.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::VdrCommandResolveCredDef, action, cb);

    let res = ErrorCode::Success;
    debug!("vdr_resolve_cred_def > {:?}", res);
    res
}

/// Resolve Credential Definition for specified fully-qualified ID with using of wallet cache.
///
/// If data is present inside of wallet cache, cached data is returned.
/// Otherwise data is fetched from the associated Ledger and stored inside of cache for future use.
///
/// EXPERIMENTAL
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// vdr: pointer to VDR object
/// wallet_handle: handle pointing to an opened wallet (returned by indy_open_wallet)
/// fqcreddef: fully-qualified CredDef ID of the target CredentialDefinition on the Ledger
/// cache_options: caching options
///     {
///         forceUpdate: (optional, false by default) Force update of record in cache from the ledger,
///     }
/// cb: Callback that takes command result as parameter
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: command handle to map callback to caller context.
/// - err: Error code.
/// - credential_definition: Resolved Credential Definition
///     {
///         id: string - identifier of credential definition
///         schemaId: string - identifier of stored in ledger schema
///         type: string - type of the credential definition. CL is the only supported type now.
///         tag: string - allows to distinct between credential definitions for the same issuer and schema
///         value: Dictionary with Credential Definition's data: {
///             primary: primary credential public key,
///             Optional<revocation>: revocation credential public key
///         },
///         ver: Version of the Credential Definition json
///     }
#[no_mangle]
pub extern "C" fn vdr_resolve_cred_def_with_cache(
    command_handle: CommandHandle,
    vdr: *const c_void,
    wallet_handle: WalletHandle,
    fqcreddef: *const c_char,
    cache_options: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, cred_def: *const c_char)>,
) -> ErrorCode {
    debug!(
        "vdr_resolve_cred_def_with_cache > wallet_handle {:?} fqcreddef {:?} cache_options {:?}",
        wallet_handle, fqcreddef, cache_options
    );

    check_useful_c_reference!(vdr, VDR, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(fqcreddef, ErrorCode::CommonInvalidParam4);
    check_useful_json!(cache_options, ErrorCode::CommonInvalidParam5, GetCacheOptions);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    debug!(
        "vdr_resolve_cred_def_with_cache ? wallet_handle {:?} fqcreddef {:?} cache_options {:?}",
        wallet_handle, fqcreddef, cache_options
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .vdr_controller
            .resolve_creddef_with_cache(vdr, wallet_handle, &fqcreddef, &cache_options)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, cred_def) = prepare_result!(res, String::new());

        debug!("vdr_resolve_cred_def_with_cache ? err {:?} cred_def {:?}", err, cred_def);

        let cred_def = ctypes::string_to_cstring(cred_def);

        cb(command_handle, err, cred_def.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::VdrCommandResolveCredDef, action, cb);

    let res = ErrorCode::Success;
    debug!("vdr_resolve_cred_def_with_cache > {:?}", res);
    res
}


/// Prepare transaction to submit DID on the Ledger.
///
/// EXPERIMENTAL
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// vdr: pointer to VDR object
/// txn_specific_params: DID transaction specific data.
///                      Depends on the Ledger type:
///     Indy:
///         {
///             dest: string - Target DID as base58-encoded string.
///             verkey: Optional<string> - Target identity verification key as base58-encoded string.
///             alias: Optional<string> DID's alias.
///             role: Optional<string> Role of a user DID record:
///                             null (common USER)
///                             TRUSTEE
///                             STEWARD
///                             TRUST_ANCHOR
///                             ENDORSER - equal to TRUST_ANCHOR that will be removed soon
///                             NETWORK_MONITOR
///                             empty string to reset role
///         }
///     Cheqd: TBD
/// submitter_did: Fully-qualified DID of the transaction author as base58-encoded string.
/// endorser: DID of the Endorser that will endorse the transaction.
///           The Endorser's DID must be present on the ledger with 'ENDORSER' role.
/// cb: Callback that takes command result as parameter
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: command handle to map callback to caller context.
/// - err: Error code.
/// - namespace: Ledger namespace to submit transaction (captured from submitter DID)
/// - txn_bytes: prepared transaction as bytes
/// - signature_spec: type of the signature transaction must be signed with (one of: `Ed25519` or `Secp256k1`)
/// - bytes_to_sign: bytes must be signed
/// - endorsement_spec: endorsement process specification
#[no_mangle]
pub extern "C" fn vdr_prepare_did(
    command_handle: CommandHandle,
    vdr: *const c_void,
    txn_specific_params: *const c_char,
    submitter_did: *const c_char,
    endorser: *const c_char,
    cb: Option<extern "C" fn(
        command_handle_: CommandHandle,
        err: ErrorCode,
        namespace: *const c_char,
        txn_bytes_raw: *const u8,
        txn_bytes_len: u32,
        signature_spec: *const c_char,
        bytes_to_sign_raw: *const u8,
        bytes_to_sign_len: u32,
        endorsement_spec: *const c_char)>, ) -> ErrorCode {
    debug!(
        "vdr_prepare_did > txn_specific_params {:?} submitter_did {:?} endorser {:?}",
        txn_specific_params, submitter_did, endorser
    );

    check_useful_c_reference!(vdr, VDR, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(txn_specific_params, ErrorCode::CommonInvalidParam3);
    check_useful_c_str!(submitter_did, ErrorCode::CommonInvalidParam4);
    check_useful_opt_c_str!(endorser, ErrorCode::CommonInvalidParam5);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    debug!(
        "vdr_prepare_did ? txn_specific_params {:?} submitter_did {:?} endorser {:?}",
        txn_specific_params, submitter_did, endorser
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .vdr_controller
            .prepare_did_txn(vdr, txn_specific_params, submitter_did, endorser)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, (namespace, txn_bytes, signature_spec, bytes_to_sign, endorsement_spec)) = prepare_result!(
            res, String::new(), Vec::new(), String::new(), Vec::new(), None
        );

        debug!(
            "vdr_prepare_did ? err {:?} namespace {:?} signature_spec {:?} txn_bytes {:?} bytes_to_sign {:?} endorsement_spec {:?}",
            err, namespace, signature_spec, txn_bytes, bytes_to_sign, endorsement_spec);

        let namespace = ctypes::string_to_cstring(namespace);
        let signature_spec = ctypes::string_to_cstring(signature_spec);
        let (txn_data, txn_len) = ctypes::vec_to_pointer(&txn_bytes);
        let (bytes_data, bytes_len) = ctypes::vec_to_pointer(&bytes_to_sign);
        let endorsement_spec = endorsement_spec.map(ctypes::string_to_cstring);

        cb(
            command_handle,
            err,
            namespace.as_ptr(),
            txn_data,
            txn_len,
            signature_spec.as_ptr(),
            bytes_data,
            bytes_len,
            endorsement_spec
                .as_ref()
                .map(|vk| vk.as_ptr())
                .unwrap_or(ptr::null()),
        )
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::VdrCommandPrepareDid, action, cb);

    let res = ErrorCode::Success;
    debug!("vdr_prepare_did > {:?}", res);
    res
}

/// Prepare transaction to submit Schema on the Ledger.
///
/// EXPERIMENTAL
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// handle: handle pointing to created VDR object (returned by vdr_create)
/// txn_specific_params: Schema transaction specific data
///                      Depends on the Ledger type:
///     Indy:
///         {
///             id: identifier of schema
///             attrNames: array of attribute name strings (the number of attributes should be less or equal than 125)
///             name: Schema's name string
///             version: Schema's version string,
///             ver: Version of the Schema json
///         }
/// submitter_did: Fully-qualified DID of the transaction author as base58-encoded string.
/// endorser: DID of the Endorser that will endorse the transaction.
///           The Endorser's DID must be present on the ledger with 'ENDORSER' role.
/// cb: Callback that takes command result as parameter
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: command handle to map callback to caller context.
/// - err: Error code.
/// - namespace: Ledger namespace to submit transaction (captured from submitter DID)
/// - txn_bytes: prepared transaction as bytes
/// - signature_spec: type of the signature transaction must be signed with (one of: `Ed25519` or `Secp256k1`)
/// - bytes_to_sign: bytes must be signed
/// - endorsement_spec: endorsement process specification
#[no_mangle]
pub extern "C" fn vdr_prepare_schema(
    command_handle: CommandHandle,
    vdr: *const c_void,
    txn_specific_params: *const c_char,
    submitter_did: *const c_char,
    endorser: *const c_char,
    cb: Option<extern "C" fn(
        command_handle_: CommandHandle,
        err: ErrorCode,
        namespace: *const c_char,
        txn_bytes_raw: *const u8,
        txn_bytes_len: u32,
        signature_spec: *const c_char,
        bytes_to_sign_raw: *const u8,
        bytes_to_sign_len: u32,
        endorsement_spec: *const c_char)>, ) -> ErrorCode {
    debug!(
        "vdr_prepare_schema > txn_specific_params {:?} submitter_did {:?} endorser {:?}",
        txn_specific_params, submitter_did, endorser
    );

    check_useful_c_reference!(vdr, VDR, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(txn_specific_params, ErrorCode::CommonInvalidParam3);
    check_useful_c_str!(submitter_did, ErrorCode::CommonInvalidParam4);
    check_useful_opt_c_str!(endorser, ErrorCode::CommonInvalidParam5);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    debug!(
        "vdr_prepare_schema ? txn_specific_params {:?} submitter_did {:?} endorser {:?}",
        txn_specific_params, submitter_did, endorser
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .vdr_controller
            .prepare_schema_txn(vdr, txn_specific_params, submitter_did, endorser)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, (namespace, txn_bytes, signature_spec, bytes_to_sign, endorsement_spec)) = prepare_result!(
            res, String::new(), Vec::new(), String::new(), Vec::new(), None
        );

        debug!(
            "vdr_prepare_schema ? err {:?} namespace {:?} signature_spec {:?} txn_bytes {:?} bytes_to_sign {:?} endorsement_spec {:?}",
            err, namespace, txn_bytes, signature_spec, bytes_to_sign, endorsement_spec);

        let namespace = ctypes::string_to_cstring(namespace);
        let signature_spec = ctypes::string_to_cstring(signature_spec);
        let (txn_data, txn_len) = ctypes::vec_to_pointer(&txn_bytes);
        let (bytes_data, bytes_len) = ctypes::vec_to_pointer(&bytes_to_sign);
        let endorsement_spec = endorsement_spec.map(ctypes::string_to_cstring);

        cb(
            command_handle,
            err,
            namespace.as_ptr(),
            txn_data,
            txn_len,
            signature_spec.as_ptr(),
            bytes_data,
            bytes_len,
            endorsement_spec
                .as_ref()
                .map(|vk| vk.as_ptr())
                .unwrap_or(ptr::null()),
        )
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::VdrCommandPrepareSchema, action, cb);

    let res = ErrorCode::Success;
    debug!("vdr_prepare_schema > {:?}", res);
    res
}

/// Prepare transaction to submit Credential Definition on the Ledger.
///
/// EXPERIMENTAL
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// vdr: pointer to VDR object
/// txn_specific_params: CredDef transaction specific data
///                      Depends on the Ledger type:
///     Indy:
///         {
///             id: string - identifier of credential definition
///             schemaId: string - identifier of stored in ledger schema
///             type: string - type of the credential definition. CL is the only supported type now.
///             tag: string - allows to distinct between credential definitions for the same issuer and schema
///             value: Dictionary with Credential Definition's data: {
///                 primary: primary credential public key,
///                 Optional<revocation>: revocation credential public key
///             },
///             ver: Version of the CredDef json
///         }
/// submitter_did: Fully-qualified DID of the transaction author as base58-encoded string.
/// endorser: DID of the Endorser that will endorse the transaction.
///           The Endorser's DID must be present on the ledger with 'ENDORSER' role.
/// cb: Callback that takes command result as parameter
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: command handle to map callback to caller context.
/// - err: Error code.
/// - namespace: Ledger namespace to submit transaction (captured from submitter DID)
/// - txn_bytes: prepared transaction as bytes
/// - signature_spec: type of the signature transaction must be signed with (one of: `Ed25519` or `Secp256k1`)
/// - bytes_to_sign: bytes must be signed
/// - endorsement_spec: endorsement process specification
#[no_mangle]
pub extern "C" fn vdr_prepare_cred_def(
    command_handle: CommandHandle,
    vdr: *const c_void,
    txn_specific_params: *const c_char,
    submitter_did: *const c_char,
    endorser: *const c_char,
    cb: Option<extern "C" fn(
        command_handle_: CommandHandle,
        err: ErrorCode,
        namespace: *const c_char,
        txn_bytes_raw: *const u8,
        txn_bytes_len: u32,
        signature_spec: *const c_char,
        bytes_to_sign_raw: *const u8,
        bytes_to_sign_len: u32,
        endorsement_spec: *const c_char)>,
) -> ErrorCode {
    debug!(
        "vdr_prepare_cred_def > txn_specific_params {:?} submitter_did {:?} endorser {:?}",
        txn_specific_params, submitter_did, endorser
    );

    check_useful_c_reference!(vdr, VDR, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(txn_specific_params, ErrorCode::CommonInvalidParam3);
    check_useful_c_str!(submitter_did, ErrorCode::CommonInvalidParam4);
    check_useful_opt_c_str!(endorser, ErrorCode::CommonInvalidParam5);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    debug!(
        "vdr_prepare_cred_def ? txn_specific_params {:?} submitter_did {:?} endorser {:?}",
        txn_specific_params, submitter_did, endorser
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .vdr_controller
            .prepare_creddef_txn(vdr, txn_specific_params, submitter_did, endorser)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, (namespace, txn_bytes, signature_spec, bytes_to_sign, endorsement_spec)) = prepare_result!(
            res, String::new(),  Vec::new(), String::new(),Vec::new(), None
        );

        debug!(
            "vdr_prepare_cred_def ? err {:?} namespace {:?} signature_spec {:?} txn_bytes {:?} bytes_to_sign {:?} endorsement_spec {:?}",
            err, namespace, signature_spec, txn_bytes, bytes_to_sign, endorsement_spec);

        let namespace = ctypes::string_to_cstring(namespace);
        let signature_spec = ctypes::string_to_cstring(signature_spec);
        let (txn_data, txn_len) = ctypes::vec_to_pointer(&txn_bytes);
        let (bytes_data, bytes_len) = ctypes::vec_to_pointer(&bytes_to_sign);
        let endorsement_spec = endorsement_spec.map(ctypes::string_to_cstring);

        cb(
            command_handle,
            err,
            namespace.as_ptr(),
            txn_data,
            txn_len,
            signature_spec.as_ptr(),
            bytes_data,
            bytes_len,
            endorsement_spec
                .as_ref()
                .map(|vk| vk.as_ptr())
                .unwrap_or(ptr::null()),
        )
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::VdrCommandPrepareCredDef, action, cb);

    let res = ErrorCode::Success;
    debug!("vdr_prepare_cred_def > {:?}", res);
    res
}


/// Submit transaction to the Ledger.
///
/// EXPERIMENTAL
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// vdr: pointer to VDR object
/// namespace of the registered Ledger to submit transaction
/// txn_bytes_raw: a pointer to first byte of transaction
/// txn_bytes_len: a transaction length
/// signature_spec: type of the signature used for transaction signing
/// signature_raw: a pointer to first byte of the transaction signature
/// signatures_len: a transaction signature length
/// endorsement: (Optional) transaction endorsement data (depends on the ledger type)
///     Indy:
///         {
///             "signature" - endorser signature as base58 string
///         }
///     Cheqd: TODO
/// cb: Callback that takes command result as parameter
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: command handle to map callback to caller context.
/// - err: Error code.
/// - response: received response
#[no_mangle]
pub extern "C" fn vdr_submit_txn(
    command_handle: CommandHandle,
    vdr: *const c_void,
    namespace: *const c_char,
    txn_bytes_raw: *const u8,
    txn_bytes_len: u32,
    signature_spec: *const c_char,
    signature_raw: *const u8,
    signature_len: u32,
    endorsement: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, response: *const c_char)>,
) -> ErrorCode {
    debug!(
        "vdr_submit_txn > namespace {:?} signature_spec {:?} txn_bytes_raw {:?} bytes_to_sign_len {:?} signature_raw {:?} signature_len {:?} endorsement {:?}",
        namespace, signature_spec, txn_bytes_raw, txn_bytes_len, signature_raw, signature_len, endorsement
    );

    check_useful_c_reference!(vdr, VDR, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(namespace, ErrorCode::CommonInvalidParam3);
    check_useful_c_byte_array!(
        txn_bytes_raw,
        txn_bytes_len,
        ErrorCode::CommonInvalidParam4,
        ErrorCode::CommonInvalidParam5
    );
    check_useful_c_str!(signature_spec, ErrorCode::CommonInvalidParam6);
    check_useful_c_byte_array!(
        signature_raw,
        signature_len,
        ErrorCode::CommonInvalidParam7,
        ErrorCode::CommonInvalidParam8
    );
    check_useful_opt_c_str!(endorsement, ErrorCode::CommonInvalidParam9);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam10);

    debug!(
        "vdr_submit_txn ? namespace {:?} txn_bytes_raw {:?} txn_bytes_len {:?} signature_raw {:?} signature_len {:?} endorsement {:?}",
        namespace, txn_bytes_raw, txn_bytes_len, signature_raw, signature_len, endorsement
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .vdr_controller
            .submit_txn(vdr, namespace, signature_spec, txn_bytes_raw, signature_raw, endorsement)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, response) = prepare_result!(res, String::new());

        debug!("vdr_submit_txn ? err {:?} response {:?}", err, response);

        let response = ctypes::string_to_cstring(response);

        cb(command_handle, err, response.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::VdrCommandSubmitTxn, action, cb);

    let res = ErrorCode::Success;
    debug!("vdr_submit_txn > {:?}", res);
    res
}

/// Submit raw transaction to the Ledger.
///
/// EXPERIMENTAL
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// vdr: pointer to VDR object
/// namespace of the registered Ledger to submit transaction
/// txn_bytes_raw: a pointer to first byte of transaction
/// txn_bytes_len: a transaction length
/// cb: Callback that takes command result as parameter
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: command handle to map callback to caller context.
/// - err: Error code.
/// - response: received response

#[no_mangle]
pub extern "C" fn vdr_submit_raw_txn(
    command_handle: CommandHandle,
    vdr: *const c_void,
    namespace: *const c_char,
    txn_bytes_raw: *const u8,
    txn_bytes_len: u32,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, response: *const c_char)>,
) -> ErrorCode {
    debug!(
        "vdr_submit_raw_txn > namespace {:?} txn_bytes_raw {:?} bytes_to_sign_len {:?}",
        namespace, txn_bytes_raw, txn_bytes_len
    );

    check_useful_c_reference!(vdr, VDR, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(namespace, ErrorCode::CommonInvalidParam3);
    check_useful_c_byte_array!(
        txn_bytes_raw,
        txn_bytes_len,
        ErrorCode::CommonInvalidParam4,
        ErrorCode::CommonInvalidParam5
    );
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    debug!(
        "vdr_submit_raw_txn ? namespace {:?} txn_bytes_raw {:?} txn_bytes_len {:?}",
        namespace, txn_bytes_raw, txn_bytes_len,
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .vdr_controller
            .submit_raw_txn(vdr, namespace, txn_bytes_raw)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, response) = prepare_result!(res, String::new());

        debug!("vdr_submit_raw_txn ? err {:?} response {:?}", err, response);

        let response = ctypes::string_to_cstring(response);

        cb(command_handle, err, response.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::VdrCommandSubmitTxn, action, cb);

    let res = ErrorCode::Success;
    debug!("vdr_submit_raw_txn > {:?}", res);
    res
}

/// Submit query to the Ledger.
///
/// EXPERIMENTAL
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// vdr: pointer to VDR object
/// namespace of the registered Ledger to submit transaction
/// query: query message to submit on the Ledger
/// cb: Callback that takes command result as parameter
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: command handle to map callback to caller context.
/// - err: Error code.
/// - response: received response
#[no_mangle]
pub extern "C" fn vdr_submit_query(
    command_handle: CommandHandle,
    vdr: *const c_void,
    namespace: *const c_char,
    query: *const c_char,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, response: *const c_char)>,
) -> ErrorCode {
    debug!(
        "vdr_submit_query > namespace {:?} query {:?}",
        namespace, query
    );

    check_useful_c_reference!(vdr, VDR, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(namespace, ErrorCode::CommonInvalidParam3);
    check_useful_c_str!(query, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    debug!(
        "vdr_submit_query ? namespace {:?} query {:?}",
        namespace, query
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .vdr_controller
            .submit_query(vdr, namespace, query)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, response) = prepare_result!(res, String::new());

        debug!("vdr_submit_query ? err {:?} response {:?}", err, response);

        let response = ctypes::string_to_cstring(response);

        cb(command_handle, err, response.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::VdrCommandSubmitQuery, action, cb);

    let res = ErrorCode::Success;
    debug!("vdr_submit_query > {:?}", res);
    res
}

/// Endorse Indy transaction (prepare and sign with endorser DID).
///
/// EXPERIMENTAL
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// wallet_handle: handle pointing to an opened wallet (returned by indy_open_wallet)
/// endorsement_data: data required for transaction endorsing
///     {
///         "did": string - DID to use for transaction signing
///     }
/// signature_spec: type of the signature used for transaction signing
/// txn_bytes_to_sign_raw: a pointer to first byte of transaction bytes to sign
/// txn_bytes_to_sign_len: a transaction length
/// cb: Callback that takes command result as parameter
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: command handle to map callback to caller context.
/// - err: Error code.
/// - endorsement: generated endorsement information
///         {
///             "signature": string - endorser transaction signature as baste58 string
///         }
#[no_mangle]
pub extern "C" fn vdr_indy_endorse(
    command_handle: CommandHandle,
    wallet_handle: WalletHandle,
    endorsement_data: *const c_char,
    signature_spec: *const c_char,
    txn_bytes_to_sign_raw: *const u8,
    txn_bytes_to_sign_len: u32,
    cb: Option<extern "C" fn(command_handle_: CommandHandle, err: ErrorCode, endorsement: *const c_char)>,
) -> ErrorCode {
    debug!(
        "vdr_indy_endorse > wallet_handle {:?} endorsement_data {:?} signature_spec {:?} txn_bytes_to_sign_raw {:?} \
        txn_bytes_to_sign_len {:?}",
        wallet_handle, endorsement_data, signature_spec, txn_bytes_to_sign_raw, txn_bytes_to_sign_len,
    );

    check_useful_c_str!(endorsement_data, ErrorCode::CommonInvalidParam3);
    check_useful_c_str!(signature_spec, ErrorCode::CommonInvalidParam4);
    check_useful_c_byte_array!(
        txn_bytes_to_sign_raw,
        txn_bytes_to_sign_len,
        ErrorCode::CommonInvalidParam5,
        ErrorCode::CommonInvalidParam6
    );
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam7);

    debug!(
        "vdr_indy_endorse ? wallet_handle {:?} endorsement_data {:?} signature_spec {:?} txn_bytes_to_sign_raw {:?} \
        txn_bytes_to_sign_len {:?}",
        wallet_handle, endorsement_data, signature_spec, txn_bytes_to_sign_raw, txn_bytes_to_sign_len,
    );

    let locator = Locator::instance();

    let action = async move {
        let res = locator
            .vdr_controller
            .indy_endorse(wallet_handle, endorsement_data, signature_spec, txn_bytes_to_sign_raw)
            .await;
        res
    };

    let cb = move |res: IndyResult<_>| {
        let (err, endorsement) = prepare_result!(res, String::new());

        debug!("vdr_indy_endorse ? err {:?} response {:?}", err, endorsement);

        let endorsement = ctypes::string_to_cstring(endorsement);

        cb(command_handle, err, endorsement.as_ptr())
    };

    locator.executor.spawn_ok_instrumented(CommandMetric::VdrCommandSubmitTxn, action, cb);

    let res = ErrorCode::Success;
    debug!("vdr_indy_endorse > {:?}", res);
    res
}
