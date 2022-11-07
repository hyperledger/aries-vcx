//
//  VcxTypes.h
//  vcx-demo
//
//  Created by Norman Jarvis on 5/7/18.
//  Copyright © 2018 GuestUser. All rights reserved.
//

#ifndef VcxTypes_h
#define VcxTypes_h

typedef enum {
    none = 0,
    initialized,
    offer_sent,
    request_received,
    accepted,
    unfulfilled,
    expired,
    revoked,
} vcx_state_t;

typedef enum {
    undefined = 0,
    validated = 1,
    invalid = 2,
} vcx_proof_state_t;

typedef unsigned int VcxHandle;
typedef unsigned int vcx_error_t;
typedef VcxHandle vcx_schema_handle_t;
typedef VcxHandle vcx_credential_def_handle_t;
typedef VcxHandle vcx_connection_handle_t;
typedef VcxHandle vcx_credential_handle_t;
typedef VcxHandle vcx_proof_handle_t;
typedef VcxHandle vcx_search_handle_t;
typedef VcxHandle vcx_command_handle_t;
typedef VcxHandle vcx_payment_handle_t;
typedef unsigned int vcx_bool_t;
typedef unsigned int vcx_u32_t;
typedef int vcx_i32_t;
typedef unsigned long long vcx_u64_t;

typedef const uint8_t vcx_data_t;

typedef struct {

    union {
        vcx_schema_handle_t schema_handle;
        vcx_credential_def_handle_t credentialdef_handle;
        vcx_connection_handle_t connection_handle;
        vcx_credential_handle_t credential_handle;
        vcx_proof_handle_t proof_handle;
    } handle;

    vcx_error_t status;
    char *msg;

} vcx_status_t;

#define ERROR_RESPONSE_NUMBER -1
#define ERROR_RESPONSE_STRING nil
#define ERROR_RESPONSE_DATA nil
#define ERROR_RESPONSE_BOOL false

#endif /* VcxTypes_h */
