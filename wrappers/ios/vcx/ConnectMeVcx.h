//
//  init.h
//  vcx
//
//  Created by GuestUser on 4/30/18.
//  Copyright © 2018 GuestUser. All rights reserved.
//

#ifndef init_h
#define init_h
#import "libvcx.h"

@interface ConnectMeVcx : NSObject

- (vcx_error_t) vcxInitCore:(NSString *)config;
- (vcx_error_t) vcxInitThreadpool:(NSString *)config;
- (void) vcxOpenWallet:(void (^)(NSError *error)) completion;
- (void) createWallet:(NSString *)config
                 completion:(void (^)(NSError *error))completion;
- (void) openMainWallet:(NSString *)config
                 completion:(void (^)(NSError *error, NSNumber *handle))completion;
- (void) closeMainWallet:(void (^)(NSError *error)) completion;
- (void) vcxOpenPool:(void (^)(NSError *error)) completion;
- (void) vcxOpenMainPool:(NSString *)config
                 completion:(void (^)(NSError *error))completion;
- (void) updateWebhookUrl:(NSString *) notification_webhook_url
           withCompletion:(void (^)(NSError *error))completion;

- (void)agentProvisionAsync:(NSString *)config
                 completion:(void (^)(NSError *error, NSString *config))completion;

- (void) vcxProvisionCloudAgent:(NSString *)config
                 completion:(void (^)(NSError *error, NSString *config))completion;

- (void) vcxCreateAgencyClientForMainWallet:(NSString *)config
                 completion:(void (^)(NSError *error))completion;

- (NSString *)errorCMessage:(NSInteger) errorCode;

- (void)connectionCreateWithInvite:(NSString *)invitationId
                     inviteDetails:(NSString *)inviteDetails
                        completion:(void (^)(NSError *error, NSNumber *connectionHandle))completion;

- (void)connectionConnect:(NSNumber *)connectionHandle
           connectionType:(NSString *)connectionType
               completion:(void (^)(NSError *error))completion;

- (void)connectionGetState:(NSNumber *)connectionHandle
                completion:(void (^)(NSError *error, NSNumber *state))completion;

- (void)connectionUpdateState:(NSNumber *)connectionHandle
                   completion:(void (^)(NSError *error, NSNumber *state))completion;

- (void)connectionSerialize:(NSNumber *)connectionHandle
                 completion:(void (^)(NSError *error, NSString *serializedConnection))completion;

- (void)connectionDeserialize:(NSString *)serializedConnection
                   completion:(void (^)(NSError *error, NSNumber *connectionHandle))completion;

- (int)connectionRelease:(NSNumber *)connectionHandle;

- (void)deleteConnection:(NSNumber *)connectionHandle
          withCompletion:(void (^)(NSError *error))completion;

- (void)connectionGetPwDid:(NSNumber *)connectionHandle
                completion:(void (^)(NSError *error, NSString *pwDid))completion;

- (void)connectionGetTheirPwDid:(NSNumber *)connectionHandle
                     completion:(void (^)(NSError *error, NSString *theirPwDid))completion;

- (void)connectionSendMessage:(NSNumber *)connectionHandle
                  withMessage:(NSString *)message
       withSendMessageOptions:(NSString *)sendMessageOptions
               withCompletion:(void (^)(NSError *error, NSString *msg_id))completion;

- (void)connectionSignData:(NSNumber *)connectionHandle
                  withData:(NSData *)dataRaw
            withCompletion:(void (^)(NSError *error, NSData *signature_raw))completion;

- (void)connectionVerifySignature:(NSNumber *)connectionHandle
                         withData:(NSData *)dataRaw
                withSignatureData:(NSData *)signatureRaw
                   withCompletion:(void (^)(NSError *error, vcx_bool_t valid))completion;

- (void)connectionDownloadMessages:(NSNumber *)connectionHandle
                    messageStatus:(NSString *)messageStatus
                            uid_s:(NSString *)uid_s
                      completion:(void (^)(NSError *error, NSString* messages))completion;

- (void)connectionSendHandshakeReuse:(NSNumber *)connectionHandle
                              oobMsg:(NSString *)oobMsg
                          completion:(void (^)(NSError *error))completion;

- (void)agentUpdateInfo:(NSString *)config
             completion:(void (^)(NSError *error))completion;

- (void)getCredential:(NSNumber *)credentialHandle
           completion:(void (^)(NSError *error, NSString *credential))completion;

- (void)credentialCreateWithOffer:(NSString *)sourceId
                            offer:(NSString *)credentialOffer
                       completion:(void (^)(NSError *error, NSNumber *credentialHandle))completion;

- (void)credentialCreateWithMsgid:(NSString *)sourceId
                 connectionHandle:(NSNumber *)connectionHandle
                            msgId:(NSString *)msgId
                       completion:(void (^)(NSError *error, NSNumber *credentialHandle, NSString *credentialOffer))completion;

- (void)credentialSendRequest:(NSNumber *)credentialHandle
             connectionHandle:(NSNumber *)connectionHandle
                paymentHandle:(NSNumber *)paymentHandle
                   completion:(void (^)(NSError *error))completion;

- (void)credentialGetState:(NSNumber *)credentialHandle
                completion:(void (^)(NSError *error, NSNumber *state))completion;

- (void)credentialUpdateState:(NSNumber *)credentialHandle
                completion:(void (^)(NSError *error, NSNumber *state))completion;

- (void)credentialUpdateStateV2:(NSNumber *)credentailHandle
                connectionHandle:(NSNumber *)connectionHandle
                completion:(void (^)(NSError *error, NSNumber *state))completion;

- (void)credentialUpdateStateWithMessageV2:(NSNumber *)credentialHandle
                connectionHandle:(NSNumber *)connectionHandle
                message:(NSString *)message
                completion:(void (^)(NSError *error, NSNumber *state))completion;

- (void)credentialGetOffers:(NSNumber *)connectionHandle
                 completion:(void (^)(NSError *error, NSString *offers))completion;

- (void)credentialGetAttributes:(NSNumber *)credentialHandle
                 completion:(void (^)(NSError *error, NSString *attributes))completion;

- (void)credentialGetAttachment:(NSNumber *)credentialHandle
                 completion:(void (^)(NSError *error, NSString *attach))completion;

- (void)credentialGetTailsLocation:(NSNumber *)credentialHandle
                 completion:(void (^)(NSError *error, NSString *tailsLocation))completion;

- (void)credentialGetTailsHash:(NSNumber *)credentialHandle
                 completion:(void (^)(NSError *error, NSString *tailsHash))completion;

- (void)credentialGetRevRegId:(NSNumber *)credentialHandle
                 completion:(void (^)(NSError *error, NSString *revRegId))completion;

- (void)credentialIsRevokable:(NSNumber *)credentialHandle
                 completion:(void (^)(NSError *error, BOOL revokable))completion;

- (void)credentialSerialize:(NSNumber *)credentialHandle
                 completion:(void (^)(NSError *error, NSString *state))completion;

- (void)credentialDeserialize:(NSString *)serializedCredential
                   completion:(void (^)(NSError *error, NSNumber *credentialHandle))completion;

- (int)credentialRelease:(NSNumber *) credentialHandle;

- (void)deleteCredential:(NSNumber *)credentialHandle
                  completion:(void (^)(NSError *error))completion;

- (void)exportWallet:(NSString *)exportPath
         encryptWith:(NSString *)encryptionKey
          completion:(void (^)(NSError *error))completion;

- (void)importWallet:(NSString *)config
           completion:(void (^)(NSError *error))completion;

- (void)addRecordWallet:(NSString *)recordType
               recordId:(NSString *)recordId
            recordValue:(NSString *)recordValue
               tagsJson:(NSString *)tagsJson
           completion:(void (^)(NSError *error))completion;

- (void)updateRecordWallet:(NSString *)recordType
              withRecordId:(NSString *)recordId
           withRecordValue:(NSString *) recordValue
            withCompletion:(void (^)(NSError *error))completion;

- (void)getRecordWallet:(NSString *)recordType
               recordId:(NSString *)recordId
            optionsJson:(NSString *)optionsJson
             completion:(void (^)(NSError *error, NSString *walletValue))completion;

- (void)deleteRecordWallet:(NSString *)recordType
            recordId:(NSString *)recordId
           completion:(void (^)(NSError *error))completion;

- (void)addRecordTagsWallet:(NSString *)recordType
                   recordId:(NSString *)recordId
                   tagsJson:(NSString *) tagsJson
                 completion:(void (^)(NSError *error))completion;

- (void)updateRecordTagsWallet:(NSString *)recordType
                      recordId:(NSString *)recordId
                      tagsJson:(NSString *) tagsJson
                    completion:(void (^)(NSError *error))completion;

- (void)deleteRecordTagsWallet:(NSString *)recordType
                      recordId:(NSString *)recordId
                  tagNamesJson:(NSString *)tagNamesJson
                    completion:(void (^)(NSError *error))completion;

- (void)openSearchWallet:(NSString *)recordType
               queryJson:(NSString *)queryJson
             optionsJson:(NSString *)optionsJson
              completion:(void (^)(NSError *error, NSNumber *searchHandle))completion;

- (void)searchNextRecordsWallet:(NSNumber *)searchHandle
                          count:(NSNumber *)count
                     completion:(void (^)(NSError *error, NSString* records))completion;

- (void)closeSearchWallet:(NSNumber *)searchHandle
               completion:(void (^)(NSError *error))completion;

- (void) proofGetRequests:(NSNumber *)connectionHandle
              completion:(void (^)(NSError *error, NSString *requests))completion;

- (void) proofGetProofRequestAttachment:(NSNumber *)proofHandle
              completion:(void (^)(NSError *error, NSString *attach))completion;

- (void) proofRetrieveCredentials:(NSNumber *)proofHandle
                   withCompletion:(void (^)(NSError *error, NSString *matchingCredentials))completion;

- (void) proofGenerate:(NSNumber *)proofHandle
withSelectedCredentials:(NSString *)selectedCredentials
 withSelfAttestedAttrs:(NSString *)selfAttestedAttributes
        withCompletion:(void (^)(NSError *error))completion;

- (void) proofCreateWithMsgId:(NSString *)source_id
         withConnectionHandle:(NSNumber *)connectionHandle
                    withMsgId:(NSString *)msgId
               withCompletion:(void (^)(NSError *error, NSNumber *proofHandle, NSString *proofRequest))completion;

- (void) proofSend:(vcx_proof_handle_t)proof_handle
withConnectionHandle:(vcx_connection_handle_t)connection_handle
    withCompletion:(void (^)(NSError *error))completion;

- (void)proofGetState:(NSNumber *)proofHandle
           completion:(void (^)(NSError *error, NSNumber *state))completion;

- (void)proofUpdateState:(NSNumber *) proofHandle
              completion:(void (^)(NSError *error, NSNumber *state))completion;

- (void)proofUpdateStateV2:(NSNumber *) proofHandle
              connectionHandle:(NSNumber *)connectionHandle
              completion:(void (^)(NSError *error, NSNumber *state))completion;

- (void)proofUpdateStateWithMessageV2:(NSNumber *) proofHandle
              connectionHandle:(NSNumber *)connectionHandle
              message:(NSString *)message
              completion:(void (^)(NSError *error, NSNumber *state))completion;

- (void) proofReject: (NSNumber *)proof_handle
      withConnectionHandle:(NSNumber *)connection_handle
      withCompletion: (void (^)(NSError *error))completion;

- (void) getProofMsg:(NSNumber *) proofHandle
      withCompletion:(void (^)(NSError *error, NSString *proofMsg))completion;

- (void) getRejectMsg:(NSNumber *) proofHandle
       withCompletion:(void (^)(NSError *error, NSString *rejectMsg))completion;

- (void) proofCreateWithRequest:(NSString *) source_id
               withProofRequest:(NSString *) proofRequest
                 withCompletion:(void (^)(NSError *error, NSNumber *proofHandle))completion;

- (void) proofSerialize:(NSNumber *) proofHandle
         withCompletion:(void (^)(NSError *error, NSString *proof_request))completion;

- (void) proofDeserialize:(NSString *) serializedProof
           withCompletion:(void (^)(NSError *error, NSNumber *proofHandle)) completion;

- (int)proofRelease:(NSNumber *) proofHandle;

- (int)vcxShutdown:(BOOL *)deleteWallet;

- (void)downloadMessagesV2:(NSString *)connectionHandles
            messageStatus:(NSString *)messageStatus
                    uid_s:(NSString *)uid_s
              completion:(void (^)(NSError *error, NSString* messages))completion;

- (void)updateMessages:(NSString *)messageStatus
            pwdidsJson:(NSString *)pwdidsJson
            completion:(void (^)(NSError *error))completion;

- (void)downloadAgentMessages:(NSString *)messageStatus
                    uid_s:(NSString *)uid_s
                    completion:(void (^)(NSError *error, NSString* messages))completion;

- (void) getTxnAuthorAgreement:(void(^)(NSError *error, NSString *authorAgreement)) completion;

- (vcx_error_t) activateTxnAuthorAgreement:(NSString *)text
                               withVersion:(NSString *)version
                                  withHash:(NSString *)hash
                             withMechanism:(NSString *)mechanism
                             withTimestamp:(long)timestamp;
@end

#endif /* init_h */
