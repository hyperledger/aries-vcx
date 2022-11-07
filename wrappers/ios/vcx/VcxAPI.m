//
//  init.m
//  vcx
//
//  Created by GuestUser on 4/30/18.
//  Copyright © 2018 GuestUser. All rights reserved.
//

#import <Foundation/Foundation.h>
#import "VcxAPI.h"
#import "NSError+VcxError.h"
#import "VcxCallbacks.h"
#import "VcxWrapperCallbacks.h"
#import "libvcx.h"
#import "IndySdk.h"

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
                 completion:(void (^)(NSError *error))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_init_issuer_config(handle, config_char, &VcxWrapperCbNoResponse);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (vcx_error_t)vcxPoolSetHandle:(NSInteger)handle
                     completion:(void (^)(NSError *))completion {
    return vcx_pool_set_handle(handle);
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
                  seqNo:(NSInteger)seqNo
             completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *submitterDid_char = [submitterDid cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_get_ledger_txn(handle, submitterDid_char, seqNo, &VcxWrapperCbResponseString);

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
            completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_open_main_wallet(handle, config_char, &VcxWrapperCbResponseSignedHandle);

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

- (NSString *)errorCMessage:(NSInteger)errorCode {
    const char *ret = vcx_error_c_message((int) errorCode);

    NSString *message = nil;

    if (ret) {
        message = [NSString stringWithUTF8String:ret];
    }

    return message;
}

- (NSString *)vcxVersion {
    return [NSString stringWithUTF8String:vcx_version()];
}

- (void)vcxSchemaSerialize:(NSInteger)schemaHandle
                completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_schema_serialize(handle, schemaHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxSchemaDeserialize:(NSString *)serializedSchema
                  completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *serializedSchema_char = [serializedSchema cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_schema_deserialize(handle, serializedSchema_char, &VcxWrapperCbResponseHandle);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)vcxSchemaGetAttributes:(NSString *)sourceId
                    sequenceNo:(NSInteger)sequenceNo
                    completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *sourceId_char = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_schema_get_attributes(handle, sourceId_char, sequenceNo, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxSchemaCreate:(NSString *)sourceId
             schemaName:(NSString *)schemaName
          schemaVersion:(NSString *)schemaVersion
             schemaData:(NSString *)schemaData
          paymentHandle:(NSInteger)paymentHandle
             completion:(void (^)(NSError *, NSInteger))completion {

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
            paymentHandle,
            VcxWrapperCbResponseHandle
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
                         completion:(void (^)(NSError *, NSInteger, NSString *))completion {

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
            &VcxWrapperCbResponseHandleAndString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER, ERROR_RESPONSE_STRING);
    });
}


- (void)vcxSchemaGetSchemaId:(NSString *)sourceId
                schemaHandle:(NSInteger)schemaHandle
                  completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_schema_get_schema_id(handle, schemaHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxSchemaUpdateState:(NSString *)sourceId
                schemaHandle:(NSInteger)schemaHandle
                  completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_schema_update_state(handle, schemaHandle, &VcxWrapperCbResponseUnsignedInt);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (int)vcxSchemaRelease:(NSInteger)schemaHandle {
    return vcx_schema_release(schemaHandle);
}

- (void)vcxPublicAgentCreate:(NSString *)sourceId
              institutionDid:(NSString *)institutionDid
                  completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *sourceId_char = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *institutionDid_char = [institutionDid cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_public_agent_create(handle, sourceId_char, institutionDid_char, &VcxWrapperCbResponseHandle);

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

- (void)vcxPublicAgentDownloadConnectionRequests:(NSInteger)agentHandle
                                            uids:(NSString *)ids
                                      completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *ids_char = [ids cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_public_agent_download_connection_requests(
            handle,
            agentHandle,
            ids_char,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxPublicAgentDownloadMessage:(NSInteger)agentHandle
                                  uid:(NSString *)id
                           completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *id_char = [id cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_public_agent_download_message(
            handle,
            agentHandle,
            id_char,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxPublicAgentGetService:(NSInteger)agentHandle
                      completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_public_agent_get_service(handle, agentHandle, &VcxWrapperCbResponseString);
    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxPublicAgentSerialize:(NSInteger)agentHandle
                     completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_public_agent_serialize(handle, agentHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}


- (int)vcxPublicAgentRelease:(NSInteger)agentHandle {
    return vcx_public_agent_release(agentHandle);
}

- (void)vcxOutOfBandSenderCreate:(NSString *)config
                      completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_out_of_band_sender_create(handle, config_char, &VcxWrapperCbResponseHandle);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)vcxOutOfBandReceiverCreate:(NSString *)message
                        completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *message_char = [message cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_out_of_band_receiver_create(handle, message_char, &VcxWrapperCbResponseHandle);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)vcxOutOfBandSenderAppendMessage:(NSInteger)oobHandle
                                message:(NSString *)message
                             completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *message_char = [message cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_out_of_band_sender_append_message(
            handle,
            oobHandle,
            message_char,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)vcxOutOfBandSenderAppendService:(NSInteger)oobHandle
                                service:(NSString *)service
                             completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *service_char = [service cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_out_of_band_sender_append_service(
            handle,
            oobHandle,
            service_char,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}


- (void)vcxOutOfBandSenderAppendServiceDid:(NSInteger)oobHandle
                                       did:(NSString *)did
                                completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *did_char = [did cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_out_of_band_sender_append_service_did(
            handle,
            oobHandle,
            did_char,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)vcxOutOfBandSenderGetThreadId:(NSInteger)oobHandle
                           completion:(void (^)(NSError *, NSString *))completion {
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_out_of_band_sender_get_thread_id(
            handle,
            oobHandle,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxOutOfBandReceiverGetThreadId:(NSInteger)oobHandle
                             completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_out_of_band_receiver_get_thread_id(
            handle,
            oobHandle,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxOutOfBandReceiverExtractMessage:(NSInteger)oobHandle
                                completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_out_of_band_receiver_extract_message(
            handle,
            oobHandle,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}


- (void)vcxOutOfBandToMessage:(NSInteger)oobHandle
                   completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_out_of_band_to_message(
            handle,
            oobHandle,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxOutOfBandSenderSerialize:(NSInteger)oobHandle
                         completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_out_of_band_sender_serialize(
            handle,
            oobHandle,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxOutOfBandReceiverSerialize:(NSInteger)oobHandle
                           completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_out_of_band_receiver_serialize(
            handle,
            oobHandle,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxOutOfBandSenderDeserialize:(NSString *)oobMessage
                           completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *oobMessage_char = [oobMessage cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_out_of_band_sender_deserialize(
            handle,
            oobMessage_char,
            &VcxWrapperCbResponseHandle
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)vcxOutOfBandReceiverDeserialize:(NSString *)oobMessage
                             completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *oobMessage_char = [oobMessage cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_out_of_band_receiver_deserialize(
            handle,
            oobMessage_char,
            &VcxWrapperCbResponseHandle
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}


- (int)vcxOutOfBandSenderRelease:(NSInteger)agentHandle {
    return vcx_out_of_band_sender_release(agentHandle);
}

- (int)vcxOutOfBandReceiverRelease:(NSInteger)agentHandle {
    return vcx_out_of_band_receiver_release(agentHandle);
}

- (void)vcxOutOfBandReceiverConnectionExists:(NSInteger)oobHandle
                           connectionHandles:(NSString *)connectionHandles
                                  completion:(void (^)(NSError *, NSInteger, Boolean))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *connectionHandles_char = [connectionHandles cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_out_of_band_receiver_connection_exists(
            handle,
            oobHandle,
            connectionHandles_char,
            &VcxWrapperCbResponseHandleAndBool
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER, ERROR_RESPONSE_BOOL);
    });
}


- (void)vcxOutOfBandReceiverBuildConnection:(NSInteger)oobHandle
                                 completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_out_of_band_receiver_build_connection(
            handle,
            oobHandle,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxRevocationRegistryCreate:(NSString *)revRegConfig
                         completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *revRegConfig_char = [revRegConfig cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_revocation_registry_create(handle, revRegConfig_char, &VcxWrapperCbResponseHandle);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)vcxRevocationRegistryPublish:(NSInteger)revRegHandle
                            tailsUrl:(NSString *)tailsUrl
                          completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *tailsUrl_char = [tailsUrl cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_revocation_registry_publish(handle, revRegHandle, tailsUrl_char, &VcxWrapperCbResponseHandle);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}


- (void)vcxRevocationRegistryPublishRevocations:(NSInteger)revRegHandle
                                     completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_revocation_registry_publish_revocations(handle, revRegHandle, &VcxWrapperCbNoResponse);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)vcxRevocationRegistryGetRevRegId:(NSInteger)revRegHandle
                              completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_revocation_registry_get_rev_reg_id(handle, revRegHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxRevocationRegistryGetTailsHash:(NSInteger)revRegHandle
                               completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_revocation_registry_get_tails_hash(handle, revRegHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)vcxRevocationRegistryDeserialize:(NSString *)serializedRevReg
                              completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    const char *serializedRevReg_char = [serializedRevReg cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_revocation_registry_deserialize(handle, serializedRevReg_char, &VcxWrapperCbResponseHandle);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}


- (void)vcxRevocationRegistrySerialize:(NSInteger)revRegHandle
                            completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_revocation_registry_serialize(handle, revRegHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (int)vcxRevocationRegistryRelease:(NSInteger)revRegHandle {
    return vcx_revocation_registry_release(revRegHandle);
}

- (void)vcxCredentialDefinitionCreateV2:(NSString *)sourceId
                               schemaId:(NSString *)schemaId
                              issuerDid:(NSString *)issuerDid
                                    tag:(NSString *)tag
                      supportRevocation:(Boolean)supportRevocation
                             completion:(void (^)(NSError *, NSInteger))completion {

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
            &VcxWrapperCbResponseHandle
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}


- (void)vcxCredentialDefinitionPublish:(NSInteger)credDefHandle
                              tailsUrl:(NSString *)tailsUrl
                            completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *tailsUrl_char = [tailsUrl cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_credentialdef_publish(
            handle,
            credDefHandle,
            tailsUrl_char,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)vcxCredentialDefinitionDeserialize:(NSString *)serializedCredDef
                                completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *serializedCredDef_char = [serializedCredDef cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_credentialdef_deserialize(
            handle,
            serializedCredDef_char,
            &VcxWrapperCbResponseHandle
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)vcxCredentialDefinitionSerialize:(NSInteger)credDefHandle
                              completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_credentialdef_serialize(
            handle,
            credDefHandle,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (int)vcxCredentialDefinitionRelease:(NSInteger)credDefHandle {
    return vcx_credentialdef_release(credDefHandle);
}

- (void)vcxCredentialDefinitionGetCredDefId:(NSInteger)credDefHandle
                                 completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_credentialdef_get_cred_def_id(
            handle,
            credDefHandle,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}


- (void)vcxCredentialDefinitionUpdateState:(NSInteger)credDefHandle
                                completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_credentialdef_update_state(
            handle,
            credDefHandle,
            &VcxWrapperCbResponseHandle
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)vcxCredentialDefinitionGetState:(NSInteger)credDefHandle
                             completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_credentialdef_get_state(
            handle,
            credDefHandle,
            &VcxWrapperCbResponseHandle
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)connectionCreate:(NSString *)sourceId
              completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *sourceId_char = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_connection_create(
            handle,
            sourceId_char,
            &VcxWrapperCbResponseHandle
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)connectionCreateWithInvite:(NSString *)invitationId
                     inviteDetails:(NSString *)inviteDetails
                        completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *invitationId_char = [invitationId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *inviteDetails_char = [inviteDetails cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_connection_create_with_invite(
            handle,
            invitationId_char,
            inviteDetails_char,
            &VcxWrapperCbResponseHandle
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)connectionCreateWithConnectionRequest:(NSString *)sourceId
                                  agentHandle:(NSInteger)agentHandle
                                      request:(NSString *)request
                                   completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *sourceId_char = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *request_char = [request cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_connection_create_with_connection_request(
            handle,
            sourceId_char,
            agentHandle,
            request_char,
            &VcxWrapperCbResponseHandle
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}


- (void)connectionConnect:(NSInteger)connectionHandle
           connectionType:(NSString *)connectionType
               completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *connectionType_char = [connectionType cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_connection_connect(
            handle,
            connectionHandle,
            connectionType_char,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)connectionGetState:(NSInteger)connectionHandle
                completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_connection_get_state(handle, connectionHandle, &VcxWrapperCbResponseUnsignedInt);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)connectionUpdateState:(NSInteger)connectionHandle
                   completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_connection_update_state(handle, connectionHandle, &VcxWrapperCbResponseUnsignedInt);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)connectionUpdateStateWithMessage:(NSInteger)connectionHandle
                                 message:(NSString *)message
                              completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *message_char = [message cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_connection_update_state_with_message(
            handle,
            connectionHandle,
            message_char,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}


- (void)connectionHandleMessage:(NSInteger)connectionHandle
                        message:(NSString *)message completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *message_char = [message cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_connection_handle_message(
            handle,
            connectionHandle,
            message_char,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)connectionSerialize:(NSInteger)connectionHandle
                 completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_connection_serialize(handle, connectionHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)connectionDeserialize:(NSString *)serializedConnection
                   completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *serialized_connection = [serializedConnection cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_connection_deserialize(handle, serialized_connection, &VcxWrapperCbResponseHandle);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (int)connectionRelease:(NSInteger)connectionHandle {
    return vcx_connection_release(connectionHandle);
}

- (void)connectionInviteDetails:(NSInteger)connectionHandle
                     completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_connection_invite_details(
            handle,
            connectionHandle,
            nil, //it has no effect
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}


- (void)deleteConnection:(VcxHandle)connectionHandle
          withCompletion:(void (^)(NSError *error))completion {
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_connection_delete_connection(handle, connectionHandle, &VcxWrapperCbNoResponse);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)connectionGetPwDid:(NSInteger)connectionHandle
                completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_connection_get_pw_did(handle, connectionHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)connectionGetTheirPwDid:(NSInteger)connectionHandle
                     completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_connection_get_their_pw_did(handle, connectionHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)connectionInfo:(NSInteger)connectionHandle
            completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_connection_info(
            handle,
            connectionHandle,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)connectionGetThreadId:(NSInteger)connectionHandle
                   completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_connection_get_thread_id(
            handle,
            connectionHandle,
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

    uint8_t *data_raw = (uint8_t *) [dataRaw bytes];
    uint32_t data_length = (uint32_t) [dataRaw length];

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

    uint8_t *data_raw = (uint8_t *) [dataRaw bytes];
    uint32_t data_length = (uint32_t) [dataRaw length];

    uint8_t *signature_raw = (uint8_t *) [signatureRaw bytes];
    uint32_t signature_length = (uint32_t) [signatureRaw length];

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

- (void)connectionSendPing:(NSInteger)connectionHandle
                   comment:(NSString *)comment completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *comment_char = [comment cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_connection_send_ping(
            handle,
            connectionHandle,
            comment_char,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)connectionSendDiscoveryFeatures:(NSInteger)connectionHandle
                                  query:(NSString *)query
                                comment:(NSString *)comment
                             completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *query_char = [query cStringUsingEncoding:NSUTF8StringEncoding];
    const char *comment_char = [comment cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_connection_send_discovery_features(
            handle,
            connectionHandle,
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
                    completion:(void (^)(NSError *, NSInteger))completion {

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

- (void)issuerRevokeCredentialLocal:(NSInteger)credentialHandle
                         completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_issuer_revoke_credential_local(
            handle,
            credentialHandle,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)issuerCredentialIsRevokable:(NSInteger)credentialHandle
                         completion:(void (^)(NSError *, Boolean))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_issuer_credential_is_revokable(
            handle,
            credentialHandle,
            &VcxWrapperCbResponseBool
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_BOOL);
    });
}

- (void)issuerSendCredentialOfferV2:(NSInteger)credentialHandle
                   connectionHandle:(NSInteger)connectionHandle
                         completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_issuer_send_credential_offer_v2(
            handle,
            credentialHandle,
            connectionHandle,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}


- (void)markCredentialOfferSent:(NSInteger)credentialHandle
                     completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_mark_credential_offer_msg_sent(handle, credentialHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)issuerBuildCredentialOfferMessageV2:(NSInteger)credDefHandle
                               revRegHandle:(NSInteger)revRegHandle
                             credentialData:(NSString *)credData
                                    comment:(NSString *)comment
                                 completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *credData_char = [credData cStringUsingEncoding:NSUTF8StringEncoding];
    const char *comment_char = [comment cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_issuer_build_credential_offer_msg_v2(
            handle,
            credDefHandle,
            revRegHandle,
            credData_char,
            comment_char,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}


- (void)issuerGetCredentialOfferMessage:(NSInteger)credentialHandle
                             completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_issuer_get_credential_offer_msg(
            handle,
            credentialHandle,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)issuerGetCredentialMessage:(NSInteger)credentialHandle
                     myPairwiseDid:(NSString *)myPwDid
                        completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *myPwDid_char = [myPwDid cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_issuer_get_credential_msg(
            handle,
            credentialHandle,
            myPwDid_char,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}


- (void)issuerCredentialGetState:(NSInteger)credentialHandle
                      completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_issuer_credential_get_state(
            handle,
            credentialHandle,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)issuerCredentialGetRevRegId:(NSInteger)credentialHandle
                         completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_issuer_credential_get_rev_reg_id(handle, credentialHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)issuerSendCredential:(NSInteger)credentialHandle
            connectionHandle:(NSInteger)connectionHandle
                  completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_issuer_send_credential(
            handle,
            credentialHandle,
            connectionHandle,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}


- (void)issuerCredentialSerialize:(NSInteger)credentialHandle
                       completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_issuer_credential_serialize(handle, credentialHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)issuerCredentialDeserialize:(NSString *)serializedCredential
                         completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *serializedCredential_char = [serializedCredential cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_issuer_credential_deserialize(handle, serializedCredential_char, &VcxWrapperCbResponseHandle);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)issuerCredentialUpdateStateV2:(NSInteger)credentialHandle
                     connectionHandle:(NSInteger)connectionHandle
                           completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_v2_issuer_credential_update_state(
            handle,
            credentialHandle,
            connectionHandle,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)issuerCredentialUpdateStateWithMessageV2:(NSInteger)credentialHandle
                                connectionHandle:(NSInteger)connectionHandle
                                         message:(NSString *)message
                                      completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *message_char = [message cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_v2_issuer_credential_update_state_with_message(
            handle,
            credentialHandle,
            connectionHandle,
            message_char,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}


- (void)issuerCredentialGetThreadId:(NSInteger)credentialHandle
                         completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_issuer_credential_get_thread_id(handle, credentialHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (int)issuerCredentialRelease:(NSInteger)credentialHandle {
    return vcx_issuer_credential_release(credentialHandle);
}

- (void)getCredential:(NSInteger)credentialHandle
           completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_get_credential(handle, credentialHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)credentialCreateWithOffer:(NSString *)sourceId
                            offer:(NSString *)credentialOffer
                       completion:(void (^)(NSError *, NSInteger))completion {

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
                 connectionHandle:(VcxHandle)connectionHandle
                            msgId:(NSString *)msgId
                       completion:(void (^)(NSError *, NSInteger, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *source_id = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *msg_id = [msgId cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_credential_create_with_msgid(
            handle,
            source_id,
            connectionHandle,
            msg_id,
            &VcxWrapperCbResponseHandleAndString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER, ERROR_RESPONSE_STRING);
    });
}

- (void)credentialSendRequest:(NSInteger)credentialHandle
             connectionHandle:(VcxHandle)connectionHandle
                paymentHandle:(NSInteger)paymentHandle
                   completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_credential_send_request(
            handle,
            credentialHandle,
            connectionHandle,
            paymentHandle,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)credentialGetRequestMessage:(NSInteger)credentialHandle
                      myPairwiseDid:(NSString *)myPwDid
                  theirdPairwiseDid:(NSString *)theirPwDid
                      paymentHandle:(NSInteger)paymentHandle
                         completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *myPwDid_char = [myPwDid cStringUsingEncoding:NSUTF8StringEncoding];
    const char *theirPwDid_char = [theirPwDid cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_credential_get_request_msg(
            handle,
            credentialHandle,
            myPwDid_char,
            theirPwDid_char,
            paymentHandle,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)credentialDeclineOffer:(NSInteger)credentialHandle
              connectionHandle:(NSInteger)connectionHandle
                       comment:(NSString *)comment
                    completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *comment_char = [comment cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_credential_decline_offer(
            handle,
            credentialHandle,
            connectionHandle,
            comment_char,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)credentialGetState:(NSInteger)credentialHandle
                completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_credential_get_state(handle, credentialHandle, &VcxWrapperCbResponseUnsignedInt);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)credentialUpdateStateV2:(NSInteger)credentialHandle
               connectionHandle:(vcx_connection_handle_t)connectionHandle
                     completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_v2_credential_update_state(
            handle,
            credentialHandle,
            connectionHandle,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)credentialUpdateStateWithMessageV2:(NSInteger)credentialHandle
                          connectionHandle:(vcx_connection_handle_t)connectionHandle
                                   message:(NSString *)message
                                completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *msg = [message cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_v2_credential_update_state_with_message(
            handle,
            credentialHandle,
            connectionHandle,
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

- (void)credentialGetAttributes:(NSInteger)credentialHandle
                     completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_credential_get_attributes(handle, credentialHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)credentialGetAttachment:(NSInteger)credentialHandle
                     completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_credential_get_attachment(handle, credentialHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)credentialGetTailsLocation:(NSInteger)credentialHandle
                        completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_credential_get_tails_location(handle, credentialHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)credentialGetTailsHash:(NSInteger)credentialHandle
                    completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_credential_get_tails_hash(handle, credentialHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)credentialGetRevRegId:(NSInteger)credentialHandle
                   completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_credential_get_rev_reg_id(handle, credentialHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)credentialIsRevokable:(NSInteger)credentialHandle
                   completion:(void (^)(NSError *, vcx_bool_t))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_credential_is_revokable(handle, credentialHandle, &VcxWrapperCbResponseBool);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_BOOL);
    });
}

- (void)credentialSerialize:(NSInteger)credentialHandle
                 completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_credential_serialize(handle, credentialHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)credentialDeserialize:(NSString *)serializedCredential
                   completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *serialized_credential = [serializedCredential cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_credential_deserialize(handle, serialized_credential, &VcxWrapperCbResponseUnsignedInt);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (int)credentialRelease:(NSInteger)credentialHandle {
    return vcx_credential_release(credentialHandle);
}


- (void)deleteCredential:(NSInteger)credentialHandle
              completion:(void (^)(NSError *error))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_delete_credential(handle, credentialHandle, &VcxWrapperCbNoResponse);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (int)walletSetHandle:(NSInteger)handle {
    return vcx_wallet_set_handle((vcx_i32_t)handle);
}

- (void)exportWallet:(NSString *)exportPath
         encryptWith:(NSString *)encryptionKey
          completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *export_path = [exportPath cStringUsingEncoding:NSUTF8StringEncoding];
    const char *encryption_key = [encryptionKey cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_wallet_export(handle, export_path, encryption_key, &VcxWrapperCbNoResponse);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)importWallet:(NSString *)config
          completion:(void (^)(NSError *error))completion {

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
              completion:(void (^)(NSError *, NSInteger))completion {

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
            &VcxWrapperCbResponseHandle
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)searchNextRecordsWallet:(NSInteger)searchHandle
                          count:(int)count
                     completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_wallet_search_next_records(
            handle,
            searchHandle,
            count,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)closeSearchWallet:(NSInteger)searchHandle
               completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_wallet_close_search(handle, searchHandle, &VcxWrapperCbNoResponse);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)verifierProofCreate:(NSString *)proofRequestId
             requestedAttrs:(NSString *)requestedAttrs
        requestedPredicates:(NSString *)requestedPredicates
         revocationInterval:(NSString *)revocationInterval
                  proofName:(NSString *)proofName
                 completion:(void (^)(NSError *, NSInteger))completion; {

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
            &VcxWrapperCbResponseHandle
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)verifierProofSendRequest:(NSInteger)proofHandle
                connectionHandle:(NSInteger)connectionHandle
                      completion:(void (^)(NSError *error))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_proof_send_request(handle, proofHandle, connectionHandle, &VcxWrapperCbNoResponse);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)verifierGetProofMessage:(NSInteger)proofHandle
                     completion:(void (^)(NSError *error, NSInteger state, NSString *responseData))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_get_proof_msg(handle, proofHandle, &VcxWrapperCbResponseHandleAndString);//TODO: find way to unify number response cb

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER, ERROR_RESPONSE_STRING);
    });
}


- (void)verifierProofGetRequestMessage:(NSInteger)proofHandle completion:(void (^)(NSError *error, NSString *message))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_proof_get_request_msg(handle, proofHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)verifierProofUpdateStateV2:(NSInteger)proofHandle
                  connectionHandle:(NSInteger)connectionHandle
                        completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_v2_proof_update_state(
            handle,
            proofHandle,
            connectionHandle,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)verifierProofUpdateStateWithMessageV2:(NSInteger)proofHandle
                             connectionHandle:(NSInteger)connectionHandle
                                      message:(NSString *)message
                                   completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *message_char = [message cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_v2_proof_update_state_with_message(
            handle,
            proofHandle,
            connectionHandle,
            message_char,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)verifierProofGetState:(NSInteger)proofHandle
                   completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_proof_get_state(handle, proofHandle, &VcxWrapperCbResponseUnsignedInt);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)verifierProofGetThreadId:(NSInteger)proofHandle
                      completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_proof_get_thread_id(handle, proofHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)verifierMarkPresentationRequestMessageSent:(NSInteger)proofHandle
                                        completion:(void (^)(NSError *, NSInteger, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_mark_presentation_request_msg_sent(handle, proofHandle, &VcxWrapperCbResponseHandleAndString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER, ERROR_RESPONSE_STRING);
    });
}


- (void)verifierProofSerialize:(NSInteger)proofHandle completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_proof_serialize(handle, proofHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)verifierProofDeserialize:(NSString *)serializedProof
                      completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *serializedProof_char = [serializedProof cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_proof_deserialize(handle, serializedProof_char, &VcxWrapperCbResponseHandle);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}


- (int)verifierProofRelease:(NSInteger)proofHandle {
    return vcx_proof_release(proofHandle);
}

- (void)proofGetRequests:(NSInteger)connectionHandle
              completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_disclosed_proof_get_requests(handle, connectionHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)proofGetProofRequestAttachment:(NSInteger)proofHandle
                            completion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_disclosed_proof_get_proof_request_attachment(
            handle,
            proofHandle,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)proofCreateWithMsgId:(NSString *)sourceId
        withConnectionHandle:(NSInteger)connectionHandle
                   withMsgId:(NSString *)msgId
              withCompletion:(void (^)(NSError *, NSInteger, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *source_id = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *msg_id = [msgId cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_disclosed_proof_create_with_msgid(
            handle,
            source_id,
            connectionHandle,
            msg_id,
            &VcxWrapperCbResponseHandleAndString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER, ERROR_RESPONSE_STRING);
    });
}

- (void)proofRetrieveCredentials:(NSInteger)proofHandle
                  withCompletion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_disclosed_proof_retrieve_credentials(handle, proofHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)  proofGenerate:(NSInteger)proofHandle
withSelectedCredentials:(NSString *)selectedCredentials
  withSelfAttestedAttrs:(NSString *)selfAttestedAttributes
         withCompletion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *selected_credentials = [selectedCredentials cStringUsingEncoding:NSUTF8StringEncoding];
    const char *self_attested_attributes = [selfAttestedAttributes cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_disclosed_proof_generate_proof(
            handle,
            proofHandle,
            selected_credentials,
            self_attested_attributes,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)   proofSend:(NSInteger)proof_handle
withConnectionHandle:(NSInteger)connection_handle
      withCompletion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_disclosed_proof_send_proof(
            handle,
            proof_handle,
            connection_handle,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)proofGetState:(NSInteger)proofHandle
           completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_disclosed_proof_get_state(
            handle,
            proofHandle,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)proofUpdateStateV2:(NSInteger)proofHandle
          connectionHandle:(NSInteger)connectionHandle
                completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_v2_disclosed_proof_update_state(
            handle,
            proofHandle,
            connectionHandle,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void)proofUpdateStateWithMessageV2:(NSInteger)proofHandle
                     connectionHandle:(NSInteger)connectionHandle
                              message:(NSString *)message
                           completion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *msg = [message cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_v2_disclosed_proof_update_state_with_message(
            handle,
            proofHandle,
            connectionHandle,
            msg,
            &VcxWrapperCbResponseUnsignedInt
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (void) proofReject:(NSInteger)proof_handle
withConnectionHandle:(NSInteger)connection_handle
      withCompletion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_disclosed_proof_reject_proof(
            handle,
            proof_handle,
            connection_handle,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)proofDeclinePresentationRequest:(NSInteger)proof_handle
                       connectionHandle:(NSInteger)connection_handle
                                 reason:(NSString *)reason
                               proposal:(NSString *)proposal
                             completion:(void (^)(NSError *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *reason_ctype = [reason cStringUsingEncoding:NSUTF8StringEncoding];
    const char *proposal_ctype = [proposal cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_disclosed_proof_decline_presentation_request(
            handle,
            proof_handle,
            connection_handle,
            reason_ctype,
            proposal_ctype,
            &VcxWrapperCbNoResponse
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret]);
    });
}

- (void)proofGetThreadId:(NSInteger)proofHandle
          withCompletion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_disclosed_proof_get_thread_id(handle, proofHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)getProofMsg:(NSInteger)proofHandle
     withCompletion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_disclosed_proof_get_proof_msg(handle, proofHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)getRejectMsg:(NSInteger)proofHandle
      withCompletion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_disclosed_proof_get_reject_msg(handle, proofHandle, &VcxWrapperCbResponseString);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)proofCreateWithRequest:(NSString *)source_id
              withProofRequest:(NSString *)proofRequest
                withCompletion:(void (^)(NSError *, NSInteger))completion {

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

- (void)proofSerialize:(NSInteger)proofHandle
        withCompletion:(void (^)(NSError *, NSString *))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    vcx_error_t ret = vcx_disclosed_proof_serialize(
            handle,
            proofHandle,
            &VcxWrapperCbResponseString
    );

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_STRING);
    });
}

- (void)proofDeserialize:(NSString *)serializedProof
          withCompletion:(void (^)(NSError *, NSInteger))completion {

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *serialized_proof = [serializedProof cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_disclosed_proof_deserialize(handle, serialized_proof, &VcxWrapperCbResponseUnsignedInt);

    checkErrorAndComplete(ret, handle, ^{
        completion([NSError errorFromVcxError:ret], ERROR_RESPONSE_NUMBER);
    });
}

- (int)proofRelease:(NSInteger)proofHandle {
    return vcx_disclosed_proof_release(proofHandle);
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
