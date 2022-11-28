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

- (vcx_error_t)vcxPoolSetHandle:(NSNumber *)handle
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
                  seqNo:(NSNumber *)seqNo
             completion:(void (^)(NSError *error, NSString *verKey))completion;

- (vcx_error_t)vcxInitThreadPool:(NSString *)config;

- (void)createWallet:(NSString *)config
          completion:(void (^)(NSError *error))completion;

- (void)vcxConfigureIssuerWallet:(NSString *)seed
                      completion:(void (^)(NSError *error, NSString *conf))completion;

- (void)openMainWallet:(NSString *)config
            completion:(void (^)(NSError *error, NSNumber * handle))completion;

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

- (NSString *)errorCMessage:(NSNumber *)errorCode;

- (NSString *)vcxVersion;

- (void)vcxSchemaSerialize:(NSNumber *)schemaHandle
                completion:(void (^)(NSError *error, NSString *serializedSchema))completion;

- (void)vcxSchemaDeserialize:(NSString *)serializedSchema
                  completion:(void (^)(NSError *error, NSNumber * schemaHandle))completion;

- (void)vcxSchemaGetAttributes:(NSString *)sourceId
                    schemaId:(NSString *)schemaId
                    completion:(void (^)(NSError *error, NSNumber * schemaHandle, NSString *schemaAttributes))completion;

- (void)vcxSchemaCreate:(NSString *)sourceId
             schemaName:(NSString *)schemaName
          schemaVersion:(NSString *)schemaVersion
             schemaData:(NSString *)schemaData
          paymentHandle:(NSNumber *)paymentHandle
             completion:(void (^)(NSError *error, NSNumber * credDefHandle))completion;

- (void)vcxSchemaPrepareForEndorser:(NSString *)sourceId
                         schemaName:(NSString *)schemaName
                      schemaVersion:(NSString *)schemaVersion
                         schemaData:(NSString *)schemaData
                           endorser:(NSString *)endorser
                         completion:(void (^)(
                                 NSError *error,
                                 NSNumber * schemaHandle,
                                 NSString *schemaTransaction
                         ))completion;

- (void)vcxSchemaGetSchemaId:(NSString *)sourceId
                schemaHandle:(NSNumber *)schemaHandle
                  completion:(void (^)(NSError *error, NSString *schemaId))completion;

- (void)vcxSchemaUpdateState:(NSString *)sourceId
                schemaHandle:(NSNumber *)schemaHandle
                  completion:(void (^)(NSError *error, NSNumber * state))completion;

- (int)vcxSchemaRelease:(NSNumber *)schemaHandle;

- (void)vcxPublicAgentCreate:(NSString *)sourceId
              institutionDid:(NSString *)institutionDid
                  completion:(void (^)(
                          NSError *error,
                          NSNumber * agentHandle
                  ))completion;

- (void)vcxGeneratePublicInvite:(NSString *)publicDid
                          label:(NSString *)label
                     completion:(void (^)(
                             NSError *error,
                             NSString *publicInvite
                     ))completion;

- (void)vcxPublicAgentDownloadConnectionRequests:(NSNumber *)agentHandle
                                            uids:(NSString *)ids
                                      completion:(void (^)(
                                              NSError *error,
                                              NSString *requests
                                      ))completion;

- (void)vcxPublicAgentDownloadMessage:(NSNumber *)agentHandle
                                  uid:(NSString *)id
                           completion:(void (^)(NSError *error, NSString *message))completion;

- (void)vcxPublicAgentGetService:(NSNumber *)agentHandle
                      completion:(void (^)(NSError *error, NSString *service))completion;

- (void)vcxPublicAgentSerialize:(NSNumber *)agentHandle
                     completion:(void (^)(NSError *error, NSString *serializedAgent))completion;

- (int)vcxPublicAgentRelease:(NSNumber *)agentHandle;

- (void)vcxOutOfBandSenderCreate:(NSString *)config
                      completion:(void (^)(NSError *error, NSNumber * oobHandle))completion;

- (void)vcxOutOfBandReceiverCreate:(NSString *)message
                        completion:(void (^)(NSError *error, NSNumber * oobHandle))completion;

- (void)vcxOutOfBandSenderAppendMessage:(NSNumber *)oobHandle
                                message:(NSString *)message
                             completion:(void (^)(NSError *error))completion;

- (void)vcxOutOfBandSenderAppendService:(NSNumber *)oobHandle
                                service:(NSString *)service
                             completion:(void (^)(NSError *error))completion;

- (void)vcxOutOfBandSenderAppendServiceDid:(NSNumber *)oobHandle
                                       did:(NSString *)did
                                completion:(void (^)(NSError *error))completion;

- (void)vcxOutOfBandSenderGetThreadId:(NSNumber *)oobHandle
                           completion:(void (^)(NSError *error, NSString *threadId))completion;

- (void)vcxOutOfBandReceiverGetThreadId:(NSNumber *)oobHandle
                             completion:(void (^)(NSError *error, NSString *threadId))completion;

- (void)vcxOutOfBandReceiverExtractMessage:(NSNumber *)oobHandle
                                completion:(void (^)(NSError *error, NSString *message))completion;

- (void)vcxOutOfBandToMessage:(NSNumber *)oobHandle
                   completion:(void (^)(NSError *error, NSString *message))completion;

- (void)vcxOutOfBandSenderSerialize:(NSNumber *)oobHandle
                         completion:(void (^)(NSError *error, NSString *oobMessage))completion;

- (void)vcxOutOfBandReceiverSerialize:(NSNumber *)oobHandle
                           completion:(void (^)(NSError *error, NSString *oobMessage))completion;

- (void)vcxOutOfBandSenderDeserialize:(NSString *)oobMessage
                           completion:(void (^)(NSError *error, NSNumber * oobHandle))completion;

- (void)vcxOutOfBandReceiverDeserialize:(NSString *)oobMessage
                             completion:(void (^)(NSError *error, NSNumber * oobHandle))completion;

- (int)vcxOutOfBandSenderRelease:(NSNumber *)agentHandle;

- (int)vcxOutOfBandReceiverRelease:(NSNumber *)agentHandle;

- (void)vcxOutOfBandReceiverConnectionExists:(NSNumber *)oobHandle
                           connectionHandles:(NSString *)connectionHandles
                                  completion:(void (^)(
                                          NSError *error,
                                          NSNumber * connectionHandle,
                                          Boolean foundOne)
                                  )completion;

- (void)vcxOutOfBandReceiverBuildConnection:(NSNumber *)oobHandle
                                 completion:(void (^)(
                                         NSError *error,
                                         NSString *connection)
                                 )completion;

- (void)vcxRevocationRegistryCreate:(NSString *)revRegConfig
                         completion:(void (^)(
                                 NSError *error,
                                 NSNumber * revRegHandle)
                         )completion;

- (void)vcxRevocationRegistryPublish:(NSNumber *)revRegHandle
                            tailsUrl:(NSString *)tailsUrl
                          completion:(void (^)(
                                  NSError *error,
                                  NSNumber * handle)
                          )completion;

- (void)vcxRevocationRegistryPublishRevocations:(NSNumber *)revRegHandle
                                     completion:(void (^)(
                                             NSError *error)
                                     )completion;

- (void)vcxRevocationRegistryGetRevRegId:(NSNumber *)revRegHandle
                              completion:(void (^)(
                                      NSError *error,
                                      NSString *revRegId)
                              )completion;

- (void)vcxRevocationRegistryGetTailsHash:(NSNumber *)revRegHandle
                               completion:(void (^)(
                                       NSError *error,
                                       NSString *tailsHash)
                               )completion;

- (void)vcxRevocationRegistryDeserialize:(NSString *)serializedRevReg
                              completion:(void (^)(
                                      NSError *error,
                                      NSNumber * revRegHandle)
                              )completion;

- (void)vcxRevocationRegistrySerialize:(NSNumber *)revRegHandle
                            completion:(void (^)(
                                    NSError *error,
                                    NSString *serializedRevReg)
                            )completion;

- (int)vcxRevocationRegistryRelease:(NSNumber *)revRegHandle;

- (void)vcxCredentialDefinitionCreateV2:(NSString *)sourceId
                               schemaId:(NSString *)schemaId
                              issuerDid:(NSString *)issuerDid
                                    tag:(NSString *)tag
                      supportRevocation:(Boolean)supportRevocation
                             completion:(void (^)(
                                     NSError *error,
                                     NSNumber * credDefHandle)
                             )completion;

- (void)vcxCredentialDefinitionPublish:(NSNumber *)credDefHandle
                              tailsUrl:(NSString *)tailsUrl
                            completion:(void (^)(NSError *error))completion;

- (void)vcxCredentialDefinitionDeserialize:(NSString *)serializedCredDef
                                completion:(void (^)(NSError *error, NSNumber * credDefHandle))completion;

- (void)vcxCredentialDefinitionSerialize:(NSNumber *)credDefHandle
                              completion:(void (^)(NSError *error, NSString *serializedCredDef))completion;

- (int)vcxCredentialDefinitionRelease:(NSNumber *)credDefHandle;

- (void)vcxCredentialDefinitionGetCredDefId:(NSNumber *)credDefHandle
                                 completion:(void (^)(NSError *error, NSString *credDefId))completion;

- (void)vcxCredentialDefinitionUpdateState:(NSNumber *)credDefHandle
                                completion:(void (^)(NSError *error, NSNumber * state))completion;

- (void)vcxCredentialDefinitionGetState:(NSNumber *)credDefHandle
                             completion:(void (^)(NSError *error, NSNumber * state))completion;


- (void)connectionCreate:(NSString *)sourceId
              completion:(void (^)(NSError *error, NSNumber * connectionHandle))completion;

- (void)connectionCreateWithInvite:(NSString *)sourceId
                     inviteDetails:(NSString *)inviteDetails
                        completion:(void (^)(NSError *error, NSNumber * connectionHandle))completion;

- (void)connectionCreateWithConnectionRequestV2:(NSString *)sourceId
                                    agentHandle:(NSNumber *)agentHandle
                                        request:(NSString *)request
                                     completion:(void (^)(NSError *error, NSNumber * connectionHandle))completion;

- (void)connectionConnect:(NSNumber *)connectionHandle
           connectionType:(NSString *)connectionType
               completion:(void (^)(NSError *error, NSString *inviteDetails))completion;

- (void)connectionGetState:(NSNumber *)connectionHandle
                completion:(void (^)(NSError *error, NSNumber * state))completion;

- (void)connectionUpdateState:(NSNumber *)connectionHandle
                   completion:(void (^)(NSError *error, NSNumber * state))completion;

- (void)connectionUpdateStateWithMessage:(NSNumber *)connectionHandle
                                 message:(NSString *)message
                              completion:(void (^)(NSError *error, NSNumber * state))completion;

- (void)connectionHandleMessage:(NSNumber *)connectionHandle
                        message:(NSString *)message
                     completion:(void (^)(NSError *error))completion;

- (void)connectionSerialize:(NSNumber *)connectionHandle
                 completion:(void (^)(NSError *error, NSString *serializedConnection))completion;

- (void)connectionDeserialize:(NSString *)serializedConnection
                   completion:(void (^)(NSError *error, NSNumber * connectionHandle))completion;

- (int)connectionRelease:(NSNumber *)connectionHandle;

- (void)connectionInviteDetails:(NSNumber *)connectionHandle
                     completion:(void (^)(NSError *error, NSString *inviteDetails))completion;

- (void)deleteConnection:(NSNumber *)connectionHandle
          withCompletion:(void (^)(NSError *error))completion;

- (void)connectionGetPwDid:(NSNumber *)connectionHandle
                completion:(void (^)(NSError *error, NSString *pwDid))completion;

- (void)connectionGetTheirPwDid:(NSNumber *)connectionHandle
                     completion:(void (^)(NSError *error, NSString *theirPwDid))completion;

- (void)connectionInfo:(NSNumber *)connectionHandle
            completion:(void (^)(NSError *error, NSString *info))completion;

- (void)connectionGetThreadId:(NSNumber *)connectionHandle
                   completion:(void (^)(NSError *error, NSString *threadId))completion;

- (void)connectionSendMessage:(NSNumber *)connectionHandle
                  withMessage:(NSString *)message
       withSendMessageOptions:(NSString *)sendMessageOptions
               withCompletion:(void (^)(NSError *error, NSString *msg_id))completion;

- (void)connectionSignData:(NSNumber *)connectionHandle
                  withData:(NSData *)dataRaw
            withCompletion:(void (^)(NSError *error, NSData *signature_raw, vcx_u32_t signature_len))completion;

- (void)connectionVerifySignature:(NSNumber *)connectionHandle
                         withData:(NSData *)dataRaw
                withSignatureData:(NSData *)signatureRaw
                   withCompletion:(void (^)(NSError *error, vcx_bool_t valid))completion;

- (void)connectionSendPing:(NSNumber *)connectionHandle
                   comment:(NSString *)comment
                completion:(void (^)(NSError *error))completion;

- (void)connectionSendDiscoveryFeatures:(NSNumber *)connectionHandle
                                  query:(NSString *)query
                                comment:(NSString *)comment
                             completion:(void (^)(NSError *error))completion;

- (void)connectionDownloadMessages:(NSNumber *)connectionHandle
                     messageStatus:(NSString *)messageStatus
                             uid_s:(NSString *)uid_s
                        completion:(void (^)(NSError *error, NSString *messages))completion;

- (void)connectionSendHandshakeReuse:(NSNumber *)connectionHandle
                              oobMsg:(NSString *)oobMsg
                          completion:(void (^)(NSError *error))completion;

- (void)issuerCreateCredential:(NSString *)sourceId
                    completion:(void (^)(NSError *error, NSNumber * credentialHandle))completion;

- (void)issuerRevokeCredentialLocal:(NSNumber *)credentialHandle
                         completion:(void (^)(NSError *error))completion;

- (void)issuerCredentialIsRevokable:(NSNumber *)credentialHandle
                         completion:(void (^)(NSError *error, Boolean isRevokable))completion;

- (void)issuerSendCredentialOfferV2:(NSNumber *)credentialHandle
                   connectionHandle:(NSNumber *)connectionHandle
                         completion:(void (^)(NSError *error))completion;

- (void)markCredentialOfferSent:(NSNumber *)credentialHandle
                     completion:(void (^)(NSError *error, NSString *message))completion;

- (void)issuerBuildCredentialOfferMessageV2:(NSNumber *)credDefHandle
                               revRegHandle:(NSNumber *)revRegHandle
                             credentialData:(NSString *)credData
                                    comment:(NSString *)comment
                                 completion:(void (^)(NSError *error, NSString *message))completion;

- (void)issuerGetCredentialOfferMessage:(NSNumber *)credentialHandle
                             completion:(void (^)(NSError *error, NSString *message))completion;

- (void)issuerGetCredentialMessage:(NSNumber *)credentialHandle
                     myPairwiseDid:(NSString *)myPwDid
                        completion:(void (^)(NSError *error, NSString *message))completion;

- (void)issuerCredentialGetState:(NSNumber *)credentialHandle
                      completion:(void (^)(NSError *error, NSNumber * state))completion;

- (void)issuerCredentialGetRevRegId:(NSNumber *)credentialHandle
                         completion:(void (^)(NSError *error, NSString *revRegId))completion;

- (void)issuerSendCredential:(NSNumber *)credentialHandle
            connectionHandle:(NSNumber *)connectionHandle
                  completion:(void (^)(NSError *error, NSNumber * state))completion;

- (void)issuerCredentialSerialize:(NSNumber *)credentialHandle
                       completion:(void (^)(NSError *error, NSString *serializedCredential))completion;

- (void)issuerCredentialDeserialize:(NSString *)serializedCredential
                         completion:(void (^)(NSError *error, NSNumber * credentialHandle))completion;

- (void)issuerCredentialUpdateStateV2:(NSNumber *)credentialHandle
                     connectionHandle:(NSNumber *)connectionHandle
                           completion:(void (^)(NSError *error, NSNumber * state))completion;

- (void)issuerCredentialUpdateStateWithMessageV2:(NSNumber *)credentialHandle
                                connectionHandle:(NSNumber *)connectionHandle
                                         message:(NSString *)message
                                      completion:(void (^)(NSError *error, NSNumber * state))completion;

- (void)issuerCredentialGetThreadId:(NSNumber *)credentialHandle
                         completion:(void (^)(NSError *error, NSString *threadId))completion;

- (int)issuerCredentialRelease:(NSNumber *)credentialHandle;

- (void)getCredential:(NSNumber *)credentialHandle
           completion:(void (^)(NSError *error, NSString *credential))completion;

- (void)credentialCreateWithOffer:(NSString *)sourceId
                            offer:(NSString *)credentialOffer
                       completion:(void (^)(NSError *error, NSNumber * credentialHandle))completion;

- (void)credentialCreateWithMsgid:(NSString *)sourceId
                 connectionHandle:(NSNumber *)connectionHandle
                            msgId:(NSString *)msgId
                       completion:(void (^)(NSError *error, NSNumber * credentialHandle, NSString *credentialOffer))completion;

- (void)credentialSendRequest:(NSNumber *)credentialHandle
             connectionHandle:(NSNumber *)connectionHandle
                paymentHandle:(NSNumber *)paymentHandle
                   completion:(void (^)(NSError *error))completion;

- (void)credentialGetRequestMessage:(NSNumber *)credentialHandle
                      myPairwiseDid:(NSString *)myPwDid
                  theirdPairwiseDid:(NSString *)theirPwDid
                      paymentHandle:(NSNumber *)paymentHandle
                         completion:(void (^)(NSError *error, NSString *message))completion;

- (void)credentialDeclineOffer:(NSNumber *)credentialHandle
              connectionHandle:(NSNumber *)connectionHandle
                       comment:(NSString *)comment
                    completion:(void (^)(NSError *error))completion;

- (void)credentialGetState:(NSNumber *)credentialHandle
                completion:(void (^)(NSError *error, NSNumber * state))completion;

- (void)credentialUpdateStateV2:(NSNumber *)credentailHandle
               connectionHandle:(NSNumber *)connectionHandle
                     completion:(void (^)(NSError *error, NSNumber * state))completion;

- (void)credentialUpdateStateWithMessageV2:(NSNumber *)credentialHandle
                          connectionHandle:(NSNumber *)connectionHandle
                                   message:(NSString *)message
                                completion:(void (^)(NSError *error, NSNumber * state))completion;

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
                   completion:(void (^)(NSError *error, vcx_bool_t revokable))completion;

- (void)credentialSerialize:(NSNumber *)credentialHandle
                 completion:(void (^)(NSError *error, NSString *state))completion;

- (void)credentialDeserialize:(NSString *)serializedCredential
                   completion:(void (^)(NSError *error, NSNumber * credentialHandle))completion;

- (int)credentialRelease:(NSNumber *)credentialHandle;

- (void)deleteCredential:(NSNumber *)credentialHandle
              completion:(void (^)(NSError *error))completion;

- (int)walletSetHandle:(NSNumber *)handle;

- (void)exportWallet:(NSString *)exportPath
         encryptWith:(NSString *)encryptionKey
          completion:(void (^)(NSError *error, NSNumber * exportHandle))completion;

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
              completion:(void (^)(NSError *error, NSNumber * searchHandle))completion;

- (void)searchNextRecordsWallet:(NSNumber *)searchHandle
                          count:(int)count
                     completion:(void (^)(NSError *error, NSString *records))completion;

- (void)closeSearchWallet:(NSNumber *)searchHandle
               completion:(void (^)(NSError *error))completion;

- (void)verifierProofCreate:(NSString *)proofRequestId
             requestedAttrs:(NSString *)requestedAttrs
        requestedPredicates:(NSString *)requestedPredicates
         revocationInterval:(NSString *)revocationInterval
                  proofName:(NSString *)proofName
                 completion:(void (^)(NSError *error, NSNumber * proofHandle))completion;

- (void)verifierProofSendRequest:(NSNumber *)proofHandle
                connectionHandle:(NSNumber *)connectionHandle
                      completion:(void (^)(NSError *error))completion;

- (void)verifierGetProofMessage:(NSNumber *)proofHandle
                     completion:(void (^)(NSError *error, NSNumber * state, NSString *responseData))completion;

- (void)verifierProofGetRequestMessage:(NSNumber *)proofHandle
                            completion:(void (^)(NSError *error, NSString *message))completion;

- (void)verifierProofUpdateStateV2:(NSNumber *)proofHandle
                  connectionHandle:(NSNumber *)connectionHandle
                        completion:(void (^)(NSError *error, NSNumber * state))completion;

- (void)verifierProofUpdateStateWithMessageV2:(NSNumber *)proofHandle
                             connectionHandle:(NSNumber *)connectionHandle
                                      message:(NSString *)message
                                   completion:(void (^)(NSError *error, NSNumber * state))completion;

- (void)verifierProofGetState:(NSNumber *)proofHandle
                   completion:(void (^)(NSError *error, NSNumber * state))completion;

- (void)verifierProofGetThreadId:(NSNumber *)proofHandle
                      completion:(void (^)(NSError *error, NSString *threadId))completion;

- (void)verifierMarkPresentationRequestMessageSent:(NSNumber *)proofHandle
                                        completion:(void (^)(NSError *error, NSNumber * state, NSString *message))completion;

- (void)verifierProofSerialize:(NSNumber *)proofHandle
                    completion:(void (^)(NSError *error, NSString *serializedProof))completion;

- (void)verifierProofDeserialize:(NSString *)serializedProof
                      completion:(void (^)(NSError *error, NSNumber * proofHandle))completion;

- (int)verifierProofRelease:(NSNumber *)proofHandle;

- (void)proofGetRequests:(NSNumber *)connectionHandle
              completion:(void (^)(NSError *error, NSString *requests))completion;

- (void)proofGetProofRequestAttachment:(NSNumber *)proofHandle
                            completion:(void (^)(NSError *error, NSString *attach))completion;

- (void)proofRetrieveCredentials:(NSNumber *)proofHandle
                  withCompletion:(void (^)(NSError *error, NSString *matchingCredentials))completion;

- (void)  proofGenerate:(NSNumber *)proofHandle
withSelectedCredentials:(NSString *)selectedCredentials
  withSelfAttestedAttrs:(NSString *)selfAttestedAttributes
         withCompletion:(void (^)(NSError *error))completion;

- (void)proofCreateWithMsgId:(NSString *)source_id
        withConnectionHandle:(NSNumber *)connectionHandle
                   withMsgId:(NSString *)msgId
              withCompletion:(void (^)(NSError *, NSNumber *, NSString *))completion;

- (void)   proofSend:(NSNumber *)proof_handle
withConnectionHandle:(NSNumber *)connection_handle
      withCompletion:(void (^)(NSError *error))completion;

- (void)proofGetState:(NSNumber *)proofHandle
           completion:(void (^)(NSError *error, NSNumber * state))completion;

- (void)proofUpdateStateV2:(NSNumber *)proofHandle
          connectionHandle:(NSNumber *)connectionHandle
                completion:(void (^)(NSError *error, NSNumber * state))completion;

- (void)proofUpdateStateWithMessageV2:(NSNumber *)proofHandle
                     connectionHandle:(NSNumber *)connectionHandle
                              message:(NSString *)message
                           completion:(void (^)(NSError *error, NSNumber * state))completion;

- (void) proofReject:(NSNumber *)proof_handle
withConnectionHandle:(NSNumber *)connection_handle
      withCompletion:(void (^)(NSError *error))completion;

- (void)proofDeclinePresentationRequest:(NSNumber *)proof_handle
                       connectionHandle:(NSNumber *)connection_handle
                                 reason:(NSString *)reason
                               proposal:(NSString *)proposal
                             completion:(void (^)(NSError *error))completion;

- (void)proofGetThreadId:(NSNumber *)proofHandle
          withCompletion:(void (^)(NSError *error, NSString *threadId))completion;

- (void)getProofMsg:(NSNumber *)proofHandle
     withCompletion:(void (^)(NSError *error, NSString *proofMsg))completion;

- (void)getRejectMsg:(NSNumber *)proofHandle
      withCompletion:(void (^)(NSError *error, NSString *rejectMsg))completion;

- (void)proofCreateWithRequest:(NSString *)source_id
              withProofRequest:(NSString *)proofRequest
                withCompletion:(void (^)(NSError *, NSNumber *))completion;

- (void)proofSerialize:(NSNumber *)proofHandle
        withCompletion:(void (^)(NSError *error, NSString *proof_request))completion;

- (void)proofDeserialize:(NSString *)serializedProof
          withCompletion:(void (^)(NSError *, NSNumber *))completion;

- (int)proofRelease:(NSNumber *)proofHandle;

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
