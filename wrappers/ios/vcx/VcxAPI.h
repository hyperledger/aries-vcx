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

- (vcx_error_t)vcxPoolSetHandle:(NSUInteger)handle
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
                  seqNo:(NSUInteger)seqNo
             completion:(void (^)(NSError *error, NSString *verKey))completion;

- (vcx_error_t)vcxInitThreadPool:(NSString *)config;

- (void)createWallet:(NSString *)config
          completion:(void (^)(NSError *error))completion;

- (void)vcxConfigureIssuerWallet:(NSString *)seed
                      completion:(void (^)(NSError *error, NSString *conf))completion;

- (void)openMainWallet:(NSString *)config
            completion:(void (^)(NSError *error, NSUInteger handle))completion;

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

- (NSString *)errorCMessage:(NSUInteger)errorCode;

- (NSString *)vcxVersion;

- (void)vcxSchemaSerialize:(NSUInteger)schemaHandle
                completion:(void (^)(NSError *error, NSString *serializedSchema))completion;

- (void)vcxSchemaDeserialize:(NSString *)serializedSchema
                  completion:(void (^)(NSError *error, NSUInteger schemaHandle))completion;

- (void)vcxSchemaGetAttributes:(NSString *)sourceId
                    sequenceNo:(NSUInteger)sequenceNo
                    completion:(void (^)(NSError *error, NSString *schemaAttributes))completion;

- (void)vcxSchemaCreate:(NSString *)sourceId
             schemaName:(NSString *)schemaName
          schemaVersion:(NSString *)schemaVersion
             schemaData:(NSString *)schemaData
          paymentHandle:(NSUInteger)paymentHandle
             completion:(void (^)(NSError *error, NSUInteger credDefHandle))completion;

- (void)vcxSchemaPrepareForEndorser:(NSString *)sourceId
                         schemaName:(NSString *)schemaName
                      schemaVersion:(NSString *)schemaVersion
                         schemaData:(NSString *)schemaData
                           endorser:(NSString *)endorser
                         completion:(void (^)(
                                 NSError *error,
                                 NSUInteger schemaHandle,
                                 NSString *schemaTransaction
                         ))completion;

- (void)vcxSchemaGetSchemaId:(NSString *)sourceId
                schemaHandle:(NSUInteger)schemaHandle
                  completion:(void (^)(NSError *error, NSString *schemaId))completion;

- (void)vcxSchemaUpdateState:(NSString *)sourceId
                schemaHandle:(NSUInteger)schemaHandle
                  completion:(void (^)(NSError *error, NSUInteger state))completion;

- (int)vcxSchemaRelease:(NSUInteger)schemaHandle;

- (void)vcxPublicAgentCreate:(NSString *)sourceId
              institutionDid:(NSString *)institutionDid
                  completion:(void (^)(
                          NSError *error,
                          NSUInteger agentHandle
                  ))completion;

- (void)vcxGeneratePublicInvite:(NSString *)publicDid
                          label:(NSString *)label
                     completion:(void (^)(
                             NSError *error,
                             NSString *publicInvite
                     ))completion;

- (void)vcxPublicAgentDownloadConnectionRequests:(NSUInteger)agentHandle
                                            uids:(NSString *)ids
                                      completion:(void (^)(
                                              NSError *error,
                                              NSString *requests
                                      ))completion;

- (void)vcxPublicAgentDownloadMessage:(NSUInteger)agentHandle
                                  uid:(NSString *)id
                           completion:(void (^)(NSError *error, NSString *message))completion;

- (void)vcxPublicAgentGetService:(NSUInteger)agentHandle
                      completion:(void (^)(NSError *error, NSString *service))completion;

- (void)vcxPublicAgentSerialize:(NSUInteger)agentHandle
                     completion:(void (^)(NSError *error, NSString *serializedAgent))completion;

- (int)vcxPublicAgentRelease:(NSUInteger)agentHandle;

- (void)vcxOutOfBandSenderCreate:(NSString *)config
                      completion:(void (^)(NSError *error, NSUInteger oobHandle))completion;

- (void)vcxOutOfBandReceiverCreate:(NSString *)message
                        completion:(void (^)(NSError *error, NSUInteger oobHandle))completion;

- (void)vcxOutOfBandSenderAppendMessage:(NSUInteger)oobHandle
                                message:(NSString *)message
                             completion:(void (^)(NSError *error))completion;

- (void)vcxOutOfBandSenderAppendService:(NSUInteger)oobHandle
                                service:(NSString *)service
                             completion:(void (^)(NSError *error))completion;

- (void)vcxOutOfBandSenderAppendServiceDid:(NSUInteger)oobHandle
                                       did:(NSString *)did
                                completion:(void (^)(NSError *error))completion;

- (void)vcxOutOfBandSenderGetThreadId:(NSUInteger)oobHandle
                           completion:(void (^)(NSError *error, NSString *threadId))completion;

- (void)vcxOutOfBandReceiverGetThreadId:(NSUInteger)oobHandle
                             completion:(void (^)(NSError *error, NSString *threadId))completion;

- (void)vcxOutOfBandReceiverExtractMessage:(NSUInteger)oobHandle
                                completion:(void (^)(NSError *error, NSString *message))completion;

- (void)vcxOutOfBandToMessage:(NSUInteger)oobHandle
                   completion:(void (^)(NSError *error, NSString *message))completion;

- (void)vcxOutOfBandSenderSerialize:(NSUInteger)oobHandle
                         completion:(void (^)(NSError *error, NSString *oobMessage))completion;

- (void)vcxOutOfBandReceiverSerialize:(NSUInteger)oobHandle
                           completion:(void (^)(NSError *error, NSString *oobMessage))completion;

- (void)vcxOutOfBandSenderDeserialize:(NSString *)oobMessage
                           completion:(void (^)(NSError *error, NSUInteger oobHandle))completion;

- (void)vcxOutOfBandReceiverDeserialize:(NSString *)oobMessage
                             completion:(void (^)(NSError *error, NSUInteger oobHandle))completion;

- (int)vcxOutOfBandSenderRelease:(NSUInteger)agentHandle;

- (int)vcxOutOfBandReceiverRelease:(NSUInteger)agentHandle;

- (void)vcxOutOfBandReceiverConnectionExists:(NSUInteger)oobHandle
                           connectionHandles:(NSString *)connectionHandles
                                  completion:(void (^)(
                                          NSError *error,
                                          NSUInteger connectionHandle,
                                          Boolean foundOne)
                                  )completion;

- (void)vcxOutOfBandReceiverBuildConnection:(NSUInteger)oobHandle
                                 completion:(void (^)(
                                         NSError *error,
                                         NSString *connection)
                                 )completion;

- (void)vcxRevocationRegistryCreate:(NSString *)revRegConfig
                         completion:(void (^)(
                                 NSError *error,
                                 NSUInteger revRegHandle)
                         )completion;

- (void)vcxRevocationRegistryPublish:(NSUInteger)revRegHandle
                            tailsUrl:(NSString *)tailsUrl
                          completion:(void (^)(
                                  NSError *error,
                                  NSUInteger handle)
                          )completion;

- (void)vcxRevocationRegistryPublishRevocations:(NSUInteger)revRegHandle
                                     completion:(void (^)(
                                             NSError *error)
                                     )completion;

- (void)vcxRevocationRegistryGetRevRegId:(NSUInteger)revRegHandle
                              completion:(void (^)(
                                      NSError *error,
                                      NSString *revRegId)
                              )completion;

- (void)vcxRevocationRegistryGetTailsHash:(NSUInteger)revRegHandle
                               completion:(void (^)(
                                       NSError *error,
                                       NSString *tailsHash)
                               )completion;

- (void)vcxRevocationRegistryDeserialize:(NSString *)serializedRevReg
                              completion:(void (^)(
                                      NSError *error,
                                      NSUInteger revRegHandle)
                              )completion;

- (void)vcxRevocationRegistrySerialize:(NSUInteger)revRegHandle
                            completion:(void (^)(
                                    NSError *error,
                                    NSString *serializedRevReg)
                            )completion;

- (int)vcxRevocationRegistryRelease:(NSUInteger)revRegHandle;

- (void)vcxCredentialDefinitionCreateV2:(NSString *)sourceId
                               schemaId:(NSString *)schemaId
                              issuerDid:(NSString *)issuerDid
                                    tag:(NSString *)tag
                      supportRevocation:(Boolean)supportRevocation
                             completion:(void (^)(
                                     NSError *error,
                                     NSUInteger credDefHandle)
                             )completion;

- (void)vcxCredentialDefinitionPublish:(NSUInteger)credDefHandle
                              tailsUrl:(NSString *)tailsUrl
                            completion:(void (^)(NSError *error))completion;

- (void)vcxCredentialDefinitionDeserialize:(NSString *)serializedCredDef
                                completion:(void (^)(NSError *error, NSUInteger credDefHandle))completion;

- (void)vcxCredentialDefinitionSerialize:(NSUInteger)credDefHandle
                              completion:(void (^)(NSError *error, NSString *serializedCredDef))completion;

- (int)vcxCredentialDefinitionRelease:(NSUInteger)credDefHandle;

- (void)vcxCredentialDefinitionGetCredDefId:(NSUInteger)credDefHandle
                                 completion:(void (^)(NSError *error, NSString *credDefId))completion;

- (void)vcxCredentialDefinitionUpdateState:(NSUInteger)credDefHandle
                                completion:(void (^)(NSError *error, NSUInteger state))completion;

- (void)vcxCredentialDefinitionGetState:(NSUInteger)credDefHandle
                             completion:(void (^)(NSError *error, NSUInteger state))completion;


- (void)connectionCreate:(NSString *)sourceId
              completion:(void (^)(NSError *error, NSUInteger connectionHandle))completion;

- (void)connectionCreateWithInvite:(NSString *)sourceId
                     inviteDetails:(NSString *)inviteDetails
                        completion:(void (^)(NSError *error, NSUInteger connectionHandle))completion;

- (void)connectionCreateWithConnectionRequest:(NSString *)sourceId
                                       pwInfo:(NSString *)pwInfo
                                      request:(NSString *)request
                                   completion:(void (^)(NSError *error, NSUInteger connectionHandle))completion;

- (void)connectionCreateWithConnectionRequestV2:(NSString *)sourceId
                                  agentHandle:(NSUInteger)agentHandle
                                      request:(NSString *)request
                                   completion:(void (^)(NSError *error, NSUInteger connectionHandle))completion;

- (void)connectionConnect:(NSUInteger)connectionHandle
           connectionType:(NSString *)connectionType
               completion:(void (^)(NSError *error, NSString *inviteDetails))completion;

- (void)connectionGetState:(NSUInteger)connectionHandle
                completion:(void (^)(NSError *error, NSUInteger state))completion;

- (void)connectionUpdateState:(NSUInteger)connectionHandle
                   completion:(void (^)(NSError *error, NSUInteger state))completion;

- (void)connectionUpdateStateWithMessage:(NSUInteger)connectionHandle
                                 message:(NSString *)message
                              completion:(void (^)(NSError *error, NSUInteger state))completion;

- (void)connectionHandleMessage:(NSUInteger)connectionHandle
                        message:(NSString *)message
                     completion:(void (^)(NSError *error))completion;

- (void)connectionSerialize:(NSUInteger)connectionHandle
                 completion:(void (^)(NSError *error, NSString *serializedConnection))completion;

- (void)connectionDeserialize:(NSString *)serializedConnection
                   completion:(void (^)(NSError *error, NSUInteger connectionHandle))completion;

- (int)connectionRelease:(NSUInteger)connectionHandle;

- (void)connectionInviteDetails:(NSUInteger)connectionHandle
                     completion:(void (^)(NSError *error, NSString *inviteDetails))completion;

- (void)deleteConnection:(NSUInteger)connectionHandle
          withCompletion:(void (^)(NSError *error))completion;

- (void)connectionGetPwDid:(NSUInteger)connectionHandle
                completion:(void (^)(NSError *error, NSString *pwDid))completion;

- (void)connectionGetTheirPwDid:(NSUInteger)connectionHandle
                     completion:(void (^)(NSError *error, NSString *theirPwDid))completion;

- (void)connectionInfo:(NSUInteger)connectionHandle
            completion:(void (^)(NSError *error, NSString *info))completion;

- (void)connectionGetThreadId:(NSUInteger)connectionHandle
                   completion:(void (^)(NSError *error, NSString *threadId))completion;

- (void)connectionSendMessage:(NSUInteger)connectionHandle
                  withMessage:(NSString *)message
       withSendMessageOptions:(NSString *)sendMessageOptions
               withCompletion:(void (^)(NSError *error, NSString *msg_id))completion;

- (void)connectionSignData:(NSUInteger)connectionHandle
                  withData:(NSData *)dataRaw
            withCompletion:(void (^)(NSError *error, NSData *signature_raw, vcx_u32_t signature_len))completion;

- (void)connectionVerifySignature:(NSUInteger)connectionHandle
                         withData:(NSData *)dataRaw
                withSignatureData:(NSData *)signatureRaw
                   withCompletion:(void (^)(NSError *error, vcx_bool_t valid))completion;

- (void)connectionSendPing:(NSUInteger)connectionHandle
                   comment:(NSString *)comment
                completion:(void (^)(NSError *error))completion;

- (void)connectionSendDiscoveryFeatures:(NSUInteger)connectionHandle
                                  query:(NSString *)query
                                comment:(NSString *)comment
                             completion:(void (^)(NSError *error))completion;

- (void)connectionDownloadMessages:(NSUInteger)connectionHandle
                     messageStatus:(NSString *)messageStatus
                             uid_s:(NSString *)uid_s
                        completion:(void (^)(NSError *error, NSString *messages))completion;

- (void)connectionSendHandshakeReuse:(NSUInteger)connectionHandle
                              oobMsg:(NSString *)oobMsg
                          completion:(void (^)(NSError *error))completion;

- (void)issuerCreateCredential:(NSString *)sourceId
                    completion:(void (^)(NSError *error, NSUInteger credentialHandle))completion;

- (void)issuerRevokeCredentialLocal:(NSUInteger)credentialHandle
                         completion:(void (^)(NSError *error))completion;

- (void)issuerCredentialIsRevokable:(NSUInteger)credentialHandle
                         completion:(void (^)(NSError *error, Boolean isRevokable))completion;

- (void)issuerSendCredentialOfferV2:(NSUInteger)credentialHandle
                   connectionHandle:(NSUInteger)connectionHandle
                         completion:(void (^)(NSError *error))completion;

- (void)markCredentialOfferSent:(NSUInteger)credentialHandle
                     completion:(void (^)(NSError *error, NSString *message))completion;

- (void)issuerBuildCredentialOfferMessageV2:(NSUInteger)credDefHandle
                               revRegHandle:(NSUInteger)revRegHandle
                             credentialData:(NSString *)credData
                                    comment:(NSString *)comment
                                 completion:(void (^)(NSError *error, NSString *message))completion;

- (void)issuerGetCredentialOfferMessage:(NSUInteger)credentialHandle
                             completion:(void (^)(NSError *error, NSString *message))completion;

- (void)issuerGetCredentialMessage:(NSUInteger)credentialHandle
                     myPairwiseDid:(NSString *)myPwDid
                        completion:(void (^)(NSError *error, NSString *message))completion;

- (void)issuerCredentialGetState:(NSUInteger)credentialHandle
                      completion:(void (^)(NSError *error, NSUInteger state))completion;

- (void)issuerCredentialGetRevRegId:(NSUInteger)credentialHandle
                         completion:(void (^)(NSError *error, NSString *revRegId))completion;

- (void)issuerSendCredential:(NSUInteger)credentialHandle
            connectionHandle:(NSUInteger)connectionHandle
                  completion:(void (^)(NSError *error, NSUInteger state))completion;

- (void)issuerCredentialSerialize:(NSUInteger)credentialHandle
                       completion:(void (^)(NSError *error, NSString *serializedCredential))completion;

- (void)issuerCredentialDeserialize:(NSString *)serializedCredential
                         completion:(void (^)(NSError *error, NSUInteger credentialHandle))completion;

- (void)issuerCredentialUpdateStateV2:(NSUInteger)credentialHandle
                     connectionHandle:(NSUInteger)connectionHandle
                           completion:(void (^)(NSError *error, NSUInteger state))completion;

- (void)issuerCredentialUpdateStateWithMessageV2:(NSUInteger)credentialHandle
                                connectionHandle:(NSUInteger)connectionHandle
                                         message:(NSString *)message
                                      completion:(void (^)(NSError *error, NSUInteger state))completion;

- (void)issuerCredentialGetThreadId:(NSUInteger)credentialHandle
                         completion:(void (^)(NSError *error, NSString *threadId))completion;

- (int)issuerCredentialRelease:(NSUInteger)credentialHandle;

- (void)getCredential:(NSUInteger)credentialHandle
           completion:(void (^)(NSError *error, NSString *credential))completion;

- (void)credentialCreateWithOffer:(NSString *)sourceId
                            offer:(NSString *)credentialOffer
                       completion:(void (^)(NSError *error, NSUInteger credentialHandle))completion;

- (void)credentialCreateWithMsgid:(NSString *)sourceId
                 connectionHandle:(NSUInteger)connectionHandle
                            msgId:(NSString *)msgId
                       completion:(void (^)(NSError *error, NSUInteger credentialHandle, NSString *credentialOffer))completion;

- (void)credentialSendRequest:(NSUInteger)credentialHandle
             connectionHandle:(NSUInteger)connectionHandle
                paymentHandle:(NSUInteger)paymentHandle
                   completion:(void (^)(NSError *error))completion;

- (void)credentialGetRequestMessage:(NSUInteger)credentialHandle
                      myPairwiseDid:(NSString *)myPwDid
                  theirdPairwiseDid:(NSString *)theirPwDid
                      paymentHandle:(NSUInteger)paymentHandle
                         completion:(void (^)(NSError *error, NSString *message))completion;

- (void)credentialDeclineOffer:(NSUInteger)credentialHandle
              connectionHandle:(NSUInteger)connectionHandle
                       comment:(NSString *)comment
                    completion:(void (^)(NSError *error))completion;

- (void)credentialGetState:(NSUInteger)credentialHandle
                completion:(void (^)(NSError *error, NSUInteger state))completion;

- (void)credentialUpdateStateV2:(NSUInteger)credentailHandle
               connectionHandle:(NSUInteger)connectionHandle
                     completion:(void (^)(NSError *error, NSUInteger state))completion;

- (void)credentialUpdateStateWithMessageV2:(NSUInteger)credentialHandle
                          connectionHandle:(NSUInteger)connectionHandle
                                   message:(NSString *)message
                                completion:(void (^)(NSError *error, NSUInteger state))completion;

- (void)credentialGetOffers:(NSUInteger)connectionHandle
                 completion:(void (^)(NSError *error, NSString *offers))completion;

- (void)credentialGetAttributes:(NSUInteger)credentialHandle
                     completion:(void (^)(NSError *error, NSString *attributes))completion;

- (void)credentialGetAttachment:(NSUInteger)credentialHandle
                     completion:(void (^)(NSError *error, NSString *attach))completion;

- (void)credentialGetTailsLocation:(NSUInteger)credentialHandle
                        completion:(void (^)(NSError *error, NSString *tailsLocation))completion;

- (void)credentialGetTailsHash:(NSUInteger)credentialHandle
                    completion:(void (^)(NSError *error, NSString *tailsHash))completion;

- (void)credentialGetRevRegId:(NSUInteger)credentialHandle
                   completion:(void (^)(NSError *error, NSString *revRegId))completion;

- (void)credentialIsRevokable:(NSUInteger)credentialHandle
                   completion:(void (^)(NSError *error, vcx_bool_t revokable))completion;

- (void)credentialSerialize:(NSUInteger)credentialHandle
                 completion:(void (^)(NSError *error, NSString *state))completion;

- (void)credentialDeserialize:(NSString *)serializedCredential
                   completion:(void (^)(NSError *error, NSUInteger credentialHandle))completion;

- (int)credentialRelease:(NSUInteger)credentialHandle;

- (void)deleteCredential:(NSUInteger)credentialHandle
              completion:(void (^)(NSError *error))completion;

- (int)walletSetHandle:(NSUInteger)handle;

- (void)exportWallet:(NSString *)exportPath
         encryptWith:(NSString *)encryptionKey
          completion:(void (^)(NSError *error, NSUInteger exportHandle))completion;

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
              completion:(void (^)(NSError *error, NSUInteger searchHandle))completion;

- (void)searchNextRecordsWallet:(NSUInteger)searchHandle
                          count:(int)count
                     completion:(void (^)(NSError *error, NSString *records))completion;

- (void)closeSearchWallet:(NSUInteger)searchHandle
               completion:(void (^)(NSError *error))completion;

- (void)verifierProofCreate:(NSString *)proofRequestId
             requestedAttrs:(NSString *)requestedAttrs
        requestedPredicates:(NSString *)requestedPredicates
         revocationInterval:(NSString *)revocationInterval
                  proofName:(NSString *)proofName
                 completion:(void (^)(NSError *error, NSUInteger proofHandle))completion;

- (void)verifierProofSendRequest:(NSUInteger)proofHandle
                connectionHandle:(NSUInteger)connectionHandle
                      completion:(void (^)(NSError *error))completion;

- (void)verifierGetProofMessage:(NSUInteger)proofHandle
                     completion:(void (^)(NSError *error, NSUInteger state, NSString *responseData))completion;

- (void)verifierProofGetRequestMessage:(NSUInteger)proofHandle
                            completion:(void (^)(NSError *error, NSString *message))completion;

- (void)verifierProofUpdateStateV2:(NSUInteger)proofHandle
                  connectionHandle:(NSUInteger)connectionHandle
                        completion:(void (^)(NSError *error, NSUInteger state))completion;

- (void)verifierProofUpdateStateWithMessageV2:(NSUInteger)proofHandle
                             connectionHandle:(NSUInteger)connectionHandle
                                      message:(NSString *)message
                                   completion:(void (^)(NSError *error, NSUInteger state))completion;

- (void)verifierProofGetState:(NSUInteger)proofHandle
                   completion:(void (^)(NSError *error, NSUInteger state))completion;

- (void)verifierProofGetThreadId:(NSUInteger)proofHandle
                      completion:(void (^)(NSError *error, NSString *threadId))completion;

- (void)verifierMarkPresentationRequestMessageSent:(NSUInteger)proofHandle
                                        completion:(void (^)(NSError *error, NSUInteger state, NSString *message))completion;

- (void)verifierProofSerialize:(NSUInteger)proofHandle
                    completion:(void (^)(NSError *error, NSString *serializedProof))completion;

- (void)verifierProofDeserialize:(NSString *)serializedProof
                      completion:(void (^)(NSError *error, NSUInteger proofHandle))completion;

- (int)verifierProofRelease:(NSUInteger)proofHandle;

- (void)proofGetRequests:(NSUInteger)connectionHandle
              completion:(void (^)(NSError *error, NSString *requests))completion;

- (void)proofGetProofRequestAttachment:(NSUInteger)proofHandle
                            completion:(void (^)(NSError *error, NSString *attach))completion;

- (void)proofRetrieveCredentials:(NSUInteger)proofHandle
                  withCompletion:(void (^)(NSError *error, NSString *matchingCredentials))completion;

- (void)  proofGenerate:(NSUInteger)proofHandle
withSelectedCredentials:(NSString *)selectedCredentials
  withSelfAttestedAttrs:(NSString *)selfAttestedAttributes
         withCompletion:(void (^)(NSError *error))completion;

- (void)proofCreateWithMsgId:(NSString *)source_id
        withConnectionHandle:(NSUInteger)connectionHandle
                   withMsgId:(NSString *)msgId
              withCompletion:(void (^)(NSError *, NSUInteger, NSString *))completion;

- (void)   proofSend:(NSUInteger)proof_handle
withConnectionHandle:(NSUInteger)connection_handle
      withCompletion:(void (^)(NSError *error))completion;

- (void)proofGetState:(NSUInteger)proofHandle
           completion:(void (^)(NSError *error, NSUInteger state))completion;

- (void)proofUpdateStateV2:(NSUInteger)proofHandle
          connectionHandle:(NSUInteger)connectionHandle
                completion:(void (^)(NSError *error, NSUInteger state))completion;

- (void)proofUpdateStateWithMessageV2:(NSUInteger)proofHandle
                     connectionHandle:(NSUInteger)connectionHandle
                              message:(NSString *)message
                           completion:(void (^)(NSError *error, NSUInteger state))completion;

- (void) proofReject:(NSUInteger)proof_handle
withConnectionHandle:(NSUInteger)connection_handle
      withCompletion:(void (^)(NSError *error))completion;

- (void)proofDeclinePresentationRequest:(NSUInteger)proof_handle
                       connectionHandle:(NSUInteger)connection_handle
                                 reason:(NSString *)reason
                               proposal:(NSString *)proposal
                             completion:(void (^)(NSError *error))completion;

- (void)proofGetThreadId:(NSUInteger)proofHandle
          withCompletion:(void (^)(NSError *error, NSString *threadId))completion;

- (void)getProofMsg:(NSUInteger)proofHandle
     withCompletion:(void (^)(NSError *error, NSString *proofMsg))completion;

- (void)getRejectMsg:(NSUInteger)proofHandle
      withCompletion:(void (^)(NSError *error, NSString *rejectMsg))completion;

- (void)proofCreateWithRequest:(NSString *)source_id
              withProofRequest:(NSString *)proofRequest
                withCompletion:(void (^)(NSError *, NSUInteger))completion;

- (void)proofSerialize:(NSUInteger)proofHandle
        withCompletion:(void (^)(NSError *error, NSString *proof_request))completion;

- (void)proofDeserialize:(NSString *)serializedProof
          withCompletion:(void (^)(NSError *, NSUInteger))completion;

- (int)proofRelease:(NSUInteger)proofHandle;

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
