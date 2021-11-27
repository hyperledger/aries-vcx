//
//  init.m
//  vcx
//
//  Created by GuestUser on 4/30/18.
//  Copyright © 2018 GuestUser. All rights reserved.
//

#import <Foundation/Foundation.h>
#import "ConnectMeVcx.h"
#import "utils/NSError+VcxError.h"
#import "utils/VcxCallbacks.h"
#import "vcx.h"
#include "vcx.h"

void VcxWrapperCommonCallback(vcx_command_handle_t xcommand_handle,
                              vcx_error_t err) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *) = (void (^)(NSError *)) block;

    if (completion) {
        dispatch_async(dispatch_get_main_queue(), ^{
            NSError *error = [NSError errorFromVcxError:err];
            completion(error);
        });
    }
}

void VcxWrapperCommonHandleCallback(vcx_command_handle_t xcommand_handle,
                                    vcx_error_t err,
                                    vcx_command_handle_t h) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSNumber *) = (void (^)(NSError *, NSNumber *)) block;

    if (completion) {
        dispatch_async(dispatch_get_main_queue(), ^{
            NSError *error = [NSError errorFromVcxError:err];
            NSNumber *handle = [NSNumber numberWithUnsignedInt:h];
            completion(error, handle);
        });
    }
}

void VcxWrapperCommonUnsignedHandleCallback(vcx_command_handle_t xcommand_handle,
                                    vcx_error_t err,
                                    vcx_u32_t h) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSNumber *) = (void (^)(NSError *, NSNumber *)) block;

    if (completion) {
        dispatch_async(dispatch_get_main_queue(), ^{
            NSError *error = [NSError errorFromVcxError:err];
            NSNumber *handle = [NSNumber numberWithUnsignedInt:h];
            completion(error, handle);
        });
    }
}

void VcxWrapperCommonSignedHandleCallback(vcx_command_handle_t xcommand_handle,
                                          vcx_error_t err,
                                          VcxHandle h) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSNumber *) = (void (^)(NSError *, NSNumber *)) block;

    if (completion) {
        dispatch_async(dispatch_get_main_queue(), ^{
            NSError *error = [NSError errorFromVcxError:err];
            NSNumber *handle = [NSNumber numberWithInt:h];
            completion(error, handle);
        });
    }
}

void VcxWrapperCommonNumberCallback(vcx_command_handle_t xcommand_handle,
                                    vcx_error_t err,
                                    vcx_u32_t h) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSNumber *) = (void (^)(NSError *, NSNumber *)) block;

    if (completion) {
        dispatch_async(dispatch_get_main_queue(), ^{
            NSError *error = [NSError errorFromVcxError:err];
            NSNumber *handle = [NSNumber numberWithUnsignedInt:h];
            completion(error, handle);
        });
    }
}

void VcxWrapperCommonStringCallback(vcx_command_handle_t xcommand_handle,
                                    vcx_error_t err,
                                    const char *const arg1) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSString *) = (void (^)(NSError *, NSString *arg1)) block;
    NSString *sarg1 = nil;
    if (arg1) {
        sarg1 = [NSString stringWithUTF8String:arg1];
    }

    if (completion) {
        dispatch_async(dispatch_get_main_queue(), ^{
            NSError *error = [NSError errorFromVcxError:err];
            completion(error, sarg1);
        });
    }
}

void VcxWrapperCommonBoolCallback(vcx_command_handle_t xcommand_handle,
                                  vcx_error_t err,
                                  vcx_bool_t arg1) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, BOOL) = (void (^)(NSError *, BOOL)) block;

    if (completion) {
        dispatch_async(dispatch_get_main_queue(), ^{
            NSError *error = [NSError errorFromVcxError:err];
            completion(error, (BOOL) arg1);
        });
    }
}

void VcxWrapperCommonStringStringCallback(vcx_command_handle_t xcommand_handle,
                                          vcx_error_t err,
                                          const char *const arg1,
                                          const char *const arg2) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSString *, NSString *) = (void (^)(NSError *, NSString *, NSString *)) block;

    NSString *sarg1 = nil;
    if (arg1) {
        sarg1 = [NSString stringWithUTF8String:arg1];
    }
    NSString *sarg2 = nil;
    if (arg2) {
        sarg2 = [NSString stringWithUTF8String:arg2];
    }

    if (completion) {
        dispatch_async(dispatch_get_main_queue(), ^{
            NSError *error = [NSError errorFromVcxError:err];
            completion(error, sarg1, sarg2);
        });
    }
}

void VcxWrapperCommonStringOptStringCallback(vcx_command_handle_t xcommand_handle,
                                             vcx_error_t err,
                                             const char *const arg1,
                                             const char *const arg2) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSString *, NSString *) = (void (^)(NSError *, NSString *, NSString *)) block;

    NSString *sarg1 = nil;
    if (arg1) {
        sarg1 = [NSString stringWithUTF8String:arg1];
    }
    NSString *sarg2 = nil;
    if (arg2) {
        sarg2 = [NSString stringWithUTF8String:arg2];
    }

    if (completion) {
        dispatch_async(dispatch_get_main_queue(), ^{
            NSError *error = [NSError errorFromVcxError:err];
            completion(error, sarg1, sarg2);
        });
    }
}

void VcxWrapperCommonStringOptStringOptStringCallback(vcx_command_handle_t xcommand_handle,
                                                      vcx_error_t err,
                                                      const char *const arg1,
                                                      const char *const arg2,
                                                      const char *const arg3) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSString *, NSString *, NSString *) = (void (^)(NSError *, NSString *, NSString *, NSString *)) block;

    NSString *sarg1 = nil;
    if (arg1) {
        sarg1 = [NSString stringWithUTF8String:arg1];
    }
    NSString *sarg2 = nil;
    if (arg2) {
        sarg2 = [NSString stringWithUTF8String:arg2];
    }
    NSString *sarg3 = nil;
    if (arg3) {
        sarg3 = [NSString stringWithUTF8String:arg3];
    }

    if (completion) {
        dispatch_async(dispatch_get_main_queue(), ^{
            NSError *error = [NSError errorFromVcxError:err];
            completion(error, sarg1, sarg2, sarg3);
        });
    }
}

void VcxWrapperCommonStringStringStringCallback(vcx_command_handle_t xcommand_handle,
                                                vcx_error_t err,
                                                const char *const arg1,
                                                const char *const arg2,
                                                const char *const arg3) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSString *, NSString *, NSString *) = (void (^)(NSError *, NSString *, NSString *, NSString *)) block;

    NSString *sarg1 = nil;
    if (arg1) {
        sarg1 = [NSString stringWithUTF8String:arg1];
    }
    NSString *sarg2 = nil;
    if (arg2) {
        sarg2 = [NSString stringWithUTF8String:arg2];
    }
    NSString *sarg3 = nil;
    if (arg3) {
        sarg3 = [NSString stringWithUTF8String:arg3];
    }

    if (completion) {
        dispatch_async(dispatch_get_main_queue(), ^{
            NSError *error = [NSError errorFromVcxError:err];
            completion(error, sarg1, sarg2, sarg3);
        });
    }
}

/// Arguments arg1 and arg2 will be converted to nsdata
void VcxWrapperCommonDataCallback(vcx_command_handle_t xcommand_handle,
                                  vcx_error_t err,
                                  const uint8_t *const arg1,
                                  uint32_t arg2) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSData *) = (void (^)(NSError *, NSData *)) block;

    NSData *sarg = [NSData dataWithBytes:arg1 length:arg2];

    if (completion) {
        dispatch_async(dispatch_get_main_queue(), ^{
            NSError *error = [NSError errorFromVcxError:err];
            completion(error, sarg);
        });
    }
}

void VcxWrapperCommonStringDataCallback(vcx_command_handle_t xcommand_handle,
                                        vcx_error_t err,
                                        const char *const arg1,
                                        const uint8_t *const arg2,
                                        uint32_t arg3) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSString *, NSData *) = (void (^)(NSError *, NSString *, NSData *)) block;

    NSString *sarg1 = nil;
    if (arg1) {
        sarg1 = [NSString stringWithUTF8String:arg1];
    }
    NSData *sarg2 = nil;
    if (arg2) {
        sarg2 = [NSData dataWithBytes:arg2 length:arg3];
    }

    if (completion) {
        dispatch_async(dispatch_get_main_queue(), ^{
            NSError *error = [NSError errorFromVcxError:err];
            completion(error, sarg1, sarg2);
        });
    }
}

void VcxWrapperCommonStringStringLongCallback(vcx_command_handle_t xcommand_handle,
                                              vcx_error_t err,
                                              const char *arg1,
                                              const char *arg2,
                                              unsigned long long arg3) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSString *, NSString *, NSNumber *) = (void (^)(NSError *, NSString *, NSString *, NSNumber *)) block;
    NSString *sarg1 = nil;
    if (arg1) {
        sarg1 = [NSString stringWithUTF8String:arg1];
    }
    NSString *sarg2 = nil;
    if (arg2) {
        sarg2 = [NSString stringWithUTF8String:arg2];
    }
    NSNumber *sarg3 = [NSNumber numberWithUnsignedLongLong:arg3];


    if (completion) {
        dispatch_async(dispatch_get_main_queue(), ^{
            NSError *error = [NSError errorFromVcxError:err];
            completion(error, sarg1, sarg2, sarg3);
        });
    }
}

void VcxWrapperCommonNumberStringCallback(vcx_command_handle_t xcommand_handle,
                                          vcx_error_t err,
                                          vcx_command_handle_t h,
                                          const char *const arg2
                                          ) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSNumber *, NSString *) = (void (^)(NSError *, NSNumber *, NSString *)) block;

    NSString *sarg2 = nil;
    if (arg2) {
        sarg2 = [NSString stringWithUTF8String:arg2];
    }

    if (completion) {
        dispatch_async(dispatch_get_main_queue(), ^{
            NSError *error = [NSError errorFromVcxError:err];
            NSNumber *handle = [NSNumber numberWithUnsignedInt:h];
            completion(error, handle, sarg2);
        });
    }
}

@implementation ConnectMeVcx

- (vcx_error_t) vcxInitThreadpool:(NSString *)config
{
    const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];
    return vcx_init_threadpool(config_char);
}

- (void) createWallet: (NSString *) config
            completion: (void (^)(NSError *error)) completion
{
   vcx_error_t ret;

   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];
   ret = vcx_create_wallet(handle, config_char, VcxWrapperCommonCallback);
   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret]);
       });
   }
}

- (void) openMainWallet: (NSString *) config
            completion: (void (^)(NSError *error, NSNumber *handle)) completion
{
   vcx_error_t ret;

   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];
   ret = vcx_open_main_wallet(handle, config_char, VcxWrapperCommonSignedHandleCallback);
   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret], nil);
       });
   }
}

- (void) closeMainWallet:(void (^)(NSError *error)) completion
{
   vcx_error_t ret;

   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   ret = vcx_close_main_wallet(handle, VcxWrapperCommonCallback);
   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret]);
       });
   }
}

- (void) vcxOpenMainPool: (NSString *) config
            completion: (void (^)(NSError *error)) completion
{
   vcx_error_t ret;

   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];
   ret = vcx_open_main_pool(handle, config_char, VcxWrapperCommonCallback);
   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret]);
       });
   }
}

- (void)vcxProvisionCloudAgent:(NSString *)config
               completion:(void (^)(NSError *error, NSString *config))completion
{
    const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_provision_cloud_agent(handle, config_char, VcxWrapperCommonStringCallback);
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            NSLog(@"ERROR: agentProvision: calling completion");
            completion([NSError errorFromVcxError: ret], nil);
        });
    }
}

- (void)vcxCreateAgencyClientForMainWallet:(NSString *)config
               completion:(void (^)(NSError *error))completion
{
    const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_create_agency_client_for_main_wallet(handle, config_char, VcxWrapperCommonCallback);
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            NSLog(@"ERROR: vcxCreateAgencyClientForMainWallet: calling completion");
            completion([NSError errorFromVcxError: ret]);
        });
    }
}

- (void) updateWebhookUrl:(NSString *) notification_webhook_url
           withCompletion:(void (^)(NSError *error))completion;
{
    const char *notification_webhook_url_char = [notification_webhook_url cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_update_webhook_url(handle, notification_webhook_url_char, VcxWrapperCommonCallback);
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            NSLog(@"ERROR: vcx_update_webhook_url: calling completion");
            completion([NSError errorFromVcxError: ret]);
        });
    }
}

- (NSString *)errorCMessage:(NSInteger) errorCode {
    const char * ret = vcx_error_c_message(errorCode);

    NSString *message = nil;

    if (ret) {
        message = [NSString stringWithUTF8String:ret];
    }

    return message;
}

- (void)connectionCreateWithInvite:(NSString *)invitationId
                inviteDetails:(NSString *)inviteDetails
             completion:(void (^)(NSError *error, NSNumber *connectionHandle)) completion
{
   vcx_error_t ret;

   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char *invitationId_char = [invitationId cStringUsingEncoding:NSUTF8StringEncoding];
   const char *inviteDetails_char = [inviteDetails cStringUsingEncoding:NSUTF8StringEncoding];
   ret = vcx_connection_create_with_invite(handle, invitationId_char, inviteDetails_char, VcxWrapperCommonNumberCallback);
   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret], nil);
       });
   }
}

- (void)connectionConnect: (NSNumber *)connectionHandle
        connectionType: (NSString *) connectionType
            completion: (void (^)(NSError *error)) completion
{
   vcx_error_t ret;

   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char *connectionType_char = [connectionType cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_connection_connect(handle, connectionHandle.unsignedIntValue, connectionType_char, VcxWrapperCommonCallback);
   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret]);
       });
   }
}

- (void)connectionGetState:(NSNumber *)connectionHandle
                completion:(void (^)(NSError *error, NSNumber *state))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_connection_get_state(handle, connectionHandle.unsignedIntValue, VcxWrapperCommonNumberCallback);
    
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];
        
        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret], nil);
        });
    }
}

- (void)connectionUpdateState:(NSNumber *)connectionHandle
                   completion:(void (^)(NSError *error, NSNumber *state))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_connection_update_state(handle, connectionHandle.unsignedIntValue, VcxWrapperCommonNumberCallback);
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret], nil);
        });
    }
}

- (void)connectionSerialize:(NSNumber *)connectionHandle
                  completion:(void (^)(NSError *error, NSString *serializedConnection))completion{
    vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_connection_serialize(handle, connectionHandle.unsignedIntValue, VcxWrapperCommonStringCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret], nil);
       });
   }
}

- (void)connectionDeserialize:(NSString *)serializedConnection
                    completion:(void (^)(NSError *error, NSNumber *connectionHandle))completion{
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *serialized_connection=[serializedConnection cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_connection_deserialize(handle, serialized_connection, VcxWrapperCommonNumberCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret], nil);
       });
   }
}

- (int)connectionRelease:(NSNumber *)connectionHandle {
    return vcx_connection_release(connectionHandle.unsignedIntValue);
}

- (void)deleteConnection:(NSNumber *)connectionHandle
          withCompletion:(void (^)(NSError *error))completion
{
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_connection_delete_connection(handle, connectionHandle.unsignedIntValue, VcxWrapperCommonCallback);
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            NSLog(@"deleteConnection: calling completion");
            completion([NSError errorFromVcxError: ret]);
        });
    }
}

- (void)connectionGetPwDid:(NSNumber *)connectionHandle
                   completion:(void (^)(NSError *error, NSString *pwDid))completion
{
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_connection_get_pw_did(handle,connectionHandle.unsignedIntValue, VcxWrapperCommonStringCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret], nil);
       });
   }
}

- (void)connectionGetTheirPwDid:(NSNumber *)connectionHandle
                   completion:(void (^)(NSError *error, NSString *theirPwDid))completion
{
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_connection_get_their_pw_did(handle,connectionHandle.unsignedIntValue, VcxWrapperCommonStringCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret], nil);
       });
   }
}

- (void)connectionSendMessage:(NSNumber *)connectionHandle
                  withMessage:(NSString *)message
       withSendMessageOptions:(NSString *)sendMessageOptions
               withCompletion:(void (^)(NSError *error, NSString *msg_id))completion
{
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *message_ctype = [message cStringUsingEncoding:NSUTF8StringEncoding];
    const char *sendMessageOptions_ctype = [sendMessageOptions cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_connection_send_message(handle,
                                                  connectionHandle.unsignedIntValue,
                                                  message_ctype,
                                                  sendMessageOptions_ctype,
                                                  VcxWrapperCommonStringCallback);
    if (ret != 0)
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor:handle];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret], nil);
        });
    }
}

- (void)connectionSignData:(NSNumber *)connectionHandle
                  withData:(NSData *)dataRaw
            withCompletion:(void (^)(NSError *error, NSData *signature_raw))completion
{
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    uint8_t *data_raw = (uint8_t *) [dataRaw bytes];
    uint32_t data_length = (uint32_t) [dataRaw length];

    vcx_error_t ret = vcx_connection_sign_data(handle,
                                               connectionHandle.unsignedIntValue,
                                               data_raw,
                                               data_length,
                                               VcxWrapperCommonDataCallback);
    if (ret != 0)
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor:handle];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret], nil);
        });
    }
}

- (void)connectionVerifySignature:(NSNumber *)connectionHandle
                         withData:(NSData *)dataRaw
                withSignatureData:(NSData *)signatureRaw
                   withCompletion:(void (^)(NSError *error, vcx_bool_t valid))completion
{
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    uint8_t *data_raw = (uint8_t *) [dataRaw bytes];
    uint32_t data_length = (uint32_t) [dataRaw length];

    uint8_t *signature_raw = (uint8_t *) [signatureRaw bytes];
    uint32_t signature_length = (uint32_t) [signatureRaw length];

    vcx_error_t ret = vcx_connection_verify_signature(handle,
                                                      connectionHandle.unsignedIntValue,
                                                      data_raw,
                                                      data_length,
                                                      signature_raw,
                                                      signature_length,
                                                      VcxWrapperCommonBoolCallback);
    if (ret != 0)
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor:handle];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret], false);
        });
    }
}

- (void)connectionDownloadMessages:(NSNumber *)connectionHandle
                    messageStatus:(NSString *)messageStatus
                            uid_s:(NSString *)uid_s
                      completion:(void (^)(NSError *error, NSString* messages))completion {
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * message_status = [messageStatus cStringUsingEncoding:NSUTF8StringEncoding];
    const char * uids = [uid_s cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_connection_messages_download(handle, connectionHandle.unsignedIntValue, message_status, uids, VcxWrapperCommonStringCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret], nil);
        });
    }

}

- (void)getCredential:(NSNumber *)credentialHandle
           completion:(void (^)(NSError *error, NSString *credential))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_get_credential(handle, credentialHandle.unsignedIntValue, VcxWrapperCommonStringCallback);

    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret], nil);
       });
    }
}

- (void)credentialCreateWithOffer:(NSString *)sourceId
            offer:(NSString *)credentialOffer
           completion:(void (^)(NSError *error, NSNumber *credentialHandle))completion{
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char * credential_offer=[credentialOffer cStringUsingEncoding:NSUTF8StringEncoding];
   const char * source_id = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_credential_create_with_offer(handle, source_id,credential_offer, VcxWrapperCommonNumberCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret], 0);
       });
   }
}

- (void)credentialCreateWithMsgid:(NSString *)sourceId
                 connectionHandle:(NSNumber *)connectionHandle
                            msgId:(NSString *)msgId
                       completion:(void (^)(NSError *error, NSNumber *credentialHandle, NSString *credentialOffer))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * source_id = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
    const char * msg_id= [msgId cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_credential_create_with_msgid(handle, source_id, connectionHandle.unsignedIntValue, msg_id, VcxWrapperCommonNumberStringCallback);

    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret], 0, nil);
       });
    }
}

- (void)credentialSendRequest:(NSNumber *)credentialHandle
             connectionHandle:(NSNumber *)connectionHandle
                paymentHandle:(NSNumber *)paymentHandle
                   completion:(void (^)(NSError *))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_credential_send_request(handle,
                                      credentialHandle.unsignedIntValue,
                                      connectionHandle.unsignedIntValue,
                                      paymentHandle.unsignedIntValue,
                                      VcxWrapperCommonCallback);

    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret]);
       });
    }
}
- (void)credentialGetState:(NSNumber *)credentialHandle
                completion:(void (^)(NSError *error, NSNumber *state))completion{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_credential_get_state(handle,
                                   credentialHandle.unsignedIntValue,
                                   VcxWrapperCommonNumberCallback);

    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret], 0);
       });
    }
}

- (void)credentialUpdateStateV2:(NSNumber *)credentialHandle
               connectionHandle:(NSNumber *)connectionHandle
               completion:(void (^)(NSError *error, NSNumber *state))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_v2_credential_update_state(handle,
                                         credentialHandle.unsignedIntValue,
                                         connectionHandle.unsignedIntValue,
                                         VcxWrapperCommonNumberCallback);

    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret], 0);
       });
    }
}

- (void)credentialGetOffers:(NSNumber *)connectionHandle
                   completion:(void (^)(NSError *error, NSString *offers))completion{
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_credential_get_offers(handle,
                                    connectionHandle.unsignedIntValue,
                                    VcxWrapperCommonStringCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret],nil);
       });
   }
}

- (void)credentialGetAttributes:(NSNumber *)credentialHandle
                   completion:(void (^)(NSError *error, NSString *attrs))completion{
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_credential_get_attributes(handle,
                                        credentialHandle.unsignedIntValue,
                                        VcxWrapperCommonStringCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret],nil);
       });
   }
}

- (void)credentialGetAttachment:(NSNumber *)credentialHandle
                   completion:(void (^)(NSError *error, NSString *attach))completion{
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_credential_get_attachment(handle,
                                        credentialHandle.unsignedIntValue,
                                        VcxWrapperCommonStringCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret],nil);
       });
   }
}

- (void)credentialGetTailsLocation:(NSNumber *)credentialHandle
                   completion:(void (^)(NSError *error, NSString *tailsLocation))completion{
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

   ret = vcx_credential_get_tails_location(handle,
                                           credentialHandle.unsignedIntValue,
                                           VcxWrapperCommonStringCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret],nil);
       });
   }
}

- (void)credentialGetTailsHash:(NSNumber *)credentialHandle
                   completion:(void (^)(NSError *error, NSString *tailsHash))completion{
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

   ret = vcx_credential_get_tails_hash(handle,
                                       credentialHandle.unsignedIntValue,
                                       VcxWrapperCommonStringCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret],nil);
       });
   }
}

- (void)credentialGetRevRegId:(NSNumber *)credentialHandle
                   completion:(void (^)(NSError *error, NSString *revRegId))completion{
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

   ret = vcx_credential_get_rev_reg_id(handle,
                                       credentialHandle.unsignedIntValue,
                                       VcxWrapperCommonStringCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret],nil);
       });
   }
}

- (void)credentialIsRevokable:(NSNumber *)credentialHandle
                   completion:(void (^)(NSError *error, BOOL revokable))completion{
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

   ret = vcx_credential_is_revokable(handle,
                                     credentialHandle.unsignedIntValue,
                                     VcxWrapperCommonBoolCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret], false);
       });
   }
}

- (void)generateProof:(NSString *)proofRequestId
       requestedAttrs:(NSString *)requestedAttrs
  requestedPredicates:(NSString *)requestedPredicates
            proofName:(NSString *)proofName
           completion:(void (^)(NSError *error, NSString *proofHandle))completion;
{
    vcx_error_t ret;

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *proofRequestId_char = [proofRequestId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *requestedAttrs_char = [requestedAttrs cStringUsingEncoding:NSUTF8StringEncoding];
    const char *requestedPredicates_char = [requestedPredicates cStringUsingEncoding:NSUTF8StringEncoding];
    const char *proofName_char = [proofName cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_proof_create(handle,
                           proofRequestId_char,
                           requestedAttrs_char,
                           requestedPredicates_char,
                           proofName_char,
                           VcxWrapperCommonStringCallback);

    if ( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret], nil);
       });
    }
}

- (void)credentialSerialize:(NSNumber *)credentialHandle
                  completion:(void (^)(NSError *error, NSString *state))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_credential_serialize(handle,
                                   credentialHandle.unsignedIntValue,
                                   VcxWrapperCommonStringCallback);

    if ( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret], nil);
       });
    }
}

- (void)credentialDeserialize:(NSString *)serializedCredential
                    completion:(void (^)(NSError *error, NSNumber *credentialHandle))completion{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *serialized_credential = [serializedCredential cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_credential_deserialize(handle,
                                     serialized_credential,
                                     VcxWrapperCommonNumberCallback);

    if ( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret], 0);
       });
    }
}

- (int)credentialRelease:(NSNumber *) credentialHandle {
    return vcx_credential_release(credentialHandle.unsignedIntValue);
}


- (void)deleteCredential:(NSNumber *)credentialHandle
          completion:(void (^)(NSError *error))completion
{
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_delete_credential(handle,
                                            credentialHandle.unsignedIntValue,
                                            VcxWrapperCommonCallback);
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret]);
        });
    }
}

- (void)exportWallet:(NSString *)exportPath
            encryptWith:(NSString *)encryptionKey
           completion:(void (^)(NSError *error))completion {
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char * export_path=[exportPath cStringUsingEncoding:NSUTF8StringEncoding];
   const char * encryption_key = [encryptionKey cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_wallet_export(handle, export_path, encryption_key, VcxWrapperCommonCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret]);
       });
   }
}

- (void)importWallet:(NSString *)config
           completion:(void (^)(NSError *error))completion {
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   ret = vcx_wallet_import(handle, [config cStringUsingEncoding:NSUTF8StringEncoding], VcxWrapperCommonCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret]);
       });
   }
}

- (void)addRecordWallet:(NSString *)recordType
               recordId:(NSString *)recordId
            recordValue:(NSString *)recordValue
               tagsJson:(NSString *)tagsJson
             completion:(void (^)(NSError *error))completion {
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   if (!tagsJson.length) tagsJson = @"{}";
    
   const char * record_type =[recordType cStringUsingEncoding:NSUTF8StringEncoding];
   const char * record_id = [recordId cStringUsingEncoding:NSUTF8StringEncoding];
   const char * record_value =[recordValue cStringUsingEncoding:NSUTF8StringEncoding];
   const char * tags_json =[tagsJson cStringUsingEncoding:NSUTF8StringEncoding];
    
    ret = vcx_wallet_add_record(handle, record_type, record_id, record_value, tags_json, VcxWrapperCommonCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret]);
       });
   }
}

- (void)getRecordWallet:(NSString *)recordType
               recordId:(NSString *)recordId
            optionsJson:(NSString *)optionsJson
             completion:(void (^)(NSError *error, NSString* walletValue))completion {
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    
   if (!optionsJson.length) optionsJson = @"{}";
   const char * record_type =[recordType cStringUsingEncoding:NSUTF8StringEncoding];
   const char * record_id = [recordId cStringUsingEncoding:NSUTF8StringEncoding];
   const char * options_json = [optionsJson cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_wallet_get_record(handle, record_type, record_id, options_json, VcxWrapperCommonStringCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret], nil);
       });
   }
}

- (int)vcxShutdown:(BOOL *) deleteWallet {
    int delete_wallet = deleteWallet;
    return vcx_shutdown(delete_wallet);
}

- (void)deleteRecordWallet:(NSString *)recordType
            recordId:(NSString *)recordId
           completion:(void (^)(NSError *error))completion {
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char * record_type =[recordType cStringUsingEncoding:NSUTF8StringEncoding];
   const char * record_id = [recordId cStringUsingEncoding:NSUTF8StringEncoding];
   ret = vcx_wallet_delete_record(handle, record_type, record_id, VcxWrapperCommonCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret]);
       });
   }
}

- (void)updateRecordWallet:(NSString *)recordType
              withRecordId:(NSString *)recordId
           withRecordValue:(NSString *) recordValue
            withCompletion:(void (^)(NSError *error))completion {

    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * record_type =[recordType cStringUsingEncoding:NSUTF8StringEncoding];
    const char * record_id = [recordId cStringUsingEncoding:NSUTF8StringEncoding];
    const char * record_value =[recordValue cStringUsingEncoding:NSUTF8StringEncoding];

    ret = vcx_wallet_update_record_value(handle, record_type, record_id, record_value, VcxWrapperCommonCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret]);
        });
    }
}

- (void)addRecordTagsWallet:(NSString *)recordType
               recordId:(NSString *)recordId
            tagsJson:(NSString *) tagsJson
             completion:(void (^)(NSError *error))completion {
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char * record_type =[recordType cStringUsingEncoding:NSUTF8StringEncoding];
   const char * record_id = [recordId cStringUsingEncoding:NSUTF8StringEncoding];
   const char * tags_json =[tagsJson cStringUsingEncoding:NSUTF8StringEncoding];

    ret = vcx_wallet_add_record_tags(handle, record_type, record_id, tags_json, VcxWrapperCommonCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret]);
       });
   }
}

- (void)updateRecordTagsWallet:(NSString *)recordType
              recordId:(NSString *)recordId
           tagsJson:(NSString *) tagsJson
            completion:(void (^)(NSError *error))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * record_type =[recordType cStringUsingEncoding:NSUTF8StringEncoding];
    const char * record_id = [recordId cStringUsingEncoding:NSUTF8StringEncoding];
    const char * tags_json =[tagsJson cStringUsingEncoding:NSUTF8StringEncoding];

    ret = vcx_wallet_update_record_tags(handle, record_type, record_id, tags_json, VcxWrapperCommonCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret]);
        });
    }
}

- (void)deleteRecordTagsWallet:(NSString *)recordType
            recordId:(NSString *)recordId
            tagNamesJson:(NSString *)tagNamesJson
           completion:(void (^)(NSError *error))completion {
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char * record_type =[recordType cStringUsingEncoding:NSUTF8StringEncoding];
   const char * record_id = [recordId cStringUsingEncoding:NSUTF8StringEncoding];
   const char * tag_names_json = [tagNamesJson cStringUsingEncoding:NSUTF8StringEncoding];

   ret = vcx_wallet_delete_record_tags(handle, record_type, record_id, tag_names_json,  VcxWrapperCommonCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret]);
       });
   }
}

- (void)openSearchWallet:(NSString *)recordType
               queryJson:(NSString *)queryJson
            optionsJson:(NSString *)optionsJson
             completion:(void (^)(NSError *error, NSNumber *searchHandle))completion {
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

   if (!queryJson.length) queryJson = @"{}";
   if (!optionsJson.length) optionsJson = @"{}";

   const char * record_type =[recordType cStringUsingEncoding:NSUTF8StringEncoding];
   const char * query_json = [queryJson cStringUsingEncoding:NSUTF8StringEncoding];
   const char * options_json =[optionsJson cStringUsingEncoding:NSUTF8StringEncoding];

    ret = vcx_wallet_open_search(handle, record_type, query_json, options_json, VcxWrapperCommonNumberCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret], 0);
       });
   }
}

- (void)searchNextRecordsWallet:(NSNumber *)searchHandle
              count:(NSNumber *)count
            completion:(void (^)(NSError *error, NSString* records))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_wallet_search_next_records(handle,
                                         searchHandle.unsignedIntValue,
                                         count.intValue,
                                         VcxWrapperCommonStringCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret], nil);
        });
    }
}

- (void)closeSearchWallet:(NSNumber *)searchHandle
           completion:(void (^)(NSError *error))completion {
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

   ret = vcx_wallet_close_search(handle,
                                 searchHandle.unsignedIntValue,
                                 VcxWrapperCommonCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret]);
       });
   }
}

- (void)proofGetRequests:(NSNumber *)connectionHandle
                   completion:(void (^)(NSError *error, NSString *requests))completion{
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_disclosed_proof_get_requests(handle,
                                           connectionHandle.unsignedIntValue,
                                           VcxWrapperCommonStringCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret],nil);
       });
   }
}

- (void)proofGetProofRequestAttachment:(NSNumber *)proofHandle
                   completion:(void (^)(NSError *error, NSString *attach))completion{
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_disclosed_proof_get_proof_request_attachment(handle,
                                                           proofHandle.unsignedIntValue,
                                                           VcxWrapperCommonStringCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       dispatch_async(dispatch_get_main_queue(), ^{
           completion([NSError errorFromVcxError: ret],nil);
       });
   }
}

- (void) proofCreateWithMsgId:(NSString *)sourceId
         withConnectionHandle:(NSNumber *)connectionHandle
                    withMsgId:(NSString *)msgId
               withCompletion:(void (^)(NSError *error, NSNumber *proofHandle, NSString *proofRequest))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *source_id = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *msg_id = [msgId cStringUsingEncoding:NSUTF8StringEncoding];

    ret = vcx_disclosed_proof_create_with_msgid(handle,
                                                source_id,
                                                connectionHandle.unsignedIntValue,
                                                msg_id,
                                                VcxWrapperCommonNumberStringCallback);

    if ( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret], nil, nil);
        });
    }
}

- (void) proofRetrieveCredentials:(NSNumber *)proofHandle
                   withCompletion:(void (^)(NSError *error, NSString *matchingCredentials))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_disclosed_proof_retrieve_credentials(handle,
                                                   proofHandle.unsignedIntValue,
                                                   VcxWrapperCommonStringCallback);

    if ( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret], nil);
        });
    }
}

- (void) proofGenerate:(NSNumber *)proofHandle
withSelectedCredentials:(NSString *)selectedCredentials
 withSelfAttestedAttrs:(NSString *)selfAttestedAttributes
        withCompletion:(void (^)(NSError *error))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *selected_credentials = [selectedCredentials cStringUsingEncoding:NSUTF8StringEncoding];
    const char *self_attested_attributes = [selfAttestedAttributes cStringUsingEncoding:NSUTF8StringEncoding];

    ret = vcx_disclosed_proof_generate_proof(handle,
                                             proofHandle.unsignedIntValue,
                                             selected_credentials,
                                             self_attested_attributes,
                                             VcxWrapperCommonCallback);

    if ( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret]);
        });
    }
}

- (void) proofSend:(vcx_proof_handle_t)proof_handle
withConnectionHandle:(vcx_connection_handle_t)connection_handle
    withCompletion:(void (^)(NSError *error))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_disclosed_proof_send_proof(handle, proof_handle, connection_handle, VcxWrapperCommonCallback);

    if ( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret]);
        });
    }
}

- (void)proofGetState:(NSNumber *)proofHandle
                completion:(void (^)(NSError *error, NSNumber *state))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_disclosed_proof_get_state(handle,
                                        proofHandle.unsignedIntValue,
                                        VcxWrapperCommonNumberCallback);
    
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];
        
        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret], nil);
        });
    }
}

- (void) proofUpdateStateV2:(NSNumber *) proofHandle
           connectionHandle:(NSNumber *)connectionHandle
                 completion:(void (^)(NSError *error, NSNumber *state))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_v2_disclosed_proof_update_state(handle,
                                              proofHandle.unsignedIntValue,
                                              connectionHandle.unsignedIntValue,
                                              VcxWrapperCommonNumberCallback);
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret], nil);
        });
    }
}

- (void) proofReject:(NSNumber *)proof_handle
withConnectionHandle:(NSNumber *)connection_handle
      withCompletion:(void (^)(NSError *error))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor: completion];

    ret = vcx_disclosed_proof_reject_proof(handle,
                                           proof_handle.unsignedIntValue,
                                           connection_handle.unsignedIntValue,
                                           VcxWrapperCommonCallback);

    if (ret != 0)
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret]);
        });
    }
}

- (void) getProofMsg:(NSNumber *) proofHandle
         withCompletion:(void (^)(NSError *error, NSString *proofMsg))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_disclosed_proof_get_proof_msg(handle,
                                            proofHandle.unsignedIntValue,
                                            VcxWrapperCommonStringCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret], nil);
        });
    }
}

- (void) getRejectMsg:(NSNumber *) proofHandle
         withCompletion:(void (^)(NSError *error, NSString *rejectMsg))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_disclosed_proof_get_reject_msg(handle,
                                             proofHandle.unsignedIntValue,
                                             VcxWrapperCommonStringCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret], nil);
        });
    }
}

- (void) connectionRedirect: (NSNumber *) redirect_connection_handle
        withConnectionHandle: (NSNumber *) connection_handle
        withCompletion: (void (^)(NSError *error)) completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor: completion];

    ret = vcx_connection_redirect(handle,
                                  connection_handle.unsignedIntValue,
                                  redirect_connection_handle.unsignedIntValue,
                                  VcxWrapperCommonCallback);

    if (ret != 0)
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret]);
        });
    }
}

- (void) getRedirectDetails: (NSNumber *) connection_handle
        withCompletion: (void (^)(NSError *error, NSString *redirectDetails)) completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_connection_get_redirect_details(handle,
                                              connection_handle.unsignedIntValue,
                                              VcxWrapperCommonStringCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret], nil);
        });
    }
}

- (void) proofCreateWithRequest:(NSString *) source_id
               withProofRequest:(NSString *) proofRequest
                 withCompletion:(void (^)(NSError *error, NSNumber *proofHandle))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *sourceId = [source_id cStringUsingEncoding:NSUTF8StringEncoding];
    const char *proof_request = [proofRequest cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_disclosed_proof_create_with_request(handle,
                                                  sourceId,
                                                  proof_request,
                                                  VcxWrapperCommonNumberCallback);

    if ( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret], 0);
        });
    }
}

- (void) proofSerialize:(NSNumber *) proofHandle
         withCompletion:(void (^)(NSError *error, NSString *proof_request))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_disclosed_proof_serialize(handle,
                                        proofHandle.unsignedIntValue,
                                        VcxWrapperCommonStringCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret], nil);
        });
    }
}

- (void) proofDeserialize:(NSString *) serializedProof
           withCompletion:(void (^)(NSError *error, NSNumber *proofHandle)) completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *serialized_proof = [serializedProof cStringUsingEncoding:NSUTF8StringEncoding];

    ret = vcx_disclosed_proof_deserialize(handle, serialized_proof, VcxWrapperCommonNumberCallback);

    if ( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret], 0);
        });
    }
}

- (int)proofRelease:(NSNumber *) proofHandle {
    return vcx_disclosed_proof_release(proofHandle.unsignedIntValue);
}

- (void)createPaymentAddress:(NSString *)seed
              withCompletion:(void (^)(NSError *error, NSString *address))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    const char *c_seed = [seed cStringUsingEncoding:NSUTF8StringEncoding];

    ret = vcx_wallet_create_payment_address(handle, c_seed, VcxWrapperCommonStringCallback);

    if ( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret], nil);
        });
    }
}

- (void)getTokenInfo:(NSNumber *)payment_handle
      withCompletion:(void (^)(NSError *error, NSString *tokenInfo))completion
{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_wallet_get_token_info(handle,
                                    payment_handle.unsignedIntValue,
                                    VcxWrapperCommonStringCallback);

    if ( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret], nil);
        });
    }
}

- (void)sendTokens:(NSNumber *)payment_handle
        withTokens:(NSString *)tokens
     withRecipient:(NSString *)recipient
    withCompletion:(void (^)(NSError *error, NSString *recipient))completion
{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    const char* c_recipient = [recipient cStringUsingEncoding:NSUTF8StringEncoding];
    const char* c_tokens = [tokens cStringUsingEncoding:NSUTF8StringEncoding];

    ret = vcx_wallet_send_tokens(handle,
                                 payment_handle.unsignedIntValue,
                                 c_tokens,
                                 c_recipient,
                                 VcxWrapperCommonStringCallback);

    if ( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret], nil);
        });
    }
}

- (void)downloadMessages:(NSString *)messageStatus
                    uid_s:(NSString *)uid_s
                  pwdids:(NSString *)pwdids
              completion:(void (^)(NSError *error, NSString* messages))completion{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * message_status = [messageStatus cStringUsingEncoding:NSUTF8StringEncoding];
    const char * uids = [uid_s cStringUsingEncoding:NSUTF8StringEncoding];
    const char * pw_dids = [pwdids cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_messages_download(handle, message_status, uids, pw_dids, VcxWrapperCommonStringCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret], nil);
        });
    }
}

- (void)downloadMessagesV2:(NSString *)connectionHandles
            messageStatus:(NSString *)messageStatus
                    uid_s:(NSString *)uid_s
              completion:(void (^)(NSError *error, NSString* messages))completion{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * message_status = [messageStatus cStringUsingEncoding:NSUTF8StringEncoding];
    const char * uids = [uid_s cStringUsingEncoding:NSUTF8StringEncoding];
    const char * connection_handles = [connectionHandles cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_v2_messages_download(handle, connection_handles, message_status, uids, VcxWrapperCommonStringCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret], nil);
        });
    }
}

- (void)updateMessages:(NSString *)messageStatus
                 pwdidsJson:(NSString *)pwdidsJson
              completion:(void (^)(NSError *error))completion{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * message_status = [messageStatus cStringUsingEncoding:NSUTF8StringEncoding];
    const char * msg_json = [pwdidsJson cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_messages_update_status(handle, message_status, msg_json, VcxWrapperCommonCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret]);
        });
    }
}

- (void) getLedgerFees:(void(^)(NSError *error, NSString *fees)) completion
{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_ledger_get_fees(handle, VcxWrapperCommonStringCallback);

    if (ret != 0)
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret], nil);
        });
    }
}

/// Retrieve author agreement set on the Ledger
///
/// #params
/// completion: Callback that provides array of matching messages retrieved
///
/// #Returns
/// Error code as a u32
- (void) getTxnAuthorAgreement:(void(^)(NSError *error, NSString *authorAgreement)) completion
{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_get_ledger_author_agreement(handle, VcxWrapperCommonStringCallback);

    if (ret != 0)
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret], nil);
        });
    }
}


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
- (vcx_error_t) activateTxnAuthorAgreement:(NSString *)text
                               withVersion:(NSString *)version
                                  withHash:(NSString *)hash
                             withMechanism:(NSString *)mechanism
                             withTimestamp:(long)timestamp
{
    return vcx_set_active_txn_author_agreement_meta(
        [text UTF8String],
        [version UTF8String],
        [hash UTF8String],
        [mechanism UTF8String],
        timestamp
    );
}

@end
