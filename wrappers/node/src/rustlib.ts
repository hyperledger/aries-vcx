import * as ref from 'ref-napi';
import * as StructType from 'ref-struct-di';
import { ICbRef } from './utils/ffi-helpers';

import { VCXRuntime } from './vcx';

export const VcxStatus = StructType({
  handle: 'int',
  msg: 'string',
  status: 'int',
});

interface IUintTypes {
  [key: string]: string;
}
const UINTS_TYPES: IUintTypes = { x86: 'uint32', x64: 'uint64' };
const ARCHITECTURE: string = process.env.LIBVCX_FFI_ARCHITECTURE || 'x64';
const FFI_UINT: string = UINTS_TYPES[ARCHITECTURE];

// FFI Type Strings
export const FFI_ERROR_CODE = 'int';
export const FFI_BOOL = 'bool';
export const FFI_CONNECTION_HANDLE = 'uint32';
export const FFI_UNSIGNED_INT = 'uint32';
export const FFI_UNSIGNED_LONG = 'uint64';
export const FFI_UNSIGNED_INT_PTR = FFI_UINT;
export const FFI_STRING = 'string';
export const FFI_STRING_DATA = 'string';
export const FFI_SOURCE_ID = 'string';
export const FFI_CONNECTION_DATA = 'string';
export const FFI_VOID = ref.types.void;
export const FFI_CONNECTION_HANDLE_PTR = ref.refType(FFI_CONNECTION_HANDLE);
export const FFI_CALLBACK_PTR = 'pointer';
export const FFI_COMMAND_HANDLE = 'uint32';
export const FFI_CREDENTIAL_HANDLE = 'uint32';
export const FFI_PROOF_HANDLE = 'uint32';
export const FFI_CREDENTIALDEF_HANDLE = 'uint32';
export const FFI_SCHEMA_HANDLE = 'uint32';
export const FFI_OOB_HANDLE = 'uint32';
export const FFI_REV_REG_HANDLE = 'uint32';
export const FFI_AGENT_HANDLE = 'uint32';
export const FFI_PAYMENT_HANDLE = 'uint32';
export const FFI_POINTER = 'pointer';
export const FFI_VOID_POINTER = 'void *';

// Evernym extensions
export const FFI_INDY_NUMBER = 'int32';

export interface IFFIEntryPoint {
  vcx_open_main_pool: (commandId: number, config: string, cb: any) => number,

  vcx_create_agency_client_for_main_wallet: (commandId: number, config: string, cb: any) => number,
  vcx_provision_cloud_agent: (commandId: number, config: string, cb: any) => number,
  vcx_init_threadpool: (config: string) => number,
  vcx_init_issuer_config: (commandId: number, config: string, cb: any) => number,

  vcx_shutdown: (deleteIndyInfo: boolean) => number;
  vcx_error_c_message: (errorCode: number) => string;
  vcx_version: () => string;
  vcx_enable_mocks: () => void;
  vcx_v2_messages_download: (
    commandId: number,
    status: string,
    uids: string,
    pairwiseDids: string,
    cb: ICbRef,
  ) => number;
  vcx_messages_update_status: (
    commandId: number,
    status: string,
    msgIds: string,
    cb: ICbRef,
  ) => number;
  vcx_get_ledger_author_agreement: (commandId: number, cb: ICbRef) => number;
  vcx_set_active_txn_author_agreement_meta: (
    text: string | undefined | null,
    version: string | undefined | null,
    hash: string | undefined | null,
    accMechType: string,
    timeOfAcceptance: number,
  ) => number;

  // wallet
  vcx_create_wallet: (commandId: number, config: string, cb: ICbRef) => number,
  vcx_configure_issuer_wallet: (commandId: number, seed: string, cb: ICbRef) => number,
  vcx_open_main_wallet: (commandId: number, config: string, cb: ICbRef) => number,
  vcx_close_main_wallet: (commandId: number, cb: ICbRef) => number,

  vcx_wallet_add_record: (
    commandId: number,
    type: string,
    id: string,
    value: string,
    tags: string,
    cb: ICbRef,
  ) => number;
  vcx_wallet_update_record_value: (
    commandId: number,
    type: string,
    id: string,
    value: string,
    cb: ICbRef,
  ) => number;
  vcx_wallet_update_record_tags: (
    commandId: number,
    type: string,
    id: string,
    tags: string,
    cb: ICbRef,
  ) => number;
  vcx_wallet_add_record_tags: (
    commandId: number,
    type: string,
    id: string,
    tags: string,
    cb: ICbRef,
  ) => number;
  vcx_wallet_delete_record_tags: (
    commandId: number,
    type: string,
    id: string,
    tagsList: string,
    cb: ICbRef,
  ) => number;
  vcx_wallet_delete_record: (commandId: number, type: string, id: string, cb: ICbRef) => number;
  vcx_wallet_get_record: (
    commandId: number,
    type: string,
    id: string,
    options: string,
    cb: ICbRef,
  ) => number;
  vcx_wallet_open_search: (
    commandId: number,
    type: string,
    query: string,
    options: string,
    cb: ICbRef,
  ) => number;
  vcx_wallet_close_search: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_wallet_search_next_records: (
    commandId: number,
    handle: number,
    count: number,
    cb: ICbRef,
  ) => number;
  vcx_wallet_set_handle: (handle: number) => void;
  vcx_wallet_import: (commandId: number, config: string, cb: ICbRef) => number;
  vcx_wallet_export: (
    commandId: number,
    importPath: string,
    backupKey: string,
    cb: ICbRef,
  ) => number;
  vcx_update_webhook_url: (commandId: number, webhookUrl: string, cb: ICbRef) => number;
  vcx_pool_set_handle: (handle: number) => void;
  vcx_endorse_transaction: (commandId: number, transaction: string, cb: ICbRef) => number;
  vcx_rotate_verkey: (commandId: number, did: string, cb: ICbRef) => number;
  vcx_rotate_verkey_start: (commandId: number, did: string, cb: ICbRef) => number;
  vcx_rotate_verkey_apply: (commandId: number, did: string, tempVk: string, cb: ICbRef) => number;
  vcx_get_verkey_from_wallet: (commandId: number, did: string, cb: ICbRef) => number;
  vcx_get_verkey_from_ledger: (commandId: number, did: string, cb: ICbRef) => number;
  vcx_get_ledger_txn: (commandId: number, did: string, seq_no: number, cb: ICbRef) => number;

  // connection
  vcx_connection_delete_connection: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_connection_connect: (commandId: number, handle: number, data: string, cb: ICbRef) => number;
  vcx_connection_create: (commandId: number, data: string, cb: ICbRef) => number;
  vcx_connection_create_with_invite: (
    commandId: number,
    data: string,
    invite: string,
    cb: ICbRef,
  ) => number;
  vcx_connection_create_with_connection_request: (
    commandId: number,
    sourceId: string,
    agentHandle: number,
    request: string,
    cb: ICbRef,
  ) => number;
  vcx_connection_deserialize: (commandId: number, data: string, cb: ICbRef) => number;
  vcx_connection_release: (handle: number) => number;
  vcx_connection_serialize: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_connection_update_state: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_connection_update_state_with_message: (
    commandId: number,
    handle: number,
    message: string,
    cb: ICbRef,
  ) => number;
  vcx_connection_handle_message: (
      commandId: number,
      handle: number,
      message: string,
      cb: ICbRef,
  ) => number;
  vcx_connection_get_state: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_connection_invite_details: (
    commandId: number,
    handle: number,
    abbreviated: boolean,
    cb: ICbRef,
  ) => number;
  vcx_connection_send_message: (
    commandId: number,
    handle: number,
    msg: string,
    sendMsgOptions: string,
    cb: ICbRef,
  ) => number;
  vcx_connection_send_handshake_reuse: (
    commandId: number,
    handle: number,
    oob_id: string,
    cb: ICbRef,
  ) => number;
  vcx_connection_sign_data: (
    commandId: number,
    handle: number,
    data: number,
    dataLength: number,
    cb: ICbRef,
  ) => number;
  vcx_connection_verify_signature: (
    commandId: number,
    handle: number,
    data: number,
    dataLength: number,
    signature: number,
    signatureLength: number,
    cb: ICbRef,
  ) => number;
  vcx_connection_send_ping: (
    commandId: number,
    handle: number,
    comment: string | null | undefined,
    cb: ICbRef,
  ) => number;
  vcx_connection_send_discovery_features: (
    commandId: number,
    handle: number,
    query: string | null | undefined,
    comment: string | null | undefined,
    cb: ICbRef,
  ) => number;
  vcx_connection_get_pw_did: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_connection_get_their_pw_did: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_connection_info: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_connection_get_thread_id: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_connection_messages_download: (
    commandId: number,
    handle: number,
    status: string,
    uids: string,
    cb: ICbRef,
  ) => number;

  // issuer
  vcx_issuer_credential_release: (handle: number) => number;
  vcx_issuer_credential_deserialize: (commandId: number, data: string, cb: ICbRef) => number;
  vcx_issuer_credential_serialize: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_issuer_credential_get_thread_id: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_v2_issuer_credential_update_state: (
    commandId: number,
    handle: number,
    connHandle: number,
    cb: ICbRef,
  ) => number;
  vcx_v2_issuer_credential_update_state_with_message: (
    commandId: number,
    handle: number,
    connHandle: number,
    msg: string,
    cb: ICbRef,
  ) => number;
  vcx_issuer_credential_get_state: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_issuer_credential_get_rev_reg_id: (
    commandId: number,
    credentialHandle: number,
    cb: ICbRef,
  ) => number;
  vcx_issuer_create_credential: (
    commandId: number,
    sourceId: string,
    cb: ICbRef,
  ) => number;
  vcx_issuer_revoke_credential_local: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_issuer_credential_is_revokable: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_issuer_send_credential: (
    commandId: number,
    credentialHandle: number,
    connectionHandle: number,
    cb: ICbRef,
  ) => number;
  vcx_issuer_get_credential_msg: (
    commandId: number,
    credentialHandle: number,
    myPwDid: string,
    cb: ICbRef,
  ) => number;
  vcx_issuer_send_credential_offer_v2: (
      commandId: number,
      credentialHandle: number,
      connectionHandle: number,
      cb: ICbRef,
  ) => number;
  vcx_mark_credential_offer_msg_sent: (
      commandId: number,
      credentialHandle: number,
      cb: ICbRef,
  ) => number;
  vcx_issuer_build_credential_offer_msg_v2: (
      commandId: number,
      credentialHandle: number,
      credentialDefHandle: number,
      revRegHandle: number,
      credentialData: string,
      comment: string,
      cb: ICbRef,
  ) => number;
  vcx_issuer_get_credential_offer_msg: (
    commandId: number,
    credentialHandle: number,
    cb: ICbRef,
  ) => number;

  // proof
  vcx_proof_create: (
    commandId: number,
    sourceId: string,
    attrs: string,
    predicates: string,
    revocationInterval: string,
    name: string,
    cb: ICbRef,
  ) => number;
  vcx_proof_deserialize: (commandId: number, data: string, cb: ICbRef) => number;
  vcx_get_proof_msg: (
    commandId: number,
    proofHandle: number,
    cb: ICbRef,
  ) => number;
  vcx_proof_release: (handle: number) => number;
  vcx_proof_send_request: (
    commandId: number,
    proofHandle: number,
    connectionHandle: number,
    cb: ICbRef,
  ) => number;
  vcx_proof_get_request_msg: (commandId: number, proofHandle: number, cb: ICbRef) => number;
  vcx_proof_serialize: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_v2_proof_update_state: (
    commandId: number,
    handle: number,
    connHandle: number,
    cb: ICbRef,
  ) => number;
  vcx_v2_proof_update_state_with_message: (
    commandId: number,
    handle: number,
    connHandle: number,
    msg: string,
    cb: ICbRef,
  ) => number;
  vcx_proof_get_state: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_proof_get_thread_id: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_mark_presentation_request_msg_sent: (
      commandId: number,
      proofHandle: number,
      cb: ICbRef,
  ) => number;


  // disclosed proof
  vcx_disclosed_proof_create_with_request: (
    commandId: number,
    sourceId: string,
    req: string,
    cb: ICbRef,
  ) => number;
  vcx_disclosed_proof_create_with_msgid: (
    commandId: number,
    sourceId: string,
    connectionHandle: number,
    msgId: string,
    cb: ICbRef,
  ) => number;
  vcx_disclosed_proof_release: (handle: number) => number;
  vcx_disclosed_proof_send_proof: (
    commandId: number,
    proofHandle: number,
    connectionHandle: number,
    cb: ICbRef,
  ) => number;
  vcx_disclosed_proof_reject_proof: (
    commandId: number,
    proofHandle: number,
    connectionHandle: number,
    cb: ICbRef,
  ) => number;
  vcx_disclosed_proof_get_proof_msg: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_disclosed_proof_get_reject_msg: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_disclosed_proof_serialize: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_disclosed_proof_deserialize: (commandId: number, data: string, cb: ICbRef) => number;
  vcx_v2_disclosed_proof_update_state: (
    commandId: number,
    handle: number,
    connHandle: number,
    cb: ICbRef,
  ) => number;
  vcx_disclosed_proof_get_state: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_disclosed_proof_get_requests: (
    commandId: number,
    connectionHandle: number,
    cb: ICbRef,
  ) => number;
  vcx_disclosed_proof_retrieve_credentials: (
    commandId: number,
    handle: number,
    cb: ICbRef,
  ) => number;
  vcx_disclosed_proof_get_proof_request_attachment: (
    commandId: number,
    handle: number,
    cb: ICbRef,
  ) => number;
  vcx_disclosed_proof_generate_proof: (
    commandId: number,
    handle: number,
    selectedCreds: string,
    selfAttestedAttrs: string,
    cb: ICbRef,
  ) => number;
  vcx_disclosed_proof_decline_presentation_request: (
    commandId: number,
    handle: number,
    connectionHandle: number,
    reason: string | undefined | null,
    proposal: string | undefined | null,
    cb: ICbRef,
  ) => number;
  vcx_disclosed_proof_get_thread_id: (commandId: number, handle: number, cb: ICbRef) => number;

  // credential
  vcx_credential_create_with_offer: (
    commandId: number,
    sourceId: string,
    offer: string,
    cb: ICbRef,
  ) => number;
  vcx_credential_create_with_msgid: (
    commandId: number,
    sourceId: string,
    connectionHandle: number,
    msgId: string,
    cb: ICbRef,
  ) => number;
  vcx_credential_release: (handle: number) => number;
  vcx_credential_send_request: (
    commandId: number,
    handle: number,
    connectionHandle: number,
    payment: number,
    cb: ICbRef,
  ) => number;
  vcx_credential_get_request_msg: (
    commandId: number,
    handle: number,
    myPwDid: string,
    theirPwDid: string | undefined | null,
    payment: number,
    cb: ICbRef,
  ) => number;
  vcx_credential_decline_offer: (
    commandId: number,
    handle: number,
    connectionHandle: number,
    comment: string,
    cb: ICbRef,
  ) => number;
  vcx_credential_serialize: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_credential_deserialize: (commandId: number, data: string, cb: ICbRef) => number;
  vcx_v2_credential_update_state: (
    commandId: number,
    handle: number,
    connHandle: number,
    cb: ICbRef,
  ) => number;
  vcx_credential_get_state: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_credential_get_offers: (commandId: number, connectionHandle: number, cb: ICbRef) => number;
  vcx_credential_get_attributes: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_credential_get_attachment: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_credential_get_tails_location: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_credential_get_tails_hash: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_credential_get_rev_reg_id: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_credential_get_thread_id: (commandId: number, handle: number, cb: ICbRef) => number;

  // logger
  vcx_set_default_logger: (level: string) => number;
  vcx_set_logger: (context: Buffer, enabled: ICbRef, logFn: ICbRef, flush: ICbRef) => number;

  // mock
  vcx_set_next_agency_response: (messageIndex: number) => void;

  // credentialdef
  vcx_credentialdef_create_v2: (
    commandId: number,
    sourceId: string,
    schemaId: string,
    issuerDid: string | null,
    tag: string,
    support_revocation: boolean,
    cb: ICbRef,
  ) => number;
  vcx_credentialdef_publish: (commandId: number, handle: number, tailsUrl: string | null, cb: ICbRef) => number;
  vcx_credentialdef_deserialize: (commandId: number, data: string, cb: ICbRef) => number;
  vcx_credentialdef_serialize: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_credentialdef_release: (handle: number) => number;
  vcx_credentialdef_get_cred_def_id: (commandId: number, handle: number, cb: ICbRef) => string;
  vcx_credentialdef_update_state: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_credentialdef_get_state: (commandId: number, handle: number, cb: ICbRef) => number;

  // schema
  vcx_schema_get_attributes: (
    commandId: number,
    sourceId: string,
    schemaId: string,
    cb: ICbRef,
  ) => number;
  vcx_schema_create: (
    commandId: number,
    sourceId: string,
    schemaName: string,
    version: string,
    schemaData: string,
    paymentHandle: number,
    cb: ICbRef,
  ) => number;
  vcx_schema_prepare_for_endorser: (
    commandId: number,
    sourceId: string,
    schemaName: string,
    version: string,
    schemaData: string,
    endorser: string,
    cb: ICbRef,
  ) => number;
  vcx_schema_get_schema_id: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_schema_deserialize: (commandId: number, data: string, cb: ICbRef) => number;
  vcx_schema_serialize: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_schema_release: (handle: number) => number;
  vcx_schema_update_state: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_schema_get_state: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_public_agent_create: (commandId: number, sourceId: string, institutionDid: string, cb: ICbRef) => number;
  vcx_generate_public_invite: (commandId: number, public_did: string, label: string, cb: ICbRef) => number;
  vcx_public_agent_download_connection_requests: (commandId: number, handle: number, uids: string, cb: ICbRef) => number;
  vcx_public_agent_download_message: (commandId: number, handle: number, uid: string, cb: ICbRef) => number;
  vcx_public_agent_get_service: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_public_agent_serialize: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_public_agent_deserialize: (commandId: number, data: string, cb: ICbRef) => number;
  vcx_public_agent_release: (handle: number) => number;
  vcx_out_of_band_sender_create: (commandId: number, config: string, cb: ICbRef) => number;
  vcx_out_of_band_receiver_create: (commandId: number, msg: string, cb: ICbRef) => number;
  vcx_out_of_band_sender_append_message: (commandId: number, handle: number, message: string, cb: ICbRef) => number;
  vcx_out_of_band_sender_append_service: (commandId: number, handle: number, service: string, cb: ICbRef) => number;
  vcx_out_of_band_sender_append_service_did: (commandId: number, handle: number, did: string, cb: ICbRef) => number;
  vcx_out_of_band_sender_get_thread_id: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_out_of_band_receiver_get_thread_id: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_out_of_band_receiver_extract_message: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_out_of_band_to_message: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_out_of_band_sender_serialize: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_out_of_band_sender_deserialize: (commandId: number, data: string, cb: ICbRef) => number;
  vcx_out_of_band_receiver_serialize: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_out_of_band_receiver_deserialize: (commandId: number, data: string, cb: ICbRef) => number;
  vcx_out_of_band_sender_release: (handle: number) => number;
  vcx_out_of_band_receiver_release: (handle: number) => number;
  vcx_out_of_band_receiver_connection_exists: (commandId: number, handle: number, handles: string, cb: ICbRef) => number;
  vcx_out_of_band_receiver_build_connection: (commandId: number, handle: number, cb: ICbRef) => number;

  vcx_revocation_registry_create: (commandId: number, config: string, cb: ICbRef) => number;
  vcx_revocation_registry_publish: (commandId: number, handle: number, tailsUrl: string, cb: ICbRef) => number;
  vcx_revocation_registry_publish_revocations: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_revocation_registry_get_rev_reg_id: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_revocation_registry_get_tails_hash: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_revocation_registry_deserialize: (commandId: number, data: string, cb: ICbRef) => number;
  vcx_revocation_registry_serialize: (commandId: number, handle: number, cb: ICbRef) => number;
  vcx_revocation_registry_release: (handle: number) => number;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const FFIConfiguration: { [Key in keyof IFFIEntryPoint]: any } = {
  vcx_init_threadpool: [FFI_ERROR_CODE, [FFI_STRING_DATA]],
  vcx_enable_mocks: [FFI_ERROR_CODE, []],
  vcx_init_issuer_config: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_create_agency_client_for_main_wallet: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_provision_cloud_agent: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],

  vcx_open_main_pool: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],

  vcx_create_wallet: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_open_main_wallet: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_close_main_wallet: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CALLBACK_PTR]],
  vcx_configure_issuer_wallet: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_shutdown: [FFI_ERROR_CODE, [FFI_BOOL]],
  vcx_error_c_message: [FFI_STRING, [FFI_ERROR_CODE]],
  vcx_version: [FFI_STRING, []],
  vcx_update_webhook_url: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_v2_messages_download: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_STRING_DATA, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_messages_update_status: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_get_ledger_author_agreement: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CALLBACK_PTR]],
  vcx_set_active_txn_author_agreement_meta: [
    FFI_ERROR_CODE,
    [FFI_STRING_DATA, FFI_STRING_DATA, FFI_STRING_DATA, FFI_STRING_DATA, FFI_UNSIGNED_LONG],
  ],
  vcx_endorse_transaction: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_rotate_verkey: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_rotate_verkey_start: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_rotate_verkey_apply: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_get_verkey_from_wallet: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_get_verkey_from_ledger: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_get_ledger_txn: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_INDY_NUMBER, FFI_CALLBACK_PTR],
  ],

  // wallet
  vcx_wallet_set_handle: [FFI_INDY_NUMBER, [FFI_INDY_NUMBER]],
  vcx_pool_set_handle: [FFI_INDY_NUMBER, [FFI_INDY_NUMBER]],
  vcx_wallet_add_record: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_CALLBACK_PTR],
  ],
  vcx_wallet_update_record_value: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING, FFI_STRING, FFI_STRING, FFI_CALLBACK_PTR],
  ],
  vcx_wallet_update_record_tags: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING, FFI_STRING, FFI_STRING, FFI_CALLBACK_PTR],
  ],
  vcx_wallet_add_record_tags: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING, FFI_STRING, FFI_STRING, FFI_CALLBACK_PTR],
  ],
  vcx_wallet_delete_record_tags: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING, FFI_STRING, FFI_STRING, FFI_CALLBACK_PTR],
  ],
  vcx_wallet_delete_record: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING, FFI_STRING, FFI_CALLBACK_PTR],
  ],
  vcx_wallet_get_record: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING, FFI_STRING, FFI_STRING, FFI_CALLBACK_PTR],
  ],
  vcx_wallet_open_search: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING, FFI_STRING, FFI_STRING, FFI_CALLBACK_PTR],
  ],
  vcx_wallet_close_search: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_COMMAND_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_wallet_search_next_records: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_COMMAND_HANDLE, FFI_COMMAND_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_wallet_import: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING, FFI_CALLBACK_PTR]],
  vcx_wallet_export: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING, FFI_STRING, FFI_CALLBACK_PTR],
  ],

  // connection
  vcx_connection_delete_connection: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_connection_connect: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CONNECTION_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_connection_create: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_connection_create_with_invite: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_connection_create_with_connection_request: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_AGENT_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_connection_deserialize: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_connection_release: [FFI_ERROR_CODE, [FFI_CONNECTION_HANDLE]],
  vcx_connection_serialize: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_connection_update_state: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_connection_update_state_with_message: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_connection_handle_message: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_connection_get_state: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_connection_invite_details: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_BOOL, FFI_CALLBACK_PTR],
  ],
  vcx_connection_send_message: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_STRING_DATA, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_connection_send_handshake_reuse: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_connection_sign_data: [
    FFI_ERROR_CODE,
    [
      FFI_COMMAND_HANDLE,
      FFI_CONNECTION_HANDLE,
      FFI_UNSIGNED_INT_PTR,
      FFI_UNSIGNED_INT,
      FFI_CALLBACK_PTR,
    ],
  ],
  vcx_connection_verify_signature: [
    FFI_ERROR_CODE,
    [
      FFI_COMMAND_HANDLE,
      FFI_CONNECTION_HANDLE,
      FFI_UNSIGNED_INT_PTR,
      FFI_UNSIGNED_INT,
      FFI_UNSIGNED_INT_PTR,
      FFI_UNSIGNED_INT,
      FFI_CALLBACK_PTR,
    ],
  ],
  vcx_connection_send_ping: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_connection_send_discovery_features: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_STRING_DATA, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_connection_get_pw_did: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_connection_get_their_pw_did: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_connection_info: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_connection_messages_download: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_STRING_DATA, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_connection_get_thread_id: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR],
  ],

  // issuer
  vcx_issuer_credential_deserialize: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_issuer_credential_serialize: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_v2_issuer_credential_update_state: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_v2_issuer_credential_update_state_with_message: [
    FFI_ERROR_CODE,
    [
      FFI_COMMAND_HANDLE,
      FFI_CREDENTIAL_HANDLE,
      FFI_CONNECTION_HANDLE,
      FFI_STRING_DATA,
      FFI_CALLBACK_PTR
    ],
  ],
  vcx_issuer_credential_get_state: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_issuer_credential_get_rev_reg_id: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_issuer_create_credential: [
    FFI_ERROR_CODE,
    [
      FFI_COMMAND_HANDLE,
      FFI_SOURCE_ID,
      FFI_CALLBACK_PTR,
    ],
  ],
  vcx_issuer_revoke_credential_local: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_issuer_credential_is_revokable: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_issuer_send_credential: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_issuer_get_credential_msg: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_issuer_send_credential_offer_v2: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_mark_credential_offer_msg_sent: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_issuer_build_credential_offer_msg_v2: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CREDENTIALDEF_HANDLE, FFI_REV_REG_HANDLE, FFI_STRING_DATA, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_issuer_get_credential_offer_msg: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_issuer_credential_release: [FFI_ERROR_CODE, [FFI_CREDENTIAL_HANDLE]],
  vcx_issuer_credential_get_thread_id: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CALLBACK_PTR],
  ],

  // proof
  vcx_proof_create: [
    FFI_ERROR_CODE,
    [
      FFI_COMMAND_HANDLE,
      FFI_SOURCE_ID,
      FFI_STRING_DATA,
      FFI_STRING_DATA,
      FFI_STRING_DATA,
      FFI_STRING_DATA,
      FFI_CALLBACK_PTR,
    ],
  ],
  vcx_proof_deserialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_get_proof_msg: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_proof_release: [FFI_ERROR_CODE, [FFI_PROOF_HANDLE]],
  vcx_proof_send_request: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_proof_get_request_msg: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_proof_serialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CALLBACK_PTR]],
  vcx_v2_proof_update_state: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_v2_proof_update_state_with_message: [
    FFI_ERROR_CODE,
    [
      FFI_COMMAND_HANDLE,
      FFI_PROOF_HANDLE,
      FFI_CONNECTION_HANDLE,
      FFI_STRING_DATA,
      FFI_CALLBACK_PTR,
    ],
  ],
  vcx_proof_get_state: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CALLBACK_PTR]],
  vcx_proof_get_thread_id: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_mark_presentation_request_msg_sent: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CALLBACK_PTR],
  ],

  // disclosed proof
  vcx_disclosed_proof_create_with_request: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_SOURCE_ID, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_disclosed_proof_create_with_msgid: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_SOURCE_ID, FFI_CONNECTION_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_disclosed_proof_release: [FFI_ERROR_CODE, [FFI_PROOF_HANDLE]],
  vcx_disclosed_proof_send_proof: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_disclosed_proof_reject_proof: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_disclosed_proof_get_proof_msg: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_disclosed_proof_get_reject_msg: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_disclosed_proof_serialize: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_disclosed_proof_deserialize: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_v2_disclosed_proof_update_state: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_disclosed_proof_get_state: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_disclosed_proof_get_requests: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_disclosed_proof_retrieve_credentials: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_disclosed_proof_get_proof_request_attachment: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_disclosed_proof_generate_proof: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_STRING_DATA, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_disclosed_proof_decline_presentation_request: [
    FFI_ERROR_CODE,
    [
      FFI_COMMAND_HANDLE,
      FFI_PROOF_HANDLE,
      FFI_CONNECTION_HANDLE,
      FFI_STRING_DATA,
      FFI_STRING_DATA,
      FFI_CALLBACK_PTR,
    ],
  ],
  vcx_disclosed_proof_get_thread_id: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CALLBACK_PTR],
  ],

  // credential
  vcx_credential_create_with_offer: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_SOURCE_ID, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_credential_create_with_msgid: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_SOURCE_ID, FFI_CONNECTION_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_credential_release: [FFI_ERROR_CODE, [FFI_CREDENTIAL_HANDLE]],
  vcx_credential_send_request: [
    FFI_ERROR_CODE,
    [
      FFI_COMMAND_HANDLE,
      FFI_CREDENTIAL_HANDLE,
      FFI_CONNECTION_HANDLE,
      FFI_PAYMENT_HANDLE,
      FFI_CALLBACK_PTR,
    ],
  ],
  vcx_credential_decline_offer: [
    FFI_ERROR_CODE,
    [
      FFI_COMMAND_HANDLE,
      FFI_CREDENTIAL_HANDLE,
      FFI_CONNECTION_HANDLE,
      FFI_STRING_DATA,
      FFI_CALLBACK_PTR,
    ],
  ],
  vcx_credential_get_request_msg: [
    FFI_ERROR_CODE,
    [
      FFI_COMMAND_HANDLE,
      FFI_CREDENTIAL_HANDLE,
      FFI_STRING_DATA,
      FFI_STRING_DATA,
      FFI_PAYMENT_HANDLE,
      FFI_CALLBACK_PTR,
    ],
  ],
  vcx_credential_serialize: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_credential_deserialize: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_v2_credential_update_state: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_credential_get_state: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_credential_get_offers: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_credential_get_attributes: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_credential_get_attachment: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_credential_get_tails_location: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_credential_get_tails_hash: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_credential_get_rev_reg_id: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_credential_get_thread_id: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CALLBACK_PTR],
  ],

  // credentialDef
  vcx_credentialdef_create_v2: [
    FFI_ERROR_CODE,
    [
      FFI_COMMAND_HANDLE,
      FFI_SOURCE_ID,
      FFI_STRING_DATA,
      FFI_STRING_DATA,
      FFI_STRING_DATA,
      FFI_BOOL,
      FFI_CALLBACK_PTR,
    ],
  ],
  vcx_credentialdef_publish: [
    FFI_ERROR_CODE,
    [
      FFI_COMMAND_HANDLE,
      FFI_CREDENTIALDEF_HANDLE,
      FFI_STRING_DATA,
      FFI_CALLBACK_PTR,
    ],
  ],
  vcx_credentialdef_deserialize: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_credentialdef_release: [FFI_ERROR_CODE, [FFI_CREDENTIALDEF_HANDLE]],
  vcx_credentialdef_serialize: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CREDENTIALDEF_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_credentialdef_get_cred_def_id: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CREDENTIALDEF_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_credentialdef_update_state: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_credentialdef_get_state: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CALLBACK_PTR],
  ],

  // logger
  vcx_set_default_logger: [FFI_ERROR_CODE, [FFI_STRING]],
  vcx_set_logger: [
    FFI_ERROR_CODE,
    [FFI_VOID_POINTER, FFI_CALLBACK_PTR, FFI_CALLBACK_PTR, FFI_CALLBACK_PTR],
  ],

  // mock
  vcx_set_next_agency_response: [FFI_VOID, [FFI_UNSIGNED_INT]],

  // schema
  vcx_schema_get_attributes: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_SOURCE_ID, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_schema_create: [
    FFI_ERROR_CODE,
    [
      FFI_COMMAND_HANDLE,
      FFI_SOURCE_ID,
      FFI_STRING_DATA,
      FFI_STRING_DATA,
      FFI_STRING_DATA,
      FFI_PAYMENT_HANDLE,
      FFI_CALLBACK_PTR,
    ],
  ],
  vcx_schema_prepare_for_endorser: [
    FFI_ERROR_CODE,
    [
      FFI_COMMAND_HANDLE,
      FFI_SOURCE_ID,
      FFI_STRING_DATA,
      FFI_STRING_DATA,
      FFI_STRING_DATA,
      FFI_STRING_DATA,
      FFI_CALLBACK_PTR,
    ],
  ],
  vcx_schema_get_schema_id: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_SCHEMA_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_schema_deserialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_schema_release: [FFI_ERROR_CODE, [FFI_SCHEMA_HANDLE]],
  vcx_schema_serialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_SCHEMA_HANDLE, FFI_CALLBACK_PTR]],
  vcx_schema_update_state: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_schema_get_state: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_public_agent_create: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_generate_public_invite: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_public_agent_download_connection_requests: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_AGENT_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_public_agent_download_message: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_AGENT_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_public_agent_get_service: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_AGENT_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_public_agent_serialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_AGENT_HANDLE, FFI_CALLBACK_PTR]],
  vcx_public_agent_deserialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_public_agent_release: [FFI_ERROR_CODE, [FFI_CONNECTION_HANDLE]],
  vcx_out_of_band_sender_create: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_out_of_band_receiver_create: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_out_of_band_sender_append_message: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_OOB_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_out_of_band_sender_append_service: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_OOB_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_out_of_band_sender_append_service_did: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_OOB_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_out_of_band_sender_get_thread_id: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_OOB_HANDLE, FFI_CALLBACK_PTR]],
  vcx_out_of_band_receiver_get_thread_id: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_OOB_HANDLE, FFI_CALLBACK_PTR]],
  vcx_out_of_band_receiver_extract_message: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_OOB_HANDLE, FFI_CALLBACK_PTR]],
  vcx_out_of_band_to_message: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_OOB_HANDLE, FFI_CALLBACK_PTR]],
  vcx_out_of_band_sender_serialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_OOB_HANDLE, FFI_CALLBACK_PTR]],
  vcx_out_of_band_sender_deserialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_out_of_band_receiver_serialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_OOB_HANDLE, FFI_CALLBACK_PTR]],
  vcx_out_of_band_receiver_deserialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_out_of_band_sender_release: [FFI_ERROR_CODE, [FFI_OOB_HANDLE]],
  vcx_out_of_band_receiver_release: [FFI_ERROR_CODE, [FFI_OOB_HANDLE]],
  vcx_out_of_band_receiver_connection_exists: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_OOB_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_out_of_band_receiver_build_connection: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_OOB_HANDLE, FFI_CALLBACK_PTR]],
  vcx_revocation_registry_create: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_revocation_registry_publish: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_REV_REG_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR],
  ],
  vcx_revocation_registry_publish_revocations: [
    FFI_ERROR_CODE,
    [FFI_COMMAND_HANDLE, FFI_REV_REG_HANDLE, FFI_CALLBACK_PTR],
  ],
  vcx_revocation_registry_get_rev_reg_id: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_REV_REG_HANDLE, FFI_CALLBACK_PTR]],
  vcx_revocation_registry_get_tails_hash: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_REV_REG_HANDLE, FFI_CALLBACK_PTR]],
  vcx_revocation_registry_serialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_REV_REG_HANDLE, FFI_CALLBACK_PTR]],
  vcx_revocation_registry_deserialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_revocation_registry_release: [FFI_ERROR_CODE, [FFI_REV_REG_HANDLE]],
};

let _rustAPI: IFFIEntryPoint;
let wasInitialized = false;
export const initRustAPI = (path?: string): IFFIEntryPoint => {
  if (wasInitialized) {
    throw new Error(
      'initRustAPI was already initialized. Make sure you only call it once in the lifetime of the process.',
    );
  }
  _rustAPI = new VCXRuntime({ basepath: path }).ffi;
  wasInitialized = true;
  return _rustAPI;
};
export const rustAPI = (): IFFIEntryPoint => {
  if (!_rustAPI) {
    throw new Error('RustAPI not loaded. Make sure you are calling initRustAPI(...)');
  }
  return _rustAPI;
};

export const isRustApiInitialized = (): boolean => Boolean(_rustAPI);
