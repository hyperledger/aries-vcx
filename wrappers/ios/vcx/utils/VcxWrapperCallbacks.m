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

void VcxWrapperCbResponseSignedInt(
        vcx_command_handle_t xcommand_handle,
        vcx_error_t err,
        vcx_i32_t signed_handle
) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSNumber *) = (void (^)(NSError *, NSNumber *)) block;

    completeCallback(^{
        completion([NSError errorFromVcxError:err], [NSNumber numberWithInt:signed_handle]);
    });
}

void VcxWrapperCbResponseUnsignedInt(
        vcx_command_handle_t xcommand_handle,
        vcx_error_t err,
        vcx_command_handle_t unsigned_int
) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSNumber *) = (void (^)(NSError *, NSNumber *)) block;

    completeCallback(^{
        completion([NSError errorFromVcxError:err], [NSNumber numberWithUnsignedInt:unsigned_int]);
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

void VcxWrapperCbResponseUnsignedIntAndString(
        vcx_command_handle_t xcommand_handle,
        vcx_error_t err,
        VcxHandle handle,
        const char *const string
) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSNumber * , NSString *) = (void (^)(NSError *, NSNumber *, NSString *)) block;

    NSString *response = nil;
    if (string) {
        response = [NSString stringWithUTF8String:string];
    }

    completeCallback(^{
        completion([NSError errorFromVcxError:err], [NSNumber numberWithUnsignedInt:handle], response);
    });
}

void VcxWrapperCbResponseUnsignedIntAndBool(
        vcx_command_handle_t xcommand_handle,
        vcx_error_t err,
        VcxHandle handle,
        vcx_bool_t bhool
) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSNumber *, Boolean) = (void (^)(NSError *, NSNumber *, Boolean)) block;

    completeCallback(^{
        completion([NSError errorFromVcxError:err], [NSNumber numberWithUnsignedInt:handle], bhool);
    });
}
