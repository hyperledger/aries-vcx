//
// Created by Paulo Silva on 25/10/22.
// Copyright (c) 2022 GuestUser. All rights reserved.
//

#ifndef VcxWrapperCallbacks_h
#define VcxWrapperCallbacks_h

#import "libvcx.h"
#import "VcxTypes.h"

extern void VcxWrapperCbNoResponse(
        vcx_command_handle_t xcommand_handle,
        vcx_error_t err
);

extern void VcxWrapperCbResponseHandle(
        vcx_command_handle_t xcommand_handle,
        vcx_error_t err,
        vcx_command_handle_t handle
);

extern void VcxWrapperCbResponseSignedHandle(
        vcx_command_handle_t xcommand_handle,
        vcx_error_t err,
        vcx_i32_t signed_handle
);

extern void VcxWrapperCbResponseUnsignedInt(
        vcx_command_handle_t xcommand_handle,
        vcx_error_t err,
        vcx_command_handle_t unsigned_int
);

extern void VcxWrapperCbResponseString(
        vcx_command_handle_t xcommand_handle,
        vcx_error_t err,
        const char *const string
);

extern void VcxWrapperCbResponseBool(
        vcx_command_handle_t xcommand_handle,
        vcx_error_t err,
        unsigned int bhool
);

extern void VcxWrapperCbResponseData(
        vcx_command_handle_t xcommand_handle,
        vcx_error_t err,
        const uint8_t *const data,
        uint32_t dataLen
);

extern void VcxWrapperCbResponseHandleAndString(
        vcx_command_handle_t xcommand_handle,
        vcx_error_t err,
        vcx_command_handle_t handle,
        const char *const string
);

extern void VcxWrapperCbResponseHandleAndBool(
        vcx_command_handle_t xcommand_handle,
        vcx_error_t err,
        vcx_command_handle_t handle,
        vcx_bool_t bhool
);

#endif //VcxWrapperCallbacks_h
