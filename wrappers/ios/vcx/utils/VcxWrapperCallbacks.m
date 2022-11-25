//
// Created by Paulo Silva on 25/10/22.
// Copyright (c) 2022 GuestUser. All rights reserved.
//

#include "VcxWrapperCallbacks.h"
#import <Foundation/Foundation.h>
#include "VcxCallbacks.h"
#include "NSError+VcxError.h"
#include "VcxWrapperCallbacks.h"

void completeCallback(void (^completion)()) {
    if (completion) {
        dispatch_async(dispatch_get_main_queue(), ^{
            completion();
        });
    }
}

void VcxWrapperCbNoResponse(
        vcx_command_handle_t xcommand_handle,
        vcx_error_t err
) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *) = (void (^)(NSError *)) block;

    completeCallback(^{
        completion([NSError errorFromVcxError:err]);
    });
}

void VcxWrapperCbResponseHandle(
        vcx_command_handle_t xcommand_handle,
        vcx_error_t err,
        VcxHandle handle
) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, VcxHandle) = (void (^)(NSError *, VcxHandle)) block;

    completeCallback(^{
        completion([NSError errorFromVcxError:err], handle);
    });
}

void VcxWrapperCbResponseSignedHandle(
        vcx_command_handle_t xcommand_handle,
        vcx_error_t err,
        vcx_i32_t signed_handle
) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, vcx_i32_t) = (void (^)(NSError *, vcx_i32_t)) block;

    completeCallback(^{
        completion([NSError errorFromVcxError:err], signed_handle);
    });
}

void VcxWrapperCbResponseUnsignedInt(
        vcx_command_handle_t xcommand_handle,
        vcx_error_t err,
        vcx_command_handle_t unsigned_int
) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, vcx_command_handle_t) = (void (^)(NSError *, vcx_command_handle_t)) block;

    completeCallback(^{
        completion([NSError errorFromVcxError:err], unsigned_int);
    });
}

void VcxWrapperCbResponseString(
        vcx_command_handle_t xcommand_handle,
        vcx_error_t err,
        const char *string
) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSString *) = (void (^)(NSError *, NSString *)) block;
    NSString *response = nil;
    if (string) {
        response = [NSString stringWithUTF8String:string];
    }

    completeCallback(^{
        completion([NSError errorFromVcxError:err], response);
    });
}

void VcxWrapperCbResponseBool(
        vcx_command_handle_t xcommand_handle,
        vcx_error_t err,
        unsigned int bhool
) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, BOOL) = (void (^)(NSError *, BOOL)) block;

    completeCallback(^{
        completion([NSError errorFromVcxError:err], (BOOL) bhool);
    });
}

/// Arguments data and dataLen will be converted to nsdata
void VcxWrapperCbResponseData(
        vcx_command_handle_t xcommand_handle,
        vcx_error_t err,
        const uint8_t *const data,
        uint32_t dataLen
) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSData *) = (void (^)(NSError *, NSData *)) block;

    NSData *response = [NSData dataWithBytes:data length:dataLen];

    completeCallback(^{
        completion([NSError errorFromVcxError:err], response);
    });
}

void VcxWrapperCbResponseHandleAndString(
        vcx_command_handle_t xcommand_handle,
        vcx_error_t err,
        VcxHandle handle,
        const char *const string
) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, VcxHandle , NSString *) = (void (^)(NSError *, VcxHandle, NSString *)) block;

    NSString *response = nil;
    if (string) {
        response = [NSString stringWithUTF8String:string];
    }

    completeCallback(^{
        completion([NSError errorFromVcxError:err], handle, response);
    });
}

void VcxWrapperCbResponseHandleAndBool(
        vcx_command_handle_t xcommand_handle,
        vcx_error_t err,
        VcxHandle handle,
        vcx_bool_t bhool
) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, VcxHandle, Boolean) = (void (^)(NSError *, VcxHandle, Boolean)) block;

    completeCallback(^{
        completion([NSError errorFromVcxError:err], handle, bhool);
    });
}
