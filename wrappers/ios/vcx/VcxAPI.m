//
//  init.m
//  vcx
//
//  Created by GuestUser on 4/30/18.
//  Copyright © 2018 GuestUser. All rights reserved.
//

#import <Foundation/Foundation.h>
#import <vcx/vcx.h>
#import "VcxAPI.h"
#import "NSError+VcxError.h"
#import "VcxCallbacks.h"
#import "VcxWrapperCallbacks.h"
#import "libvcx.h"
#import "IndySdk.h"
#import "utils/IndyCallbacks.h"
#import "utils/IndySdk.h"
#import "utils/VcxLogger.h"

void checkErrorAndComplete(vcx_error_t ret, vcx_command_handle_t cmdHandle, void (^completion)()) {
    if (ret != 0) {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor:cmdHandle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion();
        });
    }
}

@implementation VcxAPI

- (void)vcxInitIssuerConfig:(NSString *)config
                 completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_init_issuer_config(handle, config_char, &VcxWrapperCbNoResponse);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (vcx_error_t)vcxPoolSetHandle:(NSNumber *)handle
                     completion:(void (^)(NSError *))completion {
    return vcx_pool_set_handle(handle.integerValue);
}

- (void)vcxEndorseTransaction:(NSString *)transaction
                   completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *transaction_char = [transaction cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_endorse_transaction(handle, transaction_char, &VcxWrapperCbNoResponse);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}


- (void)vcxRotateVerKey:(NSString *)did
             completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *did_char = [did cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_rotate_verkey(handle, did_char, &VcxWrapperCbNoResponse);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)vcxRotateVerKeyStart:(NSString *)did
                  completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *did_char = [did cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_rotate_verkey_start(handle, did_char, &VcxWrapperCbNoResponse);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)vcxRotateVerKeyApply:(NSString *)did
                  tempVerKey:(NSString *)tempVerKey
                  completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *did_char = [did cStringUsingEncoding:NSUTF8StringEncoding];
    const char *tempVerKey_char = [tempVerKey cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_rotate_verkey_apply(handle, did_char, tempVerKey_char, &VcxWrapperCbNoResponse);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}


- (void)vcxGetVerKeyFromWallet:(NSString *)did
                    completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *did_char = [did cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_get_verkey_from_wallet(handle, did_char, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxGetVerKeyFromLedger:(NSString *)did
                    completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *did_char = [did cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_get_verkey_from_ledger(handle, did_char, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxGetLedgerTxn:(NSString *)submitterDid
                  seqNo:(NSNumber *)seqNo
             completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *submitterDid_char = [submitterDid cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_get_ledger_txn(handle, submitterDid_char, seqNo.integerValue, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}


- (vcx_error_t)vcxInitThreadPool:(NSString *)config {
    const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];
    return vcx_init_threadpool(config_char);
}

- (void)createWallet:(NSString *)config
          completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_create_wallet(handle, config_char, &VcxWrapperCbNoResponse);
    if (ret != 0) {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor:handle];

        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError:ret]);
        });
    }
}

- (void)vcxConfigureIssuerWallet:(NSString *)seed
                      completion:(void (^)(NSError *, NSString *))completion {
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *seed_char = [seed cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_configure_issuer_wallet(handle, seed_char, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}


- (void)openMainWallet:(NSString *)config
            completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_open_main_wallet(handle, config_char, &VcxWrapperCbResponseSignedInt);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)closeMainWallet:(void (^)(NSError *error))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_close_main_wallet(handle, &VcxWrapperCbNoResponse);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)vcxOpenMainPool:(NSString *)config
             completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_open_main_pool(handle, config_char, &VcxWrapperCbNoResponse);
    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)vcxProvisionCloudAgent:(NSString *)config
                    completion:(void (^)(NSError *, NSString *))completion {

    const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_provision_cloud_agent(handle, config_char, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxCreateAgencyClientForMainWallet:(NSString *)config
                                completion:(void (^)(NSError *))completion {

    const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_create_agency_client_for_main_wallet(handle, config_char, &VcxWrapperCbNoResponse);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)updateWebhookUrl:(NSString *)notification_webhook_url
          withCompletion:(void (^)(NSError *))completion; {

    const char *notification_webhook_url_char = [notification_webhook_url cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_update_webhook_url(handle, notification_webhook_url_char, &VcxWrapperCbNoResponse);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (NSString *)errorCMessage:(NSNumber *)errorCode {
    const char *ret = vcx_error_c_message(errorCode.intValue);

    NSString *message = nil;

    if (ret) {
        message = [NSString stringWithUTF8String:ret];
    }

    return message;
}

- (NSString *)vcxVersion {
    return [NSString stringWithUTF8String:vcx_version()];
}

- (void)vcxSchemaSerialize:(NSNumber *)schemaHandle
                completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_schema_serialize(handle, schemaHandle.unsignedIntValue, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxSchemaDeserialize:(NSString *)serializedSchema
                  completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *serializedSchema_char = [serializedSchema cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_schema_deserialize(handle, serializedSchema_char, &VcxWrapperCbResponseUnsignedInt);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)vcxSchemaGetAttributes:(NSString *)sourceId
                    schemaId:(NSString *)schemaId
                    completion:(void (^)(NSError *, NSNumber *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *sourceId_char = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *schemaId_char = [schemaId cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_schema_get_attributes(handle, sourceId_char, schemaId_char, &VcxWrapperCbResponseUnsignedIntAndString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER, ERROR_RESPONSE_STRING);
    });
}

- (void)vcxSchemaCreate:(NSString *)sourceId
             schemaName:(NSString *)schemaName
          schemaVersion:(NSString *)schemaVersion
             schemaData:(NSString *)schemaData
          paymentHandle:(NSNumber *)paymentHandle
             completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *sourceId_char = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *schemaName_char = [schemaName cStringUsingEncoding:NSUTF8StringEncoding];
    const char *schemaVersion_char = [schemaVersion cStringUsingEncoding:NSUTF8StringEncoding];
    const char *schemaData_char = [schemaData cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_schema_create(
            handle,
            sourceId_char,
            schemaName_char,
            schemaVersion_char,
            schemaData_char,
            paymentHandle.unsignedIntValue,
            VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)vcxSchemaPrepareForEndorser:(NSString *)sourceId
                         schemaName:(NSString *)schemaName
                      schemaVersion:(NSString *)schemaVersion
                         schemaData:(NSString *)schemaData
                           endorser:(NSString *)endorser
                         completion:(void (^)(NSError *, NSNumber *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *sourceId_char = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *schemaName_char = [schemaName cStringUsingEncoding:NSUTF8StringEncoding];
    const char *schemaVersion_char = [schemaVersion cStringUsingEncoding:NSUTF8StringEncoding];
    const char *schemaData_char = [schemaData cStringUsingEncoding:NSUTF8StringEncoding];
    const char *endorser_char = [endorser cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_schema_prepare_for_endorser(
            handle,
            sourceId_char,
            schemaName_char,
            schemaVersion_char,
            schemaData_char,
            endorser_char,
            &VcxWrapperCbResponseUnsignedIntAndString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER, ERROR_RESPONSE_STRING);
    });
}


- (void)vcxSchemaGetSchemaId:(NSString *)sourceId
                schemaHandle:(NSNumber *)schemaHandle
                  completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_schema_get_schema_id(handle, schemaHandle.unsignedIntValue, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxSchemaUpdateState:(NSString *)sourceId
                schemaHandle:(NSNumber *)schemaHandle
                  completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_schema_update_state(handle, schemaHandle.unsignedIntValue, &VcxWrapperCbResponseUnsignedInt);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (int)vcxSchemaRelease:(NSNumber *)schemaHandle {
    return vcx_schema_release(schemaHandle.unsignedIntValue);
}

- (void)vcxPublicAgentCreate:(NSString *)sourceId
              institutionDid:(NSString *)institutionDid
                  completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *sourceId_char = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *institutionDid_char = [institutionDid cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_public_agent_create(handle, sourceId_char, institutionDid_char, &VcxWrapperCbResponseUnsignedInt);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}


- (void)vcxGeneratePublicInvite:(NSString *)publicDid
                          label:(NSString *)label
                     completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *publicDid_char = [publicDid cStringUsingEncoding:NSUTF8StringEncoding];
    const char *label_char = [label cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_generate_public_invite(handle, publicDid_char, label_char, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxPublicAgentDownloadConnectionRequests:(NSNumber *)agentHandle
                                            uids:(NSString *)ids
                                      completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *ids_char = [ids cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_public_agent_download_connection_requests(
            handle,
            agentHandle.unsignedIntValue,
            ids_char,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxPublicAgentDownloadMessage:(NSNumber *)agentHandle
                                  uid:(NSString *)id
                           completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *id_char = [id cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_public_agent_download_message(
            handle,
            agentHandle.unsignedIntValue,
            id_char,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxPublicAgentGetService:(NSNumber *)agentHandle
                      completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_public_agent_get_service(handle, agentHandle.unsignedIntValue, &VcxWrapperCbResponseString);
    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxPublicAgentSerialize:(NSNumber *)agentHandle
                     completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_public_agent_serialize(handle, agentHandle.unsignedIntValue, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}


- (int)vcxPublicAgentRelease:(NSNumber *)agentHandle {
    return vcx_public_agent_release(agentHandle.unsignedIntValue);
}

- (void)vcxOutOfBandSenderCreate:(NSString *)config
                      completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_out_of_band_sender_create(handle, config_char, &VcxWrapperCbResponseUnsignedInt);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)vcxOutOfBandReceiverCreate:(NSString *)message
                        completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *message_char = [message cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_out_of_band_receiver_create(handle, message_char, &VcxWrapperCbResponseUnsignedInt);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)vcxOutOfBandSenderAppendMessage:(NSNumber *)oobHandle
                                message:(NSString *)message
                             completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *message_char = [message cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_out_of_band_sender_append_message(
            handle,
            oobHandle.unsignedIntValue,
            message_char,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)vcxOutOfBandSenderAppendService:(NSNumber *)oobHandle
                                service:(NSString *)service
                             completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *service_char = [service cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_out_of_band_sender_append_service(
            handle,
            oobHandle.unsignedIntValue,
            service_char,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}


- (void)vcxOutOfBandSenderAppendServiceDid:(NSNumber *)oobHandle
                                       did:(NSString *)did
                                completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *did_char = [did cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_out_of_band_sender_append_service_did(
            handle,
            oobHandle.unsignedIntValue,
            did_char,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)vcxOutOfBandSenderGetThreadId:(NSNumber *)oobHandle
                           completion:(void (^)(NSError *, NSString *))completion {
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_out_of_band_sender_get_thread_id(
            handle,
            oobHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxOutOfBandReceiverGetThreadId:(NSNumber *)oobHandle
                             completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_out_of_band_receiver_get_thread_id(
            handle,
            oobHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxOutOfBandReceiverExtractMessage:(NSNumber *)oobHandle
                                completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_out_of_band_receiver_extract_message(
            handle,
            oobHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}


- (void)vcxOutOfBandToMessage:(NSNumber *)oobHandle
                   completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_out_of_band_to_message(
            handle,
            oobHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxOutOfBandSenderSerialize:(NSNumber *)oobHandle
                         completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_out_of_band_sender_serialize(
            handle,
            oobHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxOutOfBandReceiverSerialize:(NSNumber *)oobHandle
                           completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_out_of_band_receiver_serialize(
            handle,
            oobHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxOutOfBandSenderDeserialize:(NSString *)oobMessage
                           completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *oobMessage_char = [oobMessage cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_out_of_band_sender_deserialize(
            handle,
            oobMessage_char,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)vcxOutOfBandReceiverDeserialize:(NSString *)oobMessage
                             completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *oobMessage_char = [oobMessage cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_out_of_band_receiver_deserialize(
            handle,
            oobMessage_char,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}


- (int)vcxOutOfBandSenderRelease:(NSNumber *)agentHandle {
    return vcx_out_of_band_sender_release(agentHandle.unsignedIntValue);
}

- (int)vcxOutOfBandReceiverRelease:(NSNumber *)agentHandle {
    return vcx_out_of_band_receiver_release(agentHandle.unsignedIntValue);
}

- (void)vcxOutOfBandReceiverConnectionExists:(NSNumber *)oobHandle
                           connectionHandles:(NSString *)connectionHandles
                                  completion:(void (^)(NSError *, NSNumber *, Boolean))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *connectionHandles_char = [connectionHandles cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_out_of_band_receiver_connection_exists(
            handle,
            oobHandle.unsignedIntValue,
            connectionHandles_char,
            &VcxWrapperCbResponseUnsignedIntAndBool
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER, ERROR_RESPONSE_BOOL);
    });
}


- (void)vcxOutOfBandReceiverBuildConnection:(NSNumber *)oobHandle
                                 completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_out_of_band_receiver_build_connection(
            handle,
            oobHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxRevocationRegistryCreate:(NSString *)revRegConfig
                         completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *revRegConfig_char = [revRegConfig cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_revocation_registry_create(handle, revRegConfig_char, &VcxWrapperCbResponseUnsignedInt);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)vcxRevocationRegistryPublish:(NSNumber *)revRegHandle
                            tailsUrl:(NSString *)tailsUrl
                          completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *tailsUrl_char = [tailsUrl cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_revocation_registry_publish(handle, revRegHandle.unsignedIntValue, tailsUrl_char, &VcxWrapperCbResponseUnsignedInt);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}


- (void)vcxRevocationRegistryPublishRevocations:(NSNumber *)revRegHandle
                                     completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_revocation_registry_publish_revocations(handle, revRegHandle.unsignedIntValue, &VcxWrapperCbNoResponse);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)vcxRevocationRegistryGetRevRegId:(NSNumber *)revRegHandle
                              completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_revocation_registry_get_rev_reg_id(handle, revRegHandle.unsignedIntValue, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxRevocationRegistryGetTailsHash:(NSNumber *)revRegHandle
                               completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_revocation_registry_get_tails_hash(handle, revRegHandle.unsignedIntValue, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxRevocationRegistryDeserialize:(NSString *)serializedRevReg
                              completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    const char *serializedRevReg_char = [serializedRevReg cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_revocation_registry_deserialize(handle, serializedRevReg_char, &VcxWrapperCbResponseUnsignedInt);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}


- (void)vcxRevocationRegistrySerialize:(NSNumber *)revRegHandle
                            completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_revocation_registry_serialize(handle, revRegHandle.unsignedIntValue, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (int)vcxRevocationRegistryRelease:(NSNumber *)revRegHandle {
    return vcx_revocation_registry_release(revRegHandle.unsignedIntValue);
}

- (void)vcxCredentialDefinitionCreateV2:(NSString *)sourceId
                               schemaId:(NSString *)schemaId
                              issuerDid:(NSString *)issuerDid
                                    tag:(NSString *)tag
                      supportRevocation:(Boolean)supportRevocation
                             completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *sourceId_char = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *schemaId_char = [schemaId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *issuerDid_char = [issuerDid cStringUsingEncoding:NSUTF8StringEncoding];
    const char *tag_char = [tag cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_credentialdef_create_v2(
            handle,
            sourceId_char,
            schemaId_char,
            issuerDid_char,
            tag_char,
            supportRevocation,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}


- (void)vcxCredentialDefinitionPublish:(NSNumber *)credDefHandle
                              tailsUrl:(NSString *)tailsUrl
                            completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *tailsUrl_char = [tailsUrl cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_credentialdef_publish(
            handle,
            credDefHandle.unsignedIntValue,
            tailsUrl_char,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)vcxCredentialDefinitionDeserialize:(NSString *)serializedCredDef
                                completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *serializedCredDef_char = [serializedCredDef cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_credentialdef_deserialize(
            handle,
            serializedCredDef_char,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)vcxCredentialDefinitionSerialize:(NSNumber *)credDefHandle
                              completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_credentialdef_serialize(
            handle,
            credDefHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (int)vcxCredentialDefinitionRelease:(NSNumber *)credDefHandle {
    return vcx_credentialdef_release(credDefHandle.unsignedIntValue);
}

- (void)vcxCredentialDefinitionGetCredDefId:(NSNumber *)credDefHandle
                                 completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_credentialdef_get_cred_def_id(
            handle,
            credDefHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}


- (void)vcxCredentialDefinitionUpdateState:(NSNumber *)credDefHandle
                                completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_credentialdef_update_state(
            handle,
            credDefHandle.unsignedIntValue,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)vcxCredentialDefinitionGetState:(NSNumber *)credDefHandle
                             completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_credentialdef_get_state(
            handle,
            credDefHandle.unsignedIntValue,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)connectionCreate:(NSString *)sourceId
              completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *sourceId_char = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_connection_create(
            handle,
            sourceId_char,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)connectionCreateWithInvite:(NSString *)sourceId
                     inviteDetails:(NSString *)inviteDetails
                        completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *sourceId_char = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *inviteDetails_char = [inviteDetails cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_connection_create_with_invite(
            handle,
            sourceId_char,
            inviteDetails_char,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)connectionCreateWithConnectionRequestV2:(NSString *)sourceId
                                         pwInfo:(NSString *)pwInfo
                                        request:(NSString *)request
                                     completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *sourceId_char = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *pwInfo_char = [pwInfo cStringUsingEncoding:NSUTF8StringEncoding];
    const char *request_char = [request cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_connection_create_with_connection_request_v2(
            handle,
            sourceId_char,
            pwInfo_char,
            request_char,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}


- (void)connectionConnect:(NSNumber *)connectionHandle
           connectionType:(NSString *)connectionType
               completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *connectionType_char = [connectionType cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_connection_connect(
            handle,
            connectionHandle.unsignedIntValue,
            connectionType_char,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)connectionGetState:(NSNumber *)connectionHandle
                completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_connection_get_state(
            handle,
            connectionHandle.unsignedIntValue,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)connectionUpdateState:(NSNumber *)connectionHandle
                   completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_connection_update_state(
            handle,
            connectionHandle.unsignedIntValue,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)connectionUpdateStateWithMessage:(NSNumber *)connectionHandle
                                 message:(NSString *)message
                              completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *message_char = [message cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_connection_update_state_with_message(
            handle,
            connectionHandle.unsignedIntValue,
            message_char,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}


- (void)connectionHandleMessage:(NSNumber *)connectionHandle
                        message:(NSString *)message
                     completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *message_char = [message cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_connection_handle_message(
            handle,
            connectionHandle.unsignedIntValue,
            message_char,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)connectionSerialize:(NSNumber *)connectionHandle
                 completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_connection_serialize(
            handle,
            connectionHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)connectionDeserialize:(NSString *)serializedConnection
                   completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *serialized_connection = [serializedConnection cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_connection_deserialize(handle, serialized_connection, &VcxWrapperCbResponseUnsignedInt);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (int)connectionRelease:(NSNumber *)connectionHandle {
    return vcx_connection_release(connectionHandle);
}

- (void)connectionInviteDetails:(NSNumber *)connectionHandle
                     completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_connection_invite_details(
            handle,
            connectionHandle.unsignedIntValue,
            nil, //it has no effect
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}


- (void)deleteConnection:(VcxHandle)connectionHandle
          withCompletion:(void (^)(NSError *))completion {
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_connection_delete_connection(handle, connectionHandle, &VcxWrapperCbNoResponse);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)connectionGetPwDid:(NSNumber *)connectionHandle
                completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_connection_get_pw_did(
            handle,
            connectionHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)connectionGetTheirPwDid:(NSNumber *)connectionHandle
                     completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_connection_get_their_pw_did(
            handle,
            connectionHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)connectionInfo:(NSNumber *)connectionHandle
            completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_connection_info(
            handle,
            connectionHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)connectionGetThreadId:(NSNumber *)connectionHandle
                   completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_connection_get_thread_id(
            handle,
            connectionHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}


- (void)connectionSendMessage:(VcxHandle)connectionHandle
                  withMessage:(NSString *)message
       withSendMessageOptions:(NSString *)sendMessageOptions
               withCompletion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *message_ctype = [message cStringUsingEncoding:NSUTF8StringEncoding];
    const char *sendMessageOptions_ctype = [sendMessageOptions cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_connection_send_message(
            handle,
            connectionHandle,
            message_ctype,
            sendMessageOptions_ctype,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)connectionSignData:(VcxHandle)connectionHandle
                  withData:(NSData *)dataRaw
            withCompletion:(void (^)(NSError *, NSData *, vcx_u32_t))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    uint8_t *data_raw = (uint8_t * )
    [dataRaw bytes];
    uint32_t
            data_length = (uint32_t)
    [dataRaw length];

    vcx_error_t ret = vcx_connection_sign_data(
            handle,
            connectionHandle,
            data_raw,
            data_length,
            &VcxWrapperCbResponseData
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_DATA, ERROR_RESPONSE_NUMBER);
    });
}

- (void)connectionVerifySignature:(VcxHandle)connectionHandle
                         withData:(NSData *)dataRaw
                withSignatureData:(NSData *)signatureRaw
                   withCompletion:(void (^)(NSError *, vcx_bool_t))completion {
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    uint8_t *data_raw = (uint8_t * )
    [dataRaw bytes];
    uint32_t
            data_length = (uint32_t)
    [dataRaw length];

    uint8_t *signature_raw = (uint8_t * )
    [signatureRaw bytes];
    uint32_t
            signature_length = (uint32_t)
    [signatureRaw length];

    vcx_error_t ret = vcx_connection_verify_signature(handle,
            connectionHandle,
            data_raw,
            data_length,
            signature_raw,
            signature_length,
            &VcxWrapperCbResponseBool
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_BOOL);
    });
}

- (void)connectionSendPing:(NSNumber *)connectionHandle
                   comment:(NSString *)comment
                completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *comment_char = [comment cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_connection_send_ping(
            handle,
            connectionHandle.unsignedIntValue,
            comment_char,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)connectionSendDiscoveryFeatures:(NSNumber *)connectionHandle
                                  query:(NSString *)query
                                comment:(NSString *)comment
                             completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *query_char = [query cStringUsingEncoding:NSUTF8StringEncoding];
    const char *comment_char = [comment cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_connection_send_discovery_features(
            handle,
            connectionHandle.unsignedIntValue,
            query_char,
            comment_char,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}


- (void)connectionDownloadMessages:(VcxHandle)connectionHandle
                     messageStatus:(NSString *)messageStatus
                             uid_s:(NSString *)uid_s
                        completion:(void (^)(NSError *, NSString *))completion {
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *message_status = [messageStatus cStringUsingEncoding:NSUTF8StringEncoding];
    const char *uids = [uid_s cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_connection_messages_download(
            handle,
            connectionHandle,
            message_status,
            uids,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)connectionSendHandshakeReuse:(VcxHandle)connectionHandle
                              oobMsg:(NSString *)oobMsg
                          completion:(void (^)(NSError *))completion {
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *oobMsg_ctype = [oobMsg cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_connection_send_handshake_reuse(
            handle,
            connectionHandle,
            oobMsg_ctype,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)issuerCreateCredential:(NSString *)sourceId
                    completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *sourceId_char = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_issuer_create_credential(
            handle,
            sourceId_char,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)issuerRevokeCredentialLocal:(NSNumber *)credentialHandle
                         completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_issuer_revoke_credential_local(
            handle,
            credentialHandle.unsignedIntValue,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)issuerCredentialIsRevokable:(NSNumber *)credentialHandle
                         completion:(void (^)(NSError *, Boolean))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_issuer_credential_is_revokable(
            handle,
            credentialHandle.unsignedIntValue,
            &VcxWrapperCbResponseBool
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_BOOL);
    });
}

- (void)issuerSendCredentialOfferV2:(NSNumber *)credentialHandle
                   connectionHandle:(NSNumber *)connectionHandle
                         completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_issuer_send_credential_offer_v2(
            handle,
            credentialHandle.unsignedIntValue,
            connectionHandle.unsignedIntValue,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}


- (void)markCredentialOfferSent:(NSNumber *)credentialHandle
                     completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_mark_credential_offer_msg_sent(
            handle,
            credentialHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)issuerBuildCredentialOfferMessageV2:(NSNumber *)credDefHandle
                               revRegHandle:(NSNumber *)revRegHandle
                             credentialData:(NSString *)credData
                                    comment:(NSString *)comment
                                 completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *credData_char = [credData cStringUsingEncoding:NSUTF8StringEncoding];
    const char *comment_char = [comment cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_issuer_build_credential_offer_msg_v2(
            handle,
            credDefHandle.unsignedIntValue,
            revRegHandle.unsignedIntValue,
            credData_char,
            comment_char,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}


- (void)issuerGetCredentialOfferMessage:(NSNumber *)credentialHandle
                             completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_issuer_get_credential_offer_msg(
            handle,
            credentialHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)issuerGetCredentialMessage:(NSNumber *)credentialHandle
                     myPairwiseDid:(NSString *)myPwDid
                        completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *myPwDid_char = [myPwDid cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_issuer_get_credential_msg(
            handle,
            credentialHandle.unsignedIntValue,
            myPwDid_char,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}


- (void)issuerCredentialGetState:(NSNumber *)credentialHandle
                      completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_issuer_credential_get_state(
            handle,
            credentialHandle.unsignedIntValue,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)issuerCredentialGetRevRegId:(NSNumber *)credentialHandle
                         completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_issuer_credential_get_rev_reg_id(
            handle,
            credentialHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)issuerSendCredential:(NSNumber *)credentialHandle
            connectionHandle:(NSNumber *)connectionHandle
                  completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_issuer_send_credential(
            handle,
            credentialHandle.unsignedIntValue,
            connectionHandle.unsignedIntValue,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}


- (void)issuerCredentialSerialize:(NSNumber *)credentialHandle
                       completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_issuer_credential_serialize(
            handle,
            credentialHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)issuerCredentialDeserialize:(NSString *)serializedCredential
                         completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *serializedCredential_char = [serializedCredential cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_issuer_credential_deserialize(handle, serializedCredential_char, &VcxWrapperCbResponseUnsignedInt);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)issuerCredentialUpdateStateV2:(NSNumber *)credentialHandle
                     connectionHandle:(NSNumber *)connectionHandle
                           completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_v2_issuer_credential_update_state(
            handle,
            credentialHandle.unsignedIntValue,
            connectionHandle.unsignedIntValue,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)issuerCredentialUpdateStateWithMessageV2:(NSNumber *)credentialHandle
                                connectionHandle:(NSNumber *)connectionHandle
                                         message:(NSString *)message
                                      completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *message_char = [message cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_v2_issuer_credential_update_state_with_message(
            handle,
            credentialHandle.unsignedIntValue,
            connectionHandle.unsignedIntValue,
            message_char,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}


- (void)issuerCredentialGetThreadId:(NSNumber *)credentialHandle
                         completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_issuer_credential_get_thread_id(
            handle,
            credentialHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (int)issuerCredentialRelease:(NSNumber *)credentialHandle {
    return vcx_issuer_credential_release(credentialHandle.unsignedIntValue);
}

- (void)getCredential:(NSNumber *)credentialHandle
           completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_get_credential(
            handle,
            credentialHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)credentialCreateWithOffer:(NSString *)sourceId
                            offer:(NSString *)credentialOffer
                       completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *credential_offer = [credentialOffer cStringUsingEncoding:NSUTF8StringEncoding];
    const char *source_id = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_credential_create_with_offer(
            handle,
            source_id,
            credential_offer,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)credentialCreateWithMsgid:(NSString *)sourceId
                 connectionHandle:(NSNumber *)connectionHandle
                            msgId:(NSString *)msgId
                       completion:(void (^)(NSError *, NSNumber *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *source_id = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *msg_id = [msgId cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_credential_create_with_msgid(
            handle,
            source_id,
            connectionHandle.unsignedIntValue,
            msg_id,
            &VcxWrapperCbResponseUnsignedIntAndString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER, ERROR_RESPONSE_STRING);
    });
}

- (void)credentialSendRequest:(NSNumber *)credentialHandle
             connectionHandle:(NSNumber *)connectionHandle
                paymentHandle:(NSNumber *)paymentHandle
                   completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_credential_send_request(
            handle,
            credentialHandle.unsignedIntValue,
            connectionHandle.unsignedIntValue,
            paymentHandle.unsignedIntValue,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)credentialGetRequestMessage:(NSNumber *)credentialHandle
                      myPairwiseDid:(NSString *)myPwDid
                  theirdPairwiseDid:(NSString *)theirPwDid
                      paymentHandle:(NSNumber *)paymentHandle
                         completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *myPwDid_char = [myPwDid cStringUsingEncoding:NSUTF8StringEncoding];
    const char *theirPwDid_char = [theirPwDid cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_credential_get_request_msg(
            handle,
            credentialHandle.unsignedIntValue,
            myPwDid_char,
            theirPwDid_char,
            paymentHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)credentialDeclineOffer:(NSNumber *)credentialHandle
              connectionHandle:(NSNumber *)connectionHandle
                       comment:(NSString *)comment
                    completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *comment_char = [comment cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_credential_decline_offer(
            handle,
            credentialHandle.unsignedIntValue,
            connectionHandle.unsignedIntValue,
            comment_char,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)credentialGetState:(NSNumber *)credentialHandle
                completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_credential_get_state(
            handle,
            credentialHandle.unsignedIntValue,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)credentialUpdateStateV2:(NSNumber *)credentialHandle
               connectionHandle:(NSNumber *)connectionHandle
                     completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_v2_credential_update_state(
            handle,
            credentialHandle.unsignedIntValue,
            connectionHandle.unsignedIntValue,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)credentialUpdateStateWithMessageV2:(NSNumber *)credentialHandle
                          connectionHandle:(NSNumber *)connectionHandle
                                   message:(NSString *)message
                                completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *msg = [message cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_v2_credential_update_state_with_message(
            handle,
            credentialHandle.unsignedIntValue,
            connectionHandle.unsignedIntValue,
            msg,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)credentialGetOffers:(VcxHandle)connectionHandle
                 completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_credential_get_offers(handle, connectionHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)credentialGetAttributes:(NSNumber *)credentialHandle
                     completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_credential_get_attributes(
            handle,
            credentialHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)credentialGetAttachment:(NSNumber *)credentialHandle
                     completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_credential_get_attachment(
            handle,
            credentialHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)credentialGetTailsLocation:(NSNumber *)credentialHandle
                        completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_credential_get_tails_location(
            handle,
            credentialHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)credentialGetTailsHash:(NSNumber *)credentialHandle
                    completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_credential_get_tails_hash(
            handle,
            credentialHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)credentialGetRevRegId:(NSNumber *)credentialHandle
                   completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_credential_get_rev_reg_id(
            handle,
            credentialHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)credentialIsRevokable:(NSNumber *)credentialHandle
                   completion:(void (^)(NSError *, vcx_bool_t))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_credential_is_revokable(
            handle,
            credentialHandle.unsignedIntValue,
            &VcxWrapperCbResponseBool
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_BOOL);
    });
}

- (void)credentialSerialize:(NSNumber *)credentialHandle
                 completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_credential_serialize(
            handle,
            credentialHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)credentialDeserialize:(NSString *)serializedCredential
                   completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *serialized_credential = [serializedCredential cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_credential_deserialize(handle, serialized_credential, &VcxWrapperCbResponseUnsignedInt);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (int)credentialRelease:(NSNumber *)credentialHandle {
    return vcx_credential_release(credentialHandle.unsignedIntValue);
}


- (void)deleteCredential:(NSNumber *)credentialHandle
              completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_delete_credential(
            handle,
            credentialHandle.unsignedIntValue,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (int)walletSetHandle:(NSNumber *)handle {
    return vcx_wallet_set_handle(handle.integerValue);
}

- (void)exportWallet:(NSString *)exportPath
         encryptWith:(NSString *)encryptionKey
          completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *export_path = [exportPath cStringUsingEncoding:NSUTF8StringEncoding];
    const char *encryption_key = [encryptionKey cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_wallet_export(handle, export_path, encryption_key, &VcxWrapperCbNoResponse);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)importWallet:(NSString *)config
          completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_wallet_import(
            handle,
            [config cStringUsingEncoding:NSUTF8StringEncoding],
            VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)addRecordWallet:(NSString *)recordType
               recordId:(NSString *)recordId
            recordValue:(NSString *)recordValue
               tagsJson:(NSString *)tagsJson
             completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    if (!tagsJson.length) tagsJson = @"{}";

    const char *record_type = [recordType cStringUsingEncoding:NSUTF8StringEncoding];
    const char *record_id = [recordId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *record_value = [recordValue cStringUsingEncoding:NSUTF8StringEncoding];
    const char *tags_json = [tagsJson cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_wallet_add_record(
            handle,
            record_type,
            record_id,
            record_value,
            tags_json,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)getRecordWallet:(NSString *)recordType
               recordId:(NSString *)recordId
            optionsJson:(NSString *)optionsJson
             completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    if (!optionsJson.length) optionsJson = @"{}";
    const char *record_type = [recordType cStringUsingEncoding:NSUTF8StringEncoding];
    const char *record_id = [recordId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *options_json = [optionsJson cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_wallet_get_record(
            handle,
            record_type,
            record_id,
            options_json,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (int)vcxShutdown:(Boolean)deleteWallet {
    return vcx_shutdown(deleteWallet);
}

- (void)deleteRecordWallet:(NSString *)recordType
                  recordId:(NSString *)recordId
                completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *record_type = [recordType cStringUsingEncoding:NSUTF8StringEncoding];
    const char *record_id = [recordId cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_wallet_delete_record(
            handle,
            record_type,
            record_id, &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)updateRecordWallet:(NSString *)recordType
              withRecordId:(NSString *)recordId
           withRecordValue:(NSString *)recordValue
            withCompletion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *record_type = [recordType cStringUsingEncoding:NSUTF8StringEncoding];
    const char *record_id = [recordId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *record_value = [recordValue cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_wallet_update_record_value(
            handle,
            record_type,
            record_id,
            record_value, &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)addRecordTagsWallet:(NSString *)recordType
                   recordId:(NSString *)recordId
                   tagsJson:(NSString *)tagsJson
                 completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *record_type = [recordType cStringUsingEncoding:NSUTF8StringEncoding];
    const char *record_id = [recordId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *tags_json = [tagsJson cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_wallet_add_record_tags(
            handle,
            record_type,
            record_id,
            tags_json, &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)updateRecordTagsWallet:(NSString *)recordType
                      recordId:(NSString *)recordId
                      tagsJson:(NSString *)tagsJson
                    completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *record_type = [recordType cStringUsingEncoding:NSUTF8StringEncoding];
    const char *record_id = [recordId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *tags_json = [tagsJson cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_wallet_update_record_tags(
            handle,
            record_type,
            record_id,
            tags_json,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)deleteRecordTagsWallet:(NSString *)recordType
                      recordId:(NSString *)recordId
                  tagNamesJson:(NSString *)tagNamesJson
                    completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *record_type = [recordType cStringUsingEncoding:NSUTF8StringEncoding];
    const char *record_id = [recordId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *tag_names_json = [tagNamesJson cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_wallet_delete_record_tags(
            handle,
            record_type,
            record_id,
            tag_names_json,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)openSearchWallet:(NSString *)recordType
               queryJson:(NSString *)queryJson
             optionsJson:(NSString *)optionsJson
              completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    if (!queryJson.length) queryJson = @"{}";
    if (!optionsJson.length) optionsJson = @"{}";

    const char *record_type = [recordType cStringUsingEncoding:NSUTF8StringEncoding];
    const char *query_json = [queryJson cStringUsingEncoding:NSUTF8StringEncoding];
    const char *options_json = [optionsJson cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_wallet_open_search(
            handle,
            record_type,
            query_json,
            options_json,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)searchNextRecordsWallet:(NSNumber *)searchHandle
                          count:(int)count
                     completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_wallet_search_next_records(
            handle,
            searchHandle.integerValue,
            count,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)closeSearchWallet:(NSNumber *)searchHandle
               completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_wallet_close_search(
            handle,
            searchHandle.integerValue,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)verifierProofCreate:(NSString *)proofRequestId
             requestedAttrs:(NSString *)requestedAttrs
        requestedPredicates:(NSString *)requestedPredicates
         revocationInterval:(NSString *)revocationInterval
                  proofName:(NSString *)proofName
                 completion:(void (^)(NSError *, NSNumber *))completion; {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *proofRequestId_char = [proofRequestId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *requestedAttrs_char = [requestedAttrs cStringUsingEncoding:NSUTF8StringEncoding];
    const char *requestedPredicates_char = [requestedPredicates cStringUsingEncoding:NSUTF8StringEncoding];
    const char *revocationInterval_char = [revocationInterval cStringUsingEncoding:NSUTF8StringEncoding];
    const char *proofName_char = [proofName cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_proof_create(
            handle,
            proofRequestId_char,
            requestedAttrs_char,
            requestedPredicates_char,
            revocationInterval_char,
            proofName_char,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)verifierProofSendRequest:(NSNumber *)proofHandle
                connectionHandle:(NSNumber *)connectionHandle
                      completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_proof_send_request(
            handle,
            proofHandle.unsignedIntValue,
            connectionHandle.unsignedIntValue,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)verifierGetProofMessage:(NSNumber *)proofHandle
                     completion:(void (^)(NSError *, NSNumber *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_get_proof_msg(
            handle,
            proofHandle.unsignedIntValue,
            &VcxWrapperCbResponseUnsignedIntAndString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER, ERROR_RESPONSE_STRING);
    });
}


- (void)verifierProofGetRequestMessage:(NSNumber *)proofHandle
                            completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_proof_get_request_msg(
            handle,
            proofHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)verifierProofUpdateStateV2:(NSNumber *)proofHandle
                  connectionHandle:(NSNumber *)connectionHandle
                        completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_v2_proof_update_state(
            handle,
            proofHandle.unsignedIntValue,
            connectionHandle.unsignedIntValue,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)verifierProofUpdateStateWithMessageV2:(NSNumber *)proofHandle
                             connectionHandle:(NSNumber *)connectionHandle
                                      message:(NSString *)message
                                   completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *message_char = [message cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_v2_proof_update_state_with_message(
            handle,
            proofHandle.unsignedIntValue,
            connectionHandle.unsignedIntValue,
            message_char,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)verifierProofGetState:(NSNumber *)proofHandle
                   completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_proof_get_state(
            handle,
            proofHandle.unsignedIntValue,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)verifierProofGetThreadId:(NSNumber *)proofHandle
                      completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_proof_get_thread_id(
            handle,
            proofHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)verifierMarkPresentationRequestMessageSent:(NSNumber *)proofHandle
                                        completion:(void (^)(NSError *, NSNumber *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_mark_presentation_request_msg_sent(
            handle,
            proofHandle.unsignedIntValue,
            &VcxWrapperCbResponseUnsignedIntAndString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER, ERROR_RESPONSE_STRING);
    });
}


- (void)verifierProofSerialize:(NSNumber *)proofHandle
                    completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_proof_serialize(
            handle,
            proofHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)verifierProofDeserialize:(NSString *)serializedProof
                      completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *serializedProof_char = [serializedProof cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_proof_deserialize(handle, serializedProof_char, &VcxWrapperCbResponseUnsignedInt);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}


- (int)verifierProofRelease:(NSNumber *)proofHandle {
    return vcx_proof_release(proofHandle.unsignedIntValue);
}

- (void)proofGetRequests:(NSNumber *)connectionHandle
              completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_disclosed_proof_get_requests(
            handle,
            connectionHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)proofGetProofRequestAttachment:(NSNumber *)proofHandle
                            completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_disclosed_proof_get_proof_request_attachment(
            handle,
            proofHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)proofCreateWithMsgId:(NSString *)sourceId
        withConnectionHandle:(NSNumber *)connectionHandle
                   withMsgId:(NSString *)msgId
              withCompletion:(void (^)(NSError *, NSNumber *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *source_id = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *msg_id = [msgId cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_disclosed_proof_create_with_msgid(
            handle,
            source_id,
            connectionHandle.unsignedIntValue,
            msg_id,
            &VcxWrapperCbResponseUnsignedIntAndString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER, ERROR_RESPONSE_STRING);
    });
}

- (void)proofRetrieveCredentials:(NSNumber *)proofHandle
                  withCompletion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_disclosed_proof_retrieve_credentials(
            handle,
            proofHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)  proofGenerate:(NSNumber *)proofHandle
withSelectedCredentials:(NSString *)selectedCredentials
  withSelfAttestedAttrs:(NSString *)selfAttestedAttributes
         withCompletion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *selected_credentials = [selectedCredentials cStringUsingEncoding:NSUTF8StringEncoding];
    const char *self_attested_attributes = [selfAttestedAttributes cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_disclosed_proof_generate_proof(
            handle,
            proofHandle.unsignedIntValue,
            selected_credentials,
            self_attested_attributes,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)   proofSend:(NSNumber *)proof_handle
withConnectionHandle:(NSNumber *)connection_handle
      withCompletion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_disclosed_proof_send_proof(
            handle,
            proof_handle.unsignedIntValue,
            connection_handle.unsignedIntValue,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)proofGetState:(NSNumber *)proofHandle
           completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_disclosed_proof_get_state(
            handle,
            proofHandle.unsignedIntValue,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)proofUpdateStateV2:(NSNumber *)proofHandle
          connectionHandle:(NSNumber *)connectionHandle
                completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_v2_disclosed_proof_update_state(
            handle,
            proofHandle.unsignedIntValue,
            connectionHandle.unsignedIntValue,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)proofUpdateStateWithMessageV2:(NSNumber *)proofHandle
                     connectionHandle:(NSNumber *)connectionHandle
                              message:(NSString *)message
                           completion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *msg = [message cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_v2_disclosed_proof_update_state_with_message(
            handle,
            proofHandle.unsignedIntValue,
            connectionHandle.unsignedIntValue,
            msg,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void) proofReject:(NSNumber *)proof_handle
withConnectionHandle:(NSNumber *)connection_handle
      withCompletion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_disclosed_proof_reject_proof(
            handle,
            proof_handle.unsignedIntValue,
            connection_handle.unsignedIntValue,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)proofDeclinePresentationRequest:(NSNumber *)proof_handle
                       connectionHandle:(NSNumber *)connection_handle
                                 reason:(NSString *)reason
                               proposal:(NSString *)proposal
                             completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *reason_ctype = [reason cStringUsingEncoding:NSUTF8StringEncoding];
    const char *proposal_ctype = [proposal cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_disclosed_proof_decline_presentation_request(
            handle,
            proof_handle.unsignedIntValue,
            connection_handle.unsignedIntValue,
            reason_ctype,
            proposal_ctype,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)proofGetThreadId:(NSNumber *)proofHandle
          withCompletion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_disclosed_proof_get_thread_id(
            handle,
            proofHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)getProofMsg:(NSNumber *)proofHandle
     withCompletion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_disclosed_proof_get_proof_msg(
            handle,
            proofHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)getRejectMsg:(NSNumber *)proofHandle
      withCompletion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_disclosed_proof_get_reject_msg(
            handle,
            proofHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)proofCreateWithRequest:(NSString *)source_id
              withProofRequest:(NSString *)proofRequest
                withCompletion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *sourceId = [source_id cStringUsingEncoding:NSUTF8StringEncoding];
    const char *proof_request = [proofRequest cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_disclosed_proof_create_with_request(
            handle,
            sourceId,
            proof_request,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)proofSerialize:(NSNumber *)proofHandle
        withCompletion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_disclosed_proof_serialize(
            handle,
            proofHandle.unsignedIntValue,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)proofDeserialize:(NSString *)serializedProof
          withCompletion:(void (^)(NSError *, NSNumber *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *serialized_proof = [serializedProof cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_disclosed_proof_deserialize(handle, serialized_proof, &VcxWrapperCbResponseUnsignedInt);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (int)proofRelease:(NSNumber *)proofHandle {
    return vcx_disclosed_proof_release(proofHandle.unsignedIntValue);
}

- (void)downloadMessagesV2:(NSString *)connectionHandles
             messageStatus:(NSString *)messageStatus
                     uid_s:(NSString *)uid_s
                completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *message_status = [messageStatus cStringUsingEncoding:NSUTF8StringEncoding];
    const char *uids = [uid_s cStringUsingEncoding:NSUTF8StringEncoding];
    const char *connection_handles = [connectionHandles cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_v2_messages_download(
            handle,
            connection_handles,
            message_status,
            uids,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)updateMessages:(NSString *)messageStatus
            pwdidsJson:(NSString *)pwDidsJson
            completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *message_status = [messageStatus cStringUsingEncoding:NSUTF8StringEncoding];
    const char *msg_json = [pwDidsJson cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_messages_update_status(handle, message_status, msg_json, &VcxWrapperCbNoResponse);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

/// Retrieve author agreement set on the Ledger
///
/// #params
/// completion: Callback that provides array of matching messages retrieved
///
/// #Returns
/// Error code as a u32
- (void)getTxnAuthorAgreement:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_get_ledger_author_agreement(handle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
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
- (vcx_error_t)activateTxnAuthorAgreement:(NSString *)text
                              withVersion:(NSString *)version
                                 withHash:(NSString *)hash
                            withMechanism:(NSString *)mechanism
                            withTimestamp:(long)timestamp {
    return vcx_set_active_txn_author_agreement_meta(
            [text UTF8String],
            [version UTF8String],
            [hash UTF8String],
            [mechanism UTF8String],
            timestamp
    );
}

@end
