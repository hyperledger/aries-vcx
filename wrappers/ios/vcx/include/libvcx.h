#ifndef __VCX_H
#define __VCX_H

#ifdef __cplusplus
extern "C" {
#endif

#import "IndySdk.h"
#import "VcxTypes.h"

/**
 * Initialize the SDK
 */

vcx_error_t vcx_init_issuer_config(
        vcx_command_handle_t handle,
        const char *config,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err)
);

vcx_error_t vcx_pool_set_handle(vcx_i32_t handle);

vcx_error_t vcx_endorse_transaction(
        vcx_command_handle_t handle,
        const char *transaction,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err)
);

vcx_error_t vcx_rotate_verkey(
        vcx_command_handle_t handle,
        const char *did,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err)
);

vcx_error_t vcx_rotate_verkey_start(
        vcx_command_handle_t handle,
        const char *did,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err)
);

vcx_error_t vcx_rotate_verkey_apply(
        vcx_command_handle_t handle,
        const char *did,
        const char *tmp_vk,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err)
);

vcx_error_t vcx_get_verkey_from_wallet(
        vcx_command_handle_t handle,
        const char *did,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, const char *verKey)
);

vcx_error_t vcx_get_verkey_from_ledger(
        vcx_command_handle_t handle,
        const char *did,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, const char *verKey)
);

vcx_error_t vcx_get_ledger_txn(
        vcx_command_handle_t handle,
        const char *submitter_did,
        vcx_i32_t seq_no,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, const char *txn)
);

vcx_error_t vcx_init_threadpool(const char *config);

vcx_error_t vcx_open_main_pool(
        vcx_command_handle_t handle,
        const char *config,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err)
);

vcx_error_t vcx_create_wallet(
        vcx_command_handle_t handle,
        const char *config,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err)
);

vcx_error_t vcx_configure_issuer_wallet(
        vcx_command_handle_t handle,
        const char *seed,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, const char *conf)
);

vcx_error_t vcx_open_main_wallet(
        vcx_command_handle_t handle,
        const char *config,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, VcxHandle handle)
);

vcx_error_t vcx_close_main_wallet(
        vcx_command_handle_t handle,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err)
);

vcx_error_t vcx_update_webhook_url(
        vcx_command_handle_t handle,
        const char *notification_webhook_url,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err)
);

vcx_error_t vcx_create_agency_client_for_main_wallet(
        vcx_command_handle_t handle,
        const char *config,
        void (*cb)(vcx_command_handle_t xhandle, vcx_error_t err)
);

vcx_error_t vcx_provision_cloud_agent(
        vcx_command_handle_t handle,
        const char *config,
        void (*cb)(vcx_command_handle_t xhandle, vcx_error_t err, const char *config)
);

const char *vcx_error_c_message(int);

const char *vcx_version();

vcx_error_t vcx_get_current_error(const char **error_json_p);

/**
 * Schema object
 *
 * For creating, validating and committing a schema to the sovrin ledger.
 *

/** Populates status with the current state of this credential. */
vcx_error_t vcx_schema_serialize(
        vcx_command_handle_t command_handle,
        vcx_schema_handle_t schema_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *serialized_schema)
);

/** Re-creates a credential object from the specified serialization. */
vcx_error_t vcx_schema_deserialize(
        vcx_command_handle_t command_handle,
        const char *serialized_schema,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_schema_handle_t schema_handle)
);

/** Populates data with the contents of the schema handle. */
vcx_error_t vcx_schema_get_attributes(
        vcx_command_handle_t command_handle,
        const char *source_id,
        vcx_schema_handle_t sequence_no,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *schema_attrs)
);

vcx_error_t vcx_schema_create(
        vcx_command_handle_t command_handle,
        const char *source_id,
        const char *schema_name,
        const char *version,
        const char *schema_data,
        vcx_payment_handle_t payment_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_u32_t cred_def_handle)
);

vcx_error_t vcx_schema_prepare_for_endorser(
        vcx_command_handle_t command_handle,
        const char *source_id,
        const char *schema_name,
        const char *version,
        const char *schema_data,
        const char *endorser,
        void (*cb)(
                vcx_command_handle_t xcommand_handle,
                vcx_error_t err,
                vcx_u32_t schema_handle,
                const char *schema_transaction
        )
);

vcx_error_t vcx_schema_get_schema_id(
        vcx_command_handle_t command_handle,
        vcx_u32_t schema_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *schema_id)
);

vcx_error_t vcx_schema_update_state(
        vcx_command_handle_t command_handle,
        vcx_u32_t schema_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_state_t state)
);

/** Release memory associated with schema object. */
vcx_error_t vcx_schema_release(vcx_schema_handle_t handle);

/*
*    vcx agent
*/
vcx_error_t vcx_public_agent_create(
        vcx_command_handle_t command_handle,
        const char *source_id,
        const char *institution_did,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_u32_t agent_handle)
);

vcx_error_t vcx_generate_public_invite(
        vcx_command_handle_t command_handle,
        const char *public_did,
        const char *label,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *public_invite)
);

vcx_error_t vcx_public_agent_download_connection_requests(
        vcx_command_handle_t command_handle,
        vcx_u32_t agent_handle,
        const char *uids,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *requests)
);

vcx_error_t vcx_public_agent_download_message(
        vcx_command_handle_t command_handle,
        vcx_u32_t agent_handle,
        const char *uid,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *message)
);

vcx_error_t vcx_public_agent_get_service(
        vcx_command_handle_t command_handle,
        vcx_u32_t agent_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *service)
);

vcx_error_t vcx_public_agent_serialize(
        vcx_command_handle_t command_handle,
        vcx_u32_t agent_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *agent_json)
);

vcx_error_t vcx_public_agent_release(vcx_u32_t agent_handle);

/*
*   Out of Band protocol
*/

vcx_error_t vcx_out_of_band_sender_create(
        vcx_command_handle_t command_handle,
        const char *config,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_u32_t oob_handle)
);

vcx_error_t vcx_out_of_band_receiver_create(
        vcx_command_handle_t command_handle,
        const char *message,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_u32_t oob_handle)
);

vcx_error_t vcx_out_of_band_sender_append_message(
        vcx_command_handle_t command_handle,
        vcx_u32_t oob_handle,
        const char *message,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err)
);

vcx_error_t vcx_out_of_band_sender_append_service(
        vcx_command_handle_t command_handle,
        vcx_u32_t oob_handle,
        const char *service,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err)
);

vcx_error_t vcx_out_of_band_sender_append_service_did(
        vcx_command_handle_t command_handle,
        vcx_u32_t oob_handle,
        const char *did,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err)
);

vcx_error_t vcx_out_of_band_sender_get_thread_id(
        vcx_command_handle_t command_handle,
        vcx_u32_t oob_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *thread_id)
);

vcx_error_t vcx_out_of_band_receiver_get_thread_id(
        vcx_command_handle_t command_handle,
        vcx_u32_t oob_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *thread_id)
);

vcx_error_t vcx_out_of_band_receiver_extract_message(
        vcx_command_handle_t command_handle,
        vcx_u32_t oob_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *message)
);

vcx_error_t vcx_out_of_band_to_message(
        vcx_command_handle_t command_handle,
        vcx_u32_t oob_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *message)
);

vcx_error_t vcx_out_of_band_sender_serialize(
        vcx_command_handle_t command_handle,
        vcx_u32_t oob_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *oob_message)
);

vcx_error_t vcx_out_of_band_receiver_serialize(
        vcx_command_handle_t command_handle,
        vcx_u32_t oob_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *oob_message)
);

vcx_error_t vcx_out_of_band_sender_deserialize(
        vcx_command_handle_t command_handle,
        const char *oob_message,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_u32_t oob_handle)
);

vcx_error_t vcx_out_of_band_receiver_deserialize(
        vcx_command_handle_t command_handle,
        const char *oob_message,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_u32_t oob_handle)
);

vcx_error_t vcx_out_of_band_sender_release(
        vcx_u32_t oob_handle
);

vcx_error_t vcx_out_of_band_receiver_release(
        vcx_u32_t oob_handle
);

vcx_error_t vcx_out_of_band_receiver_connection_exists(
        vcx_command_handle_t command_handle,
        vcx_u32_t oob_handle,
        const char *connection_handles,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_connection_handle_t connection_handle, vcx_bool_t found_one)
);

vcx_error_t vcx_out_of_band_receiver_build_connection(
        vcx_command_handle_t command_handle,
        vcx_u32_t oob_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *connection)
);

/*
*   Revocation registry
*/

vcx_error_t vcx_revocation_registry_create(
        vcx_command_handle_t command_handle,
        const char *rev_reg_config,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_u32_t rev_reg_handle)
);

vcx_error_t vcx_revocation_registry_publish(
        vcx_command_handle_t command_handle,
        vcx_u32_t rev_reg_handle,
        const char *tails_url,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_u32_t handle)
);

vcx_error_t vcx_revocation_registry_publish_revocations(
        vcx_command_handle_t command_handle,
        vcx_u32_t rev_reg_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err)
);

vcx_error_t vcx_revocation_registry_get_rev_reg_id(
        vcx_command_handle_t command_handle,
        vcx_u32_t rev_reg_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *rev_reg_id)
);

vcx_error_t vcx_revocation_registry_get_tails_hash(
        vcx_command_handle_t command_handle,
        vcx_u32_t rev_reg_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *tails_hash)
);

vcx_error_t vcx_revocation_registry_deserialize(
        vcx_command_handle_t command_handle,
        const char *rev_reg_json,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_u32_t rev_reg_handle)
);

vcx_error_t vcx_revocation_registry_serialize(
        vcx_command_handle_t command_handle,
        vcx_u32_t rev_reg_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *rev_reg_json)
);

vcx_error_t vcx_revocation_registry_release(vcx_u32_t rev_reg_handle);

/**
 * credentialdef object
 *
 * For creating, validating and committing a credential definition to the sovrin ledger.
 */

vcx_error_t vcx_credentialdef_create_v2(
        vcx_command_handle_t command_handle,
        const char *source_id,
        const char *schema_id,
        const char *issuer_did,
        const char *tag,
        vcx_bool_t support_revocation,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_credential_def_handle_t cred_def_handle)
);

vcx_error_t vcx_credentialdef_publish(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t cred_def_handle,
        const char *tails_url,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err)
);

/** Re-creates a credential object from the specified serialization. */
vcx_error_t vcx_credentialdef_deserialize(
        vcx_command_handle_t command_handle,
        const char *cred_def_data,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_credential_handle_t cred_def_handle)
);

/** Populates status with the current state of this credential. */
vcx_error_t vcx_credentialdef_serialize(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t cred_def_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *cred_def_data)
);

vcx_error_t vcx_credentialdef_release(
        vcx_credential_handle_t cred_def_handle
);

vcx_error_t vcx_credentialdef_get_cred_def_id(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t cred_def_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *cred_def_id)
);

vcx_error_t vcx_credentialdef_update_state(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t cred_def_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_state_t state)
);

vcx_error_t vcx_credentialdef_get_state(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t cred_def_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_state_t state)
);

/**
 * connection object
 *
 * For creating a connection with an identity owner for interactions such as exchanging
 * credentials and proofs.
 */

/** Creates a connection object to a specific identity owner. Populates a handle to the new connection. */
vcx_error_t vcx_connection_create(
        vcx_command_handle_t command_handle,
        const char *source_id,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, vcx_connection_handle_t connection_handle)
);

/** Asynchronously request a connection be made. */
vcx_error_t vcx_connection_connect(
        vcx_command_handle_t command_handle,
        vcx_connection_handle_t connection_handle,
        const char *connection_type,
        void (*cb)(vcx_command_handle_t, vcx_error_t err, const char *invite_details)
);

/** Returns the contents of the connection handle or null if the connection does not exist. */
vcx_error_t vcx_connection_serialize(
        vcx_command_handle_t command_handle,
        vcx_connection_handle_t connection_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *state)
);

/** Re-creates a connection object from the specified serialization. */
vcx_error_t vcx_connection_deserialize(
        vcx_command_handle_t command_handle,
        const char *serialized_credential,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_connection_handle_t connection_handle)
);

/** Request a state update from the agent for the given connection. */
vcx_error_t vcx_connection_update_state(
        vcx_command_handle_t command_handle,
        vcx_connection_handle_t connection_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_state_t state)
);

vcx_error_t vcx_connection_update_state_with_message(
        vcx_command_handle_t command_handle,
        vcx_connection_handle_t connection_handle,
        const char *message,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_state_t state)
);

vcx_error_t vcx_connection_handle_message(
        vcx_command_handle_t command_handle,
        vcx_connection_handle_t connection_handle,
        const char *message,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err)
);

/** Retrieves the state of the connection */
vcx_error_t vcx_connection_get_state(
        vcx_command_handle_t command_handle,
        vcx_connection_handle_t connection_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_state_t state)
);

/** Releases the connection from memory. */
vcx_error_t vcx_connection_release(vcx_connection_handle_t connection_handle);

/** Get the invite details for the connection. */
vcx_error_t vcx_connection_invite_details(
        vcx_command_handle_t command_handle,
        vcx_connection_handle_t connection_handle,
        vcx_bool_t abbreviated,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *details)
);

/** Creates a connection from the invite details. */
vcx_error_t vcx_connection_create_with_invite(
        vcx_command_handle_t command_handle,
        const char *source_id,
        const char *invite_details,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_connection_handle_t connection_handle)
);

vcx_error_t vcx_connection_create_with_connection_request(
        vcx_command_handle_t command_handle,
        const char *source_id,
        vcx_u32_t agent_handle,
        const char *request,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_connection_handle_t connection_handle)
);


/** Deletes a connection, send an API call to agency to stop sending messages from this connection */
vcx_error_t vcx_connection_delete_connection(
        vcx_command_handle_t command_handle,
        vcx_connection_handle_t connection_handle,
        void (*cb)(vcx_command_handle_t, vcx_error_t err)
);

/** Retrieves pw_did from Connection object. */
vcx_error_t vcx_connection_get_pw_did(
        vcx_command_handle_t command_handle,
        vcx_connection_handle_t connection_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *pw_did)
);

/** Retrieves their_pw_did from Connection object. */
vcx_error_t vcx_connection_get_their_pw_did(
        vcx_command_handle_t command_handle,
        vcx_connection_handle_t connection_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *their_pw_did)
);

vcx_error_t vcx_connection_info(
        vcx_command_handle_t command_handle,
        vcx_connection_handle_t connection_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *info)
);

vcx_error_t vcx_connection_get_thread_id(
        vcx_command_handle_t command_handle,
        vcx_connection_handle_t connection_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *thread_id)
);

vcx_error_t vcx_connection_messages_download(
        vcx_command_handle_t command_handle,
        vcx_connection_handle_t connection_handle,
        const char *message_status,
        const char *uids,
        void(*cb)(vcx_command_handle_t xhandle, vcx_error_t err, const char *messages)
);

vcx_error_t vcx_connection_send_handshake_reuse(
        vcx_command_handle_t command_handle,
        vcx_connection_handle_t connection_handle,
        const char *oob_msg,
        void(*cb)(vcx_command_handle_t xhandle, vcx_error_t err)
);

/** Send a message to the specified connection
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: connection to receive the message
///
/// msg: actual message to send
///
/// send_message_options: config options json string that contains following options
///     {
///         msg_type: String, // type of message to send
///         msg_title: String, // message title (user notification)
///         ref_msg_id: Option<String>, // If responding to a message, id of the message
///     }
///
///
/// cb: Callback that provides array of matching messages retrieved
///
/// #Returns
/// Error code as a u32
 */
vcx_error_t vcx_connection_send_message(
        vcx_command_handle_t command_handle,
        vcx_connection_handle_t connection_handle,
        const char *msg,
        const char *send_message_options,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *msg_id)
);

/** Generate a signature for the specified data */
vcx_error_t vcx_connection_sign_data(
        vcx_command_handle_t command_handle,
        vcx_connection_handle_t connection_handle,
        uint8_t const *data_raw, vcx_u32_t data_len,
        void (*cb)(vcx_command_handle_t, vcx_error_t err, uint8_t const *signature_raw, vcx_u32_t signature_len)
);

/** Verify the signature is valid for the specified data */
vcx_error_t vcx_connection_verify_signature(
        vcx_command_handle_t command_handle,
        vcx_connection_handle_t connection_handle,
        uint8_t const *data_raw,
        vcx_u32_t data_len,
        uint8_t const *signature_raw,
        vcx_u32_t signature_len,
        void (*cb)(vcx_command_handle_t, vcx_error_t err, vcx_bool_t valid)
);

vcx_error_t vcx_connection_send_ping(
        vcx_command_handle_t command_handle,
        vcx_connection_handle_t connection_handle,
        const char *comment,
        void (*cb)(vcx_command_handle_t, vcx_error_t err)
);

vcx_error_t vcx_connection_send_discovery_features(
        vcx_command_handle_t command_handle,
        vcx_connection_handle_t connection_handle,
        const char *query,
        const char *comment,
        void (*cb)(vcx_command_handle_t, vcx_error_t err)
);


/**
 * credential issuer object
 *
 * Used for offering and managing a credential with an identity owner.
 */

/** Creates a credential object from the specified credentialdef handle. Populates a handle the new credential. */
vcx_error_t vcx_issuer_create_credential(
        vcx_command_handle_t command_handle,
        const char *source_id,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, vcx_credential_handle_t credential_handle)
);

vcx_error_t vcx_issuer_revoke_credential_local(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t credential_handle,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err)
);

vcx_error_t vcx_issuer_credential_is_revokable(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t credential_handle,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, vcx_bool_t revokable)
);

vcx_error_t vcx_issuer_send_credential_offer_v2(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t credential_handle,
        vcx_connection_handle_t connection_handle,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err)
);

vcx_error_t vcx_mark_credential_offer_msg_sent(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t credential_handle,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, const char *message)
);

vcx_error_t vcx_issuer_build_credential_offer_msg_v2(
        vcx_command_handle_t command_handle,
        vcx_credential_def_handle_t cred_def_handle,
        vcx_credential_handle_t rev_reg_handle,
        const char *credential_data,
        const char *comment,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, const char *message)
);

vcx_error_t vcx_issuer_get_credential_offer_msg(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t credential_handle,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, const char *message)
);

vcx_error_t vcx_issuer_get_credential_msg(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t credential_handle,
        const char *my_pw_did,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, const char *message)
);

/** Retrieves the state of the issuer_credential. */
vcx_error_t vcx_issuer_credential_get_state(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t credential_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_state_t state)
);

vcx_error_t vcx_issuer_credential_get_rev_reg_id(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t credential_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *rev_reg_id)
);

/** Asynchronously send the credential to the connection. Populates a handle to the new transaction. */
vcx_error_t vcx_issuer_send_credential(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t credential_handle,
        vcx_connection_handle_t connection_handle,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, vcx_state_t state)
);

/** Populates status with the current state of this credential. */
vcx_error_t vcx_issuer_credential_serialize(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t credential_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *serialized_credential)
);

/** Re-creates a credential object from the specified serialization. */
vcx_error_t vcx_issuer_credential_deserialize(
        vcx_command_handle_t,
        const char *serialized_credential,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_credential_handle_t credential_handle)
);

vcx_error_t vcx_issuer_credential_get_thread_id(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t credential_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *thread_id)
);

vcx_error_t vcx_v2_issuer_credential_update_state(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t credential_handle,
        vcx_connection_handle_t connection_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_state_t state)
);

vcx_error_t vcx_v2_issuer_credential_update_state_with_message(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t credential_handle,
        vcx_connection_handle_t connection_handle,
        const char *message,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_state_t state)
);

/** Releases the credential from memory. */
vcx_error_t vcx_issuer_credential_release(vcx_credential_handle_t credential_handle);


/**
 * proof object
 *
 * Used for requesting and managing a proof request with an identity owner.
 */

/** Creates a proof object.  Populates a handle to the new proof. */
vcx_error_t vcx_proof_create(
        vcx_command_handle_t command_handle,
        const char *source_id,
        const char *requested_attrs,
        const char *requested_predicates,
        const char *revocation_interval,
        const char *name,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, vcx_proof_handle_t proof_handle)
);

/** Asynchronously send a proof request to the connection. */
vcx_error_t vcx_proof_send_request(
        vcx_command_handle_t command_handle,
        vcx_proof_handle_t proof_handle,
        vcx_connection_handle_t connection_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err)
);

vcx_error_t vcx_get_proof_msg(
        vcx_command_handle_t command_handle,
        vcx_proof_handle_t proof_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_state_t proof_state, const char *response_data)
);

vcx_error_t vcx_proof_get_request_msg(
        vcx_command_handle_t command_handle,
        vcx_proof_handle_t proof_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *message)
);

vcx_error_t vcx_v2_proof_update_state(
        vcx_command_handle_t command_handle,
        vcx_proof_handle_t proof_handle,
        vcx_connection_handle_t connection_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_state_t state)
);

vcx_error_t vcx_v2_proof_update_state_with_message(
        vcx_command_handle_t command_handle,
        vcx_proof_handle_t proof_handle,
        vcx_connection_handle_t connection_handle,
        const char *message,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_state_t state)
);

/** Populate response_data with the latest proof offer received. */
//vcx_error_t vcx_get_proof(
//        vcx_command_handle_t command_handle,
//        vcx_proof_handle_t proof_handle,
//        vcx_connection_handle_t connection_handle,
//        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_proof_state_t state, const char *proof_string)
//);

/** Retrieves the state of the proof. */
vcx_error_t vcx_proof_get_state(
        vcx_command_handle_t command_handle,
        vcx_proof_handle_t proof_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_state_t state)
);

vcx_error_t vcx_proof_get_thread_id(
        vcx_command_handle_t command_handle,
        vcx_proof_handle_t proof_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_state_t state, const char *thread_id)
);

vcx_error_t vcx_mark_presentation_request_msg_sent(
        vcx_command_handle_t command_handle,
        vcx_proof_handle_t proof_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_state_t state, const char *message)
);

/** Populates status with the current state of this proof. */
vcx_error_t vcx_proof_serialize(
        vcx_command_handle_t command_handle,
        vcx_proof_handle_t proof_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *serialized_proof)
);

/** Re-creates a proof object from the specified serialization. */
vcx_error_t vcx_proof_deserialize(
        vcx_command_handle_t command_handle,
        const char *serialized_proof,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_proof_handle_t proof_handle)
);

/** Releases the proof from memory. */
vcx_error_t vcx_proof_release(vcx_proof_handle_t proof_handle);

/**
 * disclosed_proof object
 *
 * Used for sending a disclosed_proof to an identity owner.
 */

/** Creates a disclosed_proof object from a proof request.  Populates a handle to the new disclosed_proof. */
vcx_error_t vcx_disclosed_proof_create_with_request(
        vcx_command_handle_t command_handle,
        const char *source_id,
        const char *proof_req,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, vcx_proof_handle_t proof_handle)
);

/** Creates a disclosed_proof object from a msgid.  Populates a handle to the new disclosed_proof. */
vcx_error_t vcx_disclosed_proof_create_with_msgid(
        vcx_command_handle_t command_handle,
        const char *source_id, vcx_connection_handle_t connectionHandle,
        const char *msg_id,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, vcx_proof_handle_t proof_handle, const char *proof_request)
);

/** Asynchronously send a proof to the connection. */
vcx_error_t vcx_disclosed_proof_send_proof(
        vcx_command_handle_t command_handle,
        vcx_proof_handle_t proof_handle,
        vcx_connection_handle_t connection_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err)
);

/** Asynchronously send reject of a proof to the connection. */
vcx_error_t vcx_disclosed_proof_reject_proof(
        vcx_command_handle_t command_handle,
        vcx_proof_handle_t proof_handle,
        vcx_connection_handle_t connection_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err)
);

/** Declines presentation request */
vcx_error_t vcx_disclosed_proof_decline_presentation_request(
        vcx_command_handle_t command_handle,
        vcx_proof_handle_t proof_handle,
        vcx_connection_handle_t connection_handle,
        const char *reason,
        const char *proposal,
        void(*cb)(vcx_command_handle_t xhandle, vcx_error_t err)
);

vcx_error_t vcx_disclosed_proof_get_thread_id(
        vcx_command_handle_t command_handle,
        vcx_proof_handle_t proof_handle,
        void(*cb)(vcx_command_handle_t xhandle, vcx_error_t err, const char *thread_id)
);

/** Get proof msg */
vcx_error_t vcx_disclosed_proof_get_proof_msg(
        vcx_command_handle_t command_handle,
        vcx_proof_handle_t proof_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *msg)
);

/** Get proof reject msg */
vcx_error_t vcx_disclosed_proof_get_reject_msg(
        vcx_command_handle_t command_handle,
        vcx_proof_handle_t proof_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *msg)
);

/** Get attributes specified in proof request*/
vcx_error_t vcx_disclosed_proof_get_proof_request_attachment(
        vcx_command_handle_t command_handle,
        vcx_proof_handle_t proof_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *attrs)
);

vcx_error_t vcx_v2_disclosed_proof_update_state(
        vcx_command_handle_t command_handle,
        vcx_proof_handle_t proof_handle,
        vcx_connection_handle_t connection_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_state_t state)
);

vcx_error_t vcx_v2_disclosed_proof_update_state_with_message(
        vcx_command_handle_t command_handle,
        vcx_proof_handle_t proof_handle,
        vcx_connection_handle_t connection_handle,
        const char *message,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_state_t state)
);

/** Check for any proof requests from the connection. */
vcx_error_t vcx_disclosed_proof_get_requests(
        vcx_command_handle_t command_handle,
        vcx_connection_handle_t connection_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *requests)
);

/** Retrieves the state of the disclosed_proof. */
vcx_error_t vcx_disclosed_proof_get_state(
        vcx_command_handle_t command_handle,
        vcx_proof_handle_t proof_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_state_t state)
);

/** Populates status with the current state of this disclosed_proof. */
vcx_error_t vcx_disclosed_proof_serialize(
        vcx_command_handle_t command_handle,
        vcx_proof_handle_t proof_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *proof_request)
);

/** Re-creates a disclosed_proof object from the specified serialization. */
vcx_error_t vcx_disclosed_proof_deserialize(
        vcx_command_handle_t command_handle,
        const char *serialized_proof,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_proof_handle_t proof_handle)
);

/** Takes the disclosed proof object and returns a json string of all credentials matching associated proof request from wallet */
vcx_error_t vcx_disclosed_proof_retrieve_credentials(
        vcx_command_handle_t command_handle,
        vcx_proof_handle_t proof_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *matching_credentials)
);

/** Takes the disclosed proof object and generates a proof from the selected credentials and self attested attributes */
vcx_error_t vcx_disclosed_proof_generate_proof(
        vcx_command_handle_t command_handle,
        vcx_proof_handle_t proof_handle,
        const char *selected_credentials,
        const char *self_attested_attrs,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err)
);

/** Releases the disclosed_proof from memory. */
vcx_error_t vcx_disclosed_proof_release(vcx_proof_handle_t proof_handle);

/**
 * credential object
 *
 * Used for accepting and requesting a credential with an identity owner.
 */

vcx_error_t vcx_get_credential(
        vcx_command_handle_t handle,
        vcx_credential_handle_t credential_handle,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, const char *credential)
);

/** Creates a credential object from the specified credentialdef handle. Populates a handle the new credential. */
vcx_error_t vcx_credential_create_with_offer(
        vcx_command_handle_t command_handle,
        const char *source_id,
        const char *credential_offer,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, vcx_credential_handle_t credential_handle)
);

/** Creates a credential object from the connection and msg id. Populates a handle the new credential. */
vcx_error_t vcx_credential_create_with_msgid(
        vcx_command_handle_t command_handle,
        const char *source_id,
        vcx_connection_handle_t connection,
        const char *msg_id,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, vcx_credential_handle_t credential_handle, const char *credential_offer)
);

/** Asynchronously sends the credential request to the connection. */
vcx_error_t vcx_credential_send_request(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t credential_handle,
        vcx_connection_handle_t connection_handle,
        vcx_payment_handle_t payment_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err)
);

vcx_error_t vcx_credential_get_request_msg(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t credential_handle,
        const char *my_pw_did,
        const char *their_pw_did,
        vcx_payment_handle_t payment_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *message)
);

vcx_error_t vcx_credential_decline_offer(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t credential_handle,
        vcx_connection_handle_t connection_handle,
        const char *comment,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err)
);

/** Check for any credential offers from the connection. */
vcx_error_t vcx_credential_get_offers(
        vcx_command_handle_t command_handle,
        vcx_connection_handle_t connection_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *offers)
);

/** Get attributes for specified credential */
vcx_error_t vcx_credential_get_attributes(
        vcx_command_handle_t handle,
        vcx_credential_handle_t credential_handle,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, const char *attributes)
);

vcx_error_t vcx_credential_get_attachment(
        vcx_command_handle_t handle,
        vcx_credential_handle_t credential_handle,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, const char *attachment)
);

vcx_error_t vcx_credential_get_tails_location(
        vcx_command_handle_t handle,
        vcx_credential_handle_t credential_handle,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, const char *tailsLocation)
);

vcx_error_t vcx_credential_get_tails_hash(
        vcx_command_handle_t handle,
        vcx_credential_handle_t credential_handle,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, const char *tailsHash)
);

vcx_error_t vcx_credential_get_rev_reg_id(
        vcx_command_handle_t handle,
        vcx_credential_handle_t credential_handle,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, const char *revRegId)
);

vcx_error_t vcx_credential_is_revokable(
        vcx_command_handle_t handle,
        vcx_credential_handle_t credential_handle,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err, vcx_bool_t revokable)
);

vcx_error_t vcx_v2_credential_update_state(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t credential_handle,
        vcx_connection_handle_t connection_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_state_t state)
);

vcx_error_t vcx_v2_credential_update_state_with_message(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t credential_handle,
        vcx_connection_handle_t connection_handle,
        const char *message,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_state_t state)
);

/** Retrieves the state of the credential - including storing the credential if it has been sent. */
vcx_error_t vcx_credential_get_state(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t credential_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_state_t state)
);

/** Populates status with the current state of this credential. */
vcx_error_t vcx_credential_serialize(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t credential_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, const char *state)
);

/** Re-creates a credential from the specified serialization. */
vcx_error_t vcx_credential_deserialize(
        vcx_command_handle_t,
        const char *serialized_credential,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err, vcx_credential_handle_t credential_handle)
);

/** Releases the credential from memory. */
vcx_error_t vcx_credential_release(vcx_credential_handle_t credential_handle);

/** Delete the credential from the wallet. */
vcx_error_t vcx_delete_credential(
        vcx_command_handle_t command_handle,
        vcx_credential_handle_t credential_handle,
        void (*cb)(vcx_command_handle_t xcommand_handle, vcx_error_t err)
);


/**
 * wallet object
 *
 * Used for exporting and importing and managing the wallet.
 */

vcx_error_t vcx_wallet_set_handle(vcx_i32_t handle);

/** Export the wallet as an encrypted file */
vcx_error_t vcx_wallet_export(
        vcx_command_handle_t handle,
        const char *path,
        const char *backup_key,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err)
);

/** Import an encrypted file back into the wallet */
vcx_error_t vcx_wallet_import(
        vcx_command_handle_t handle,
        const char *config,
        void (*cb)(vcx_command_handle_t command_handle, vcx_error_t err)
);

/** Add a record inside a wallet */
vcx_error_t vcx_wallet_add_record(
        vcx_command_handle_t handle,
        const char *type_,
        const char *record_id,
        const char *record_value,
        const char *tags_json,
        void (*cb)(vcx_command_handle_t xhandle, vcx_error_t err)
);

/** Get a record from wallet */
vcx_error_t vcx_wallet_get_record(
        vcx_command_handle_t handle,
        const char *type_,
        const char *record_id,
        const char *options,
        void (*cb)(vcx_command_handle_t xhandle, vcx_error_t err, const char *record_json)
);

/** Delete a record from wallet */
vcx_error_t vcx_wallet_delete_record(
        vcx_command_handle_t handle,
        const char *type_,
        const char *record_id,
        void (*cb)(vcx_command_handle_t xhandle, vcx_error_t err)
);

/** Update a record in wallet if it is already added */
vcx_error_t vcx_wallet_update_record_value(
        vcx_command_handle_t handle,
        const char *type_,
        const char *record_id,
        const char *record_value,
        void (*cb)(vcx_command_handle_t xhandle, vcx_error_t err)
);

/** Add record tags to a record */
vcx_error_t vcx_wallet_add_record_tags(
        vcx_command_handle_t command_handle,
        const char *type_,
        const char *record_id,
        const char *tags_json,
        void (*cb)(vcx_command_handle_t xhandle, vcx_error_t err)
);

/** Update record tags in a record */
vcx_error_t vcx_wallet_update_record_tags(
        vcx_command_handle_t command_handle,
        const char *type_,
        const char *record_id,
        const char *tags_json,
        void (*cb)(vcx_command_handle_t xhandle, vcx_error_t err)
);

/** Delete record tags from a record */
vcx_error_t vcx_wallet_delete_record_tags(
        vcx_command_handle_t command_handle,
        const char *type_, const char *record_id,
        const char *tag_names_json,
        void (*cb)(vcx_command_handle_t xhandle, vcx_error_t err)
);

/** Opens a wallet search handle */
vcx_error_t vcx_wallet_open_search(
        vcx_command_handle_t commond_handle,
        const char *type_, const char *query_json,
        const char *options_json,
        void (*cb)(vcx_command_handle_t xhandle, vcx_error_t err, vcx_search_handle_t search_handle)
);

/** Fetch next records for wallet search */
vcx_error_t vcx_wallet_search_next_records(
        vcx_command_handle_t command_handle,
        vcx_search_handle_t search_handle,
        int count,
        void (*cb)(vcx_command_handle_t xhandle, vcx_error_t err, const char *records_json)
);

/** Close a search */
vcx_error_t vcx_wallet_close_search(
        vcx_command_handle_t commond_handle,
        vcx_search_handle_t search_handle,
        void (*cb)(vcx_command_handle_t xhandle, vcx_error_t err)
);

/** Shutdown vcx wallet */
vcx_error_t vcx_shutdown(vcx_bool_t deleteWallet);

/** Get Messages (Connections) of given status */
vcx_error_t vcx_v2_messages_download(
        vcx_command_handle_t command_handle,
        const char *connection_handles,
        const char *message_status,
        const char *uids,
        void(*cb)(vcx_command_handle_t xhandle, vcx_error_t err, const char *messages)
);

/** Update Message status */
vcx_error_t vcx_messages_update_status(
        vcx_command_handle_t command_handle,
        const char *message_status,
        const char *msg_json,
        void(*cb)(vcx_command_handle_t xhandle, vcx_error_t err)
);

/**
 * logging
 **/
vcx_error_t vcx_set_default_logger(const char *pattern);

vcx_error_t vcx_set_logger(
        const void *context,
        vcx_bool_t (*enabledFn)(
                const void *context,
                vcx_u32_t level,
                const char *target),
        void (*logFn)(
                const void *context,
                vcx_u32_t level,
                const char *target,
                const char *message,
                const char *module_path,
                const char *file,
                vcx_u32_t line),
        void (*flushFn)(const void *context)
);

/// Retrieve author agreement set on the Ledger
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// cb: Callback that provides array of matching messages retrieved
///
/// #Returns
/// Error code as a u32
vcx_error_t vcx_get_ledger_author_agreement(
        vcx_u32_t command_handle,
        void (*cb)(vcx_command_handle_t, vcx_error_t, const char *)
);

/// Set some accepted agreement as active.
///
/// As result of succesfull call of this funciton appropriate metadata will be appended to each write request by `indy_append_txn_author_agreement_meta_to_request` libindy call.
///
/// #Params
/// text and version - (optional) raw data about TAA from ledger.
///     These parameters should be passed together.
///     These parameters are required if hash parameter is ommited.
/// hash - (optional) hash on text and version. This parameter is required if text and version parameters are ommited.
/// acc_mech_type - mechanism how user has accepted the TAA
/// time_of_acceptance - UTC timestamp when user has accepted the TAA
///
/// #Returns
/// Error code as a u32
vcx_error_t vcx_set_active_txn_author_agreement_meta(
        const char *text,
        const char *version,
        const char *hash,
        const char *acc_mech_type,
        vcx_u64_t type_
);

/** For testing purposes only */
void vcx_set_next_agency_response(int);

#ifdef __cplusplus
}
#endif

#endif
