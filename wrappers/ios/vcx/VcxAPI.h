//
//  init.h
//  vcx
//
//  Created by GuestUser on 4/30/18.
//  Copyright © 2018 GuestUser. All rights reserved.
//

#ifndef init_h
#define init_h

#import <Foundation/Foundation.h>
#import "VcxTypes.h"

@interface VcxAPI : NSObject

- (void)vcxInitIssuerConfig:(NSString *)config
                 completion:(void (^)(NSError *error))completion;

- (vcx_error_t)vcxPoolSetHandle:(NSInteger)handle
                     completion:(void (^)(NSError *error))completion;

- (void)vcxEndorseTransaction:(NSString *)transaction
                   completion:(void (^)(NSError *error))completion;

- (void)vcxRotateVerKey:(NSString *)did
             completion:(void (^)(NSError *error))completion;

- (void)vcxRotateVerKeyStart:(NSString *)did
                  completion:(void (^)(NSError *error))completion;

- (void)vcxRotateVerKeyApply:(NSString *)did
                  tempVerKey:(NSString *)tempVerKey
                  completion:(void (^)(NSError *error))completion;

- (void)vcxGetVerKeyFromWallet:(NSString *)did
                    completion:(void (^)(NSError *error, NSString *verKey))completion;

- (void)vcxGetVerKeyFromLedger:(NSString *)did
                    completion:(void (^)(NSError *error, NSString *verKey))completion;

- (void)vcxGetLedgerTxn:(NSString *)submitterDid
                  seqNo:(NSInteger)seqNo
             completion:(void (^)(NSError *error, NSString *verKey))completion;

- (vcx_error_t)vcxInitThreadPool:(NSString *)config;

- (void)createWallet:(NSString *)config
          completion:(void (^)(NSError *error))completion;

- (void)vcxConfigureIssuerWallet:(NSString *)seed
                      completion:(void (^)(NSError *error, NSString *conf))completion;

- (void)openMainWallet:(NSString *)config
            completion:(void (^)(NSError *error, NSInteger handle))completion;

- (void)closeMainWallet:(void (^)(NSError *error))completion;

//- (void) vcxOpenPool:(void (^)(NSError *error)) completion;
- (void)vcxOpenMainPool:(NSString *)config
             completion:(void (^)(NSError *error))completion;

- (void)updateWebhookUrl:(NSString *)notification_webhook_url
          withCompletion:(void (^)(NSError *error))completion;

- (void)vcxProvisionCloudAgent:(NSString *)config
                    completion:(void (^)(NSError *error, NSString *config))completion;

- (void)vcxCreateAgencyClientForMainWallet:(NSString *)config
                                completion:(void (^)(NSError *error))completion;

- (NSString *)errorCMessage:(NSInteger)errorCode;

- (NSString *)vcxVersion;

- (void)vcxSchemaSerialize:(NSInteger)schemaHandle
                completion:(void (^)(NSError *error, NSString *serializedSchema))completion;

- (void)vcxSchemaDeserialize:(NSString *)serializedSchema
                  completion:(void (^)(NSError *error, NSInteger schemaHandle))completion;

- (void)vcxSchemaGetAttributes:(NSString *)sourceId
                    sequenceNo:(NSInteger)sequenceNo
                    completion:(void (^)(NSError *error, NSString *schemaAttributes))completion;

- (void)vcxSchemaCreate:(NSString *)sourceId
             schemaName:(NSString *)schemaName
          schemaVersion:(NSString *)schemaVersion
             schemaData:(NSString *)schemaData
          paymentHandle:(NSInteger)paymentHandle
             completion:(void (^)(NSError *error, NSInteger credDefHandle))completion;

- (void)vcxSchemaPrepareForEndorser:(NSString *)sourceId
                         schemaName:(NSString *)schemaName
                      schemaVersion:(NSString *)schemaVersion
                         schemaData:(NSString *)schemaData
                           endorser:(NSString *)endorser
                         completion:(void (^)(
                                 NSError *error,
                                 NSInteger schemaHandle,
                                 NSString *schemaTransaction
                         ))completion;

- (void)vcxSchemaGetSchemaId:(NSString *)sourceId
                schemaHandle:(NSInteger)schemaHandle
                  completion:(void (^)(NSError *error, NSString *schemaId))completion;

- (void)vcxSchemaUpdateState:(NSString *)sourceId
                schemaHandle:(NSInteger)schemaHandle
                  completion:(void (^)(NSError *error, NSInteger state))completion;

- (int)vcxSchemaRelease:(NSInteger)schemaHandle;

- (void)vcxPublicAgentCreate:(NSString *)sourceId
              institutionDid:(NSString *)institutionDid
                  completion:(void (^)(
                          NSError *error,
                          NSInteger agentHandle
                  ))completion;

- (void)vcxGeneratePublicInvite:(NSString *)publicDid
                          label:(NSString *)label
                     completion:(void (^)(
                             NSError *error,
                             NSString *publicInvite
                     ))completion;

- (void)vcxPublicAgentDownloadConnectionRequests:(NSInteger)agentHandle
                                            uids:(NSString *)ids
                                      completion:(void (^)(
                                              NSError *error,
                                              NSString *requests
                                      ))completion;

- (void)vcxPublicAgentDownloadMessage:(NSInteger)agentHandle
                                  uid:(NSString *)id
                           completion:(void (^)(NSError *error, NSString *message))completion;

- (void)vcxPublicAgentGetService:(NSInteger)agentHandle
                      completion:(void (^)(NSError *error, NSString *service))completion;

- (void)vcxPublicAgentSerialize:(NSInteger)agentHandle
                     completion:(void (^)(NSError *error, NSString *serializedAgent))completion;

- (int)vcxPublicAgentRelease:(NSInteger)agentHandle;

- (void)vcxOutOfBandSenderCreate:(NSString *)config
                      completion:(void (^)(NSError *error, NSInteger oobHandle))completion;

- (void)vcxOutOfBandReceiverCreate:(NSString *)message
                        completion:(void (^)(NSError *error, NSInteger oobHandle))completion;

- (void)vcxOutOfBandSenderAppendMessage:(NSInteger)oobHandle
                                message:(NSString *)message
                             completion:(void (^)(NSError *error))completion;

- (void)vcxOutOfBandSenderAppendService:(NSInteger)oobHandle
                                service:(NSString *)service
                             completion:(void (^)(NSError *error))completion;

- (void)vcxOutOfBandSenderAppendServiceDid:(NSInteger)oobHandle
                                       did:(NSString *)did
                                completion:(void (^)(NSError *error))completion;

- (void)vcxOutOfBandSenderGetThreadId:(NSInteger)oobHandle
                           completion:(void (^)(NSError *error, NSString *threadId))completion;

- (void)vcxOutOfBandReceiverGetThreadId:(NSInteger)oobHandle
                             completion:(void (^)(NSError *error, NSString *threadId))completion;

- (void)vcxOutOfBandReceiverExtractMessage:(NSInteger)oobHandle
                                completion:(void (^)(NSError *error, NSString *message))completion;

- (void)vcxOutOfBandToMessage:(NSInteger)oobHandle
                   completion:(void (^)(NSError *error, NSString *message))completion;

- (void)vcxOutOfBandSenderSerialize:(NSInteger)oobHandle
                         completion:(void (^)(NSError *error, NSString *oobMessage))completion;

- (void)vcxOutOfBandReceiverSerialize:(NSInteger)oobHandle
                           completion:(void (^)(NSError *error, NSString *oobMessage))completion;

- (void)vcxOutOfBandSenderDeserialize:(NSString *)oobMessage
                           completion:(void (^)(NSError *error, NSInteger oobHandle))completion;

- (void)vcxOutOfBandReceiverDeserialize:(NSString *)oobMessage
                             completion:(void (^)(NSError *error, NSInteger oobHandle))completion;

- (int)vcxOutOfBandSenderRelease:(NSInteger)agentHandle;

- (int)vcxOutOfBandReceiverRelease:(NSInteger)agentHandle;

- (void)vcxOutOfBandReceiverConnectionExists:(NSInteger)oobHandle
                           connectionHandles:(NSString *)connectionHandles
                                  completion:(void (^)(
                                          NSError *error,
                                          NSInteger connectionHandle,
                                          Boolean foundOne)
                                  )completion;

- (void)vcxOutOfBandReceiverBuildConnection:(NSInteger)oobHandle
                                 completion:(void (^)(
                                         NSError *error,
                                         NSString *connection)
                                 )completion;

- (void)vcxRevocationRegistryCreate:(NSString *)revRegConfig
                         completion:(void (^)(
                                 NSError *error,
                                 NSInteger revRegHandle)
                         )completion;

- (void)vcxRevocationRegistryPublish:(NSInteger)revRegHandle
                            tailsUrl:(NSString *)tailsUrl
                          completion:(void (^)(
                                  NSError *error,
                                  NSInteger handle)
                          )completion;

- (void)vcxRevocationRegistryPublishRevocations:(NSInteger)revRegHandle
                                     completion:(void (^)(
                                             NSError *error)
                                     )completion;

- (void)vcxRevocationRegistryGetRevRegId:(NSInteger)revRegHandle
                              completion:(void (^)(
                                      NSError *error,
                                      NSString *revRegId)
                              )completion;

- (void)vcxRevocationRegistryGetTailsHash:(NSInteger)revRegHandle
                               completion:(void (^)(
                                       NSError *error,
                                       NSString *tailsHash)
                               )completion;

- (void)vcxRevocationRegistryDeserialize:(NSString *)serializedRevReg
                              completion:(void (^)(
                                      NSError *error,
                                      NSInteger revRegHandle)
                              )completion;

- (void)vcxRevocationRegistrySerialize:(NSInteger)revRegHandle
                            completion:(void (^)(
                                    NSError *error,
                                    NSString *serializedRevReg)
                            )completion;

- (int)vcxRevocationRegistryRelease:(NSInteger)revRegHandle;

- (void)vcxCredentialDefinitionCreateV2:(NSString *)sourceId
                               schemaId:(NSString *)schemaId
                              issuerDid:(NSString *)issuerDid
                                    tag:(NSString *)tag
                      supportRevocation:(Boolean)supportRevocation
                             completion:(void (^)(
                                     NSError *error,
                                     NSInteger credDefHandle)
                             )completion;

- (void)vcxCredentialDefinitionPublish:(NSInteger)credDefHandle
                              tailsUrl:(NSString *)tailsUrl
                            completion:(void (^)(NSError *error))completion;

- (void)vcxCredentialDefinitionDeserialize:(NSString *)serializedCredDef
                                completion:(void (^)(NSError *error, NSInteger credDefHandle))completion;

- (void)vcxCredentialDefinitionSerialize:(NSInteger)credDefHandle
                              completion:(void (^)(NSError *error, NSString *serializedCredDef))completion;

- (int)vcxCredentialDefinitionRelease:(NSInteger)credDefHandle;

- (void)vcxCredentialDefinitionGetCredDefId:(NSInteger)credDefHandle
                                 completion:(void (^)(NSError *error, NSString *credDefId))completion;

- (void)vcxCredentialDefinitionUpdateState:(NSInteger)credDefHandle
                                completion:(void (^)(NSError *error, NSInteger state))completion;

- (void)vcxCredentialDefinitionGetState:(NSInteger)credDefHandle
                             completion:(void (^)(NSError *error, NSInteger state))completion;


- (void)connectionCreate:(NSString *)sourceId
              completion:(void (^)(NSError *error, NSInteger connectionHandle))completion;

- (void)connectionCreateWithInvite:(NSString *)invitationId
                     inviteDetails:(NSString *)inviteDetails
                        completion:(void (^)(NSError *error, NSInteger connectionHandle))completion;

- (void)connectionCreateWithConnectionRequest:(NSString *)sourceId
                                  agentHandle:(NSInteger)agentHandle
                                      request:(NSString *)request
                                   completion:(void (^)(NSError *error, NSInteger connectionHandle))completion;

- (void)connectionConnect:(NSInteger)connectionHandle
           connectionType:(NSString *)connectionType
               completion:(void (^)(NSError *error, NSString *inviteDetails))completion;

- (void)connectionGetState:(NSInteger)connectionHandle
                completion:(void (^)(NSError *error, NSInteger state))completion;

- (void)connectionUpdateState:(NSInteger)connectionHandle
                   completion:(void (^)(NSError *error, NSInteger state))completion;

- (void)connectionUpdateStateWithMessage:(NSInteger)connectionHandle
                                 message:(NSString *)message
                              completion:(void (^)(NSError *error, NSInteger state))completion;

- (void)connectionHandleMessage:(NSInteger)connectionHandle
                        message:(NSString *)message
                     completion:(void (^)(NSError *error))completion;

- (void)connectionSerialize:(NSInteger)connectionHandle
                 completion:(void (^)(NSError *error, NSString *serializedConnection))completion;

- (void)connectionDeserialize:(NSString *)serializedConnection
                   completion:(void (^)(NSError *error, NSInteger connectionHandle))completion;

- (int)connectionRelease:(NSInteger)connectionHandle;

- (void)connectionInviteDetails:(NSInteger)connectionHandle
                     completion:(void (^)(NSError *error, NSString *inviteDetails))completion;

- (void)deleteConnection:(NSInteger)connectionHandle
          withCompletion:(void (^)(NSError *error))completion;

- (void)connectionGetPwDid:(NSInteger)connectionHandle
                completion:(void (^)(NSError *error, NSString *pwDid))completion;

- (void)connectionGetTheirPwDid:(NSInteger)connectionHandle
                     completion:(void (^)(NSError *error, NSString *theirPwDid))completion;

- (void)connectionInfo:(NSInteger)connectionHandle
            completion:(void (^)(NSError *error, NSString *info))completion;

- (void)connectionGetThreadId:(NSInteger)connectionHandle
                   completion:(void (^)(NSError *error, NSString *threadId))completion;

- (void)connectionSendMessage:(NSInteger)connectionHandle
                  withMessage:(NSString *)message
       withSendMessageOptions:(NSString *)sendMessageOptions
               withCompletion:(void (^)(NSError *error, NSString *msg_id))completion;

- (void)connectionSignData:(NSInteger)connectionHandle
                  withData:(NSData *)dataRaw
            withCompletion:(void (^)(NSError *error, NSData *signature_raw, vcx_u32_t signature_len))completion;

- (void)connectionVerifySignature:(NSInteger)connectionHandle
                         withData:(NSData *)dataRaw
                withSignatureData:(NSData *)signatureRaw
                   withCompletion:(void (^)(NSError *error, vcx_bool_t valid))completion;

- (void)connectionSendPing:(NSInteger)connectionHandle
                   comment:(NSString *)comment
                completion:(void (^)(NSError *error))completion;

- (void)connectionSendDiscoveryFeatures:(NSInteger)connectionHandle
                                  query:(NSString *)query
                                comment:(NSString *)comment
                             completion:(void (^)(NSError *error))completion;

- (void)connectionDownloadMessages:(NSInteger)connectionHandle
                     messageStatus:(NSString *)messageStatus
                             uid_s:(NSString *)uid_s
                        completion:(void (^)(NSError *error, NSString *messages))completion;

- (void)connectionSendHandshakeReuse:(NSInteger)connectionHandle
                              oobMsg:(NSString *)oobMsg
                          completion:(void (^)(NSError *error))completion;

- (void)issuerCreateCredential:(NSString *)sourceId
                    completion:(void (^)(NSError *error, NSInteger credentialHandle))completion;

- (void)issuerRevokeCredentialLocal:(NSInteger)credentialHandle
                         completion:(void (^)(NSError *error))completion;

- (void)issuerCredentialIsRevokable:(NSInteger)credentialHandle
                         completion:(void (^)(NSError *error, Boolean isRevokable))completion;

- (void)issuerSendCredentialOfferV2:(NSInteger)credentialHandle
                   connectionHandle:(NSInteger)connectionHandle
                         completion:(void (^)(NSError *error))completion;

- (void)markCredentialOfferSent:(NSInteger)credentialHandle
                     completion:(void (^)(NSError *error, NSString *message))completion;

- (void)issuerBuildCredentialOfferMessageV2:(NSInteger)credDefHandle
                               revRegHandle:(NSInteger)revRegHandle
                             credentialData:(NSString *)credData
                                    comment:(NSString *)comment
                                 completion:(void (^)(NSError *error, NSString *message))completion;

- (void)issuerGetCredentialOfferMessage:(NSInteger)credentialHandle
                             completion:(void (^)(NSError *error, NSString *message))completion;

- (void)issuerGetCredentialMessage:(NSInteger)credentialHandle
                     myPairwiseDid:(NSString *)myPwDid
                        completion:(void (^)(NSError *error, NSString *message))completion;

- (void)issuerCredentialGetState:(NSInteger)credentialHandle
                      completion:(void (^)(NSError *error, NSInteger state))completion;

- (void)issuerCredentialGetRevRegId:(NSInteger)credentialHandle
                         completion:(void (^)(NSError *error, NSString *revRegId))completion;

- (void)issuerSendCredential:(NSInteger)credentialHandle
            connectionHandle:(NSInteger)connectionHandle
                  completion:(void (^)(NSError *error, NSInteger state))completion;

- (void)issuerCredentialSerialize:(NSInteger)credentialHandle
                       completion:(void (^)(NSError *error, NSString *serializedCredential))completion;

- (void)issuerCredentialDeserialize:(NSString *)serializedCredential
                         completion:(void (^)(NSError *error, NSInteger credentialHandle))completion;

- (void)issuerCredentialUpdateStateV2:(NSInteger)credentialHandle
                     connectionHandle:(NSInteger)connectionHandle
                           completion:(void (^)(NSError *error, NSInteger state))completion;

- (void)issuerCredentialUpdateStateWithMessageV2:(NSInteger)credentialHandle
                                connectionHandle:(NSInteger)connectionHandle
                                         message:(NSString *)message
                                      completion:(void (^)(NSError *error, NSInteger state))completion;

- (void)issuerCredentialGetThreadId:(NSInteger)credentialHandle
                         completion:(void (^)(NSError *error, NSString *threadId))completion;

- (int)issuerCredentialRelease:(NSInteger)credentialHandle;

- (void)getCredential:(NSInteger)credentialHandle
           completion:(void (^)(NSError *error, NSString *credential))completion;

- (void)credentialCreateWithOffer:(NSString *)sourceId
                            offer:(NSString *)credentialOffer
                       completion:(void (^)(NSError *error, NSInteger credentialHandle))completion;

- (void)credentialCreateWithMsgid:(NSString *)sourceId
                 connectionHandle:(NSInteger)connectionHandle
                            msgId:(NSString *)msgId
                       completion:(void (^)(NSError *error, NSInteger credentialHandle, NSString *credentialOffer))completion;

- (void)credentialSendRequest:(NSInteger)credentialHandle
             connectionHandle:(NSInteger)connectionHandle
                paymentHandle:(NSInteger)paymentHandle
                   completion:(void (^)(NSError *error))completion;

- (void)credentialGetRequestMessage:(NSInteger)credentialHandle
                      myPairwiseDid:(NSString *)myPwDid
                  theirdPairwiseDid:(NSString *)theirPwDid
                      paymentHandle:(NSInteger)paymentHandle
                         completion:(void (^)(NSError *error, NSString *message))completion;

- (void)credentialDeclineOffer:(NSInteger)credentialHandle
              connectionHandle:(NSInteger)connectionHandle
                       comment:(NSString *)comment
                    completion:(void (^)(NSError *error))completion;

- (void)credentialGetState:(NSInteger)credentialHandle
                completion:(void (^)(NSError *error, NSInteger state))completion;

- (void)credentialUpdateStateV2:(NSInteger)credentailHandle
               connectionHandle:(NSInteger)connectionHandle
                     completion:(void (^)(NSError *error, NSInteger state))completion;

- (void)credentialUpdateStateWithMessageV2:(NSInteger)credentialHandle
                          connectionHandle:(NSInteger)connectionHandle
                                   message:(NSString *)message
                                completion:(void (^)(NSError *error, NSInteger state))completion;

- (void)credentialGetOffers:(NSInteger)connectionHandle
                 completion:(void (^)(NSError *error, NSString *offers))completion;

- (void)credentialGetAttributes:(NSInteger)credentialHandle
                     completion:(void (^)(NSError *error, NSString *attributes))completion;

- (void)credentialGetAttachment:(NSInteger)credentialHandle
                     completion:(void (^)(NSError *error, NSString *attach))completion;

- (void)credentialGetTailsLocation:(NSInteger)credentialHandle
                        completion:(void (^)(NSError *error, NSString *tailsLocation))completion;

- (void)credentialGetTailsHash:(NSInteger)credentialHandle
                    completion:(void (^)(NSError *error, NSString *tailsHash))completion;

- (void)credentialGetRevRegId:(NSInteger)credentialHandle
                   completion:(void (^)(NSError *error, NSString *revRegId))completion;

- (void)credentialIsRevokable:(NSInteger)credentialHandle
                   completion:(void (^)(NSError *error, vcx_bool_t revokable))completion;

- (void)credentialSerialize:(NSInteger)credentialHandle
                 completion:(void (^)(NSError *error, NSString *state))completion;

- (void)credentialDeserialize:(NSString *)serializedCredential
                   completion:(void (^)(NSError *error, NSInteger credentialHandle))completion;

- (int)credentialRelease:(NSInteger)credentialHandle;

- (void)deleteCredential:(NSInteger)credentialHandle
              completion:(void (^)(NSError *error))completion;

- (int)walletSetHandle:(NSInteger)handle;

- (void)exportWallet:(NSString *)exportPath
         encryptWith:(NSString *)encryptionKey
          completion:(void (^)(NSError *error, NSInteger exportHandle))completion;

- (void)importWallet:(NSString *)config
          completion:(void (^)(NSError *error))completion;

- (void)addRecordWallet:(NSString *)recordType
               recordId:(NSString *)recordId
            recordValue:(NSString *)recordValue
               tagsJson:(NSString *)tagsJson
             completion:(void (^)(NSError *error))completion;

- (void)updateRecordWallet:(NSString *)recordType
              withRecordId:(NSString *)recordId
           withRecordValue:(NSString *)recordValue
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
                   tagsJson:(NSString *)tagsJson
                 completion:(void (^)(NSError *error))completion;

- (void)updateRecordTagsWallet:(NSString *)recordType
                      recordId:(NSString *)recordId
                      tagsJson:(NSString *)tagsJson
                    completion:(void (^)(NSError *error))completion;

- (void)deleteRecordTagsWallet:(NSString *)recordType
                      recordId:(NSString *)recordId
                  tagNamesJson:(NSString *)tagNamesJson
                    completion:(void (^)(NSError *error))completion;

- (void)openSearchWallet:(NSString *)recordType
               queryJson:(NSString *)queryJson
             optionsJson:(NSString *)optionsJson
              completion:(void (^)(NSError *error, NSInteger searchHandle))completion;

- (void)searchNextRecordsWallet:(NSInteger)searchHandle
                          count:(int)count
                     completion:(void (^)(NSError *error, NSString *records))completion;

- (void)closeSearchWallet:(NSInteger)searchHandle
               completion:(void (^)(NSError *error))completion;

- (void)verifierProofCreate:(NSString *)proofRequestId
             requestedAttrs:(NSString *)requestedAttrs
        requestedPredicates:(NSString *)requestedPredicates
         revocationInterval:(NSString *)revocationInterval
                  proofName:(NSString *)proofName
                 completion:(void (^)(NSError *error, NSInteger proofHandle))completion;

- (void)verifierProofSendRequest:(NSInteger)proofHandle
                connectionHandle:(NSInteger)connectionHandle
                      completion:(void (^)(NSError *error))completion;

- (void)verifierGetProofMessage:(NSInteger)proofHandle
                     completion:(void (^)(NSError *error, NSInteger state, NSString *responseData))completion;

- (void)verifierProofGetRequestMessage:(NSInteger)proofHandle
                            completion:(void (^)(NSError *error, NSString *message))completion;

- (void)verifierProofUpdateStateV2:(NSInteger)proofHandle
                  connectionHandle:(NSInteger)connectionHandle
                        completion:(void (^)(NSError *error, NSInteger state))completion;

- (void)verifierProofUpdateStateWithMessageV2:(NSInteger)proofHandle
                             connectionHandle:(NSInteger)connectionHandle
                                      message:(NSString *)message
                                   completion:(void (^)(NSError *error, NSInteger state))completion;

- (void)verifierProofGetState:(NSInteger)proofHandle
                   completion:(void (^)(NSError *error, NSInteger state))completion;

- (void)verifierProofGetThreadId:(NSInteger)proofHandle
                      completion:(void (^)(NSError *error, NSString *threadId))completion;

- (void)verifierMarkPresentationRequestMessageSent:(NSInteger)proofHandle
                                        completion:(void (^)(NSError *error, NSInteger state, NSString *message))completion;

- (void)verifierProofSerialize:(NSInteger)proofHandle
                    completion:(void (^)(NSError *error, NSString *serializedProof))completion;

- (void)verifierProofDeserialize:(NSString *)serializedProof
                      completion:(void (^)(NSError *error, NSInteger proofHandle))completion;

- (int)verifierProofRelease:(NSInteger)proofHandle;

- (void)proofGetRequests:(NSInteger)connectionHandle
              completion:(void (^)(NSError *error, NSString *requests))completion;

- (void)proofGetProofRequestAttachment:(NSInteger)proofHandle
                            completion:(void (^)(NSError *error, NSString *attach))completion;

- (void)proofRetrieveCredentials:(NSInteger)proofHandle
                  withCompletion:(void (^)(NSError *error, NSString *matchingCredentials))completion;

- (void)  proofGenerate:(NSInteger)proofHandle
withSelectedCredentials:(NSString *)selectedCredentials
  withSelfAttestedAttrs:(NSString *)selfAttestedAttributes
         withCompletion:(void (^)(NSError *error))completion;

- (void)proofCreateWithMsgId:(NSString *)source_id
        withConnectionHandle:(NSInteger)connectionHandle
                   withMsgId:(NSString *)msgId
              withCompletion:(void (^)(NSError *, NSInteger, NSString *))completion;

- (void)   proofSend:(NSInteger)proof_handle
withConnectionHandle:(NSInteger)connection_handle
      withCompletion:(void (^)(NSError *error))completion;

- (void)proofGetState:(NSInteger)proofHandle
           completion:(void (^)(NSError *error, NSInteger state))completion;

- (void)proofUpdateStateV2:(NSInteger)proofHandle
          connectionHandle:(NSInteger)connectionHandle
                completion:(void (^)(NSError *error, NSInteger state))completion;

- (void)proofUpdateStateWithMessageV2:(NSInteger)proofHandle
                     connectionHandle:(NSInteger)connectionHandle
                              message:(NSString *)message
                           completion:(void (^)(NSError *error, NSInteger state))completion;

- (void) proofReject:(NSInteger)proof_handle
withConnectionHandle:(NSInteger)connection_handle
      withCompletion:(void (^)(NSError *error))completion;

- (void)proofDeclinePresentationRequest:(NSInteger)proof_handle
                       connectionHandle:(NSInteger)connection_handle
                                 reason:(NSString *)reason
                               proposal:(NSString *)proposal
                             completion:(void (^)(NSError *error))completion;

- (void)proofGetThreadId:(NSInteger)proofHandle
          withCompletion:(void (^)(NSError *error, NSString *threadId))completion;

- (void)getProofMsg:(NSInteger)proofHandle
     withCompletion:(void (^)(NSError *error, NSString *proofMsg))completion;

- (void)getRejectMsg:(NSInteger)proofHandle
      withCompletion:(void (^)(NSError *error, NSString *rejectMsg))completion;

- (void)proofCreateWithRequest:(NSString *)source_id
              withProofRequest:(NSString *)proofRequest
                withCompletion:(void (^)(NSError *, NSInteger))completion;

- (void)proofSerialize:(NSInteger)proofHandle
        withCompletion:(void (^)(NSError *error, NSString *proof_request))completion;

- (void)proofDeserialize:(NSString *)serializedProof
          withCompletion:(void (^)(NSError *, NSInteger))completion;

- (int)proofRelease:(NSInteger)proofHandle;

- (int)vcxShutdown:(Boolean)deleteWallet;

- (void)downloadMessagesV2:(NSString *)connectionHandles
             messageStatus:(NSString *)messageStatus
                     uid_s:(NSString *)uid_s
                completion:(void (^)(NSError *error, NSString *messages))completion;

- (void)updateMessages:(NSString *)messageStatus
            pwdidsJson:(NSString *)pwDidsJson
            completion:(void (^)(NSError *error))completion;

- (void)getTxnAuthorAgreement:(void (^)(NSError *error, NSString *authorAgreement))completion;

- (vcx_error_t)activateTxnAuthorAgreement:(NSString *)text
                              withVersion:(NSString *)version
                                 withHash:(NSString *)hash
                            withMechanism:(NSString *)mechanism
                            withTimestamp:(long)timestamp;
@end

#endif /* init_h */
