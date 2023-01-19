const { existsSync, readFileSync } = require('fs')
const { join } = require('path')

const { platform, arch } = process

let nativeBinding = null
let localFileExisted = false
let loadError = null

function isMusl() {
  // For Node 10
  if (!process.report || typeof process.report.getReport !== 'function') {
    try {
      const lddPath = require('child_process').execSync('which ldd').toString().trim();
      return readFileSync(lddPath, 'utf8').includes('musl')
    } catch (e) {
      return true
    }
  } else {
    const { glibcVersionRuntime } = process.report.getReport().header
    return !glibcVersionRuntime
  }
}

switch (platform) {
  case 'android':
    switch (arch) {
      case 'arm64':
        localFileExisted = existsSync(join(__dirname, 'vcx-napi-rs.android-arm64.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./vcx-napi-rs.android-arm64.node')
          } else {
            nativeBinding = require('@hyperledger/vcx-napi-rs-android-arm64')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'arm':
        localFileExisted = existsSync(join(__dirname, 'vcx-napi-rs.android-arm-eabi.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./vcx-napi-rs.android-arm-eabi.node')
          } else {
            nativeBinding = require('@hyperledger/vcx-napi-rs-android-arm-eabi')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on Android ${arch}`)
    }
    break
  case 'win32':
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync(
          join(__dirname, 'vcx-napi-rs.win32-x64-msvc.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./vcx-napi-rs.win32-x64-msvc.node')
          } else {
            nativeBinding = require('@hyperledger/vcx-napi-rs-win32-x64-msvc')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'ia32':
        localFileExisted = existsSync(
          join(__dirname, 'vcx-napi-rs.win32-ia32-msvc.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./vcx-napi-rs.win32-ia32-msvc.node')
          } else {
            nativeBinding = require('@hyperledger/vcx-napi-rs-win32-ia32-msvc')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'arm64':
        localFileExisted = existsSync(
          join(__dirname, 'vcx-napi-rs.win32-arm64-msvc.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./vcx-napi-rs.win32-arm64-msvc.node')
          } else {
            nativeBinding = require('@hyperledger/vcx-napi-rs-win32-arm64-msvc')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on Windows: ${arch}`)
    }
    break
  case 'darwin':
    localFileExisted = existsSync(join(__dirname, 'vcx-napi-rs.darwin-universal.node'))
    try {
      if (localFileExisted) {
        nativeBinding = require('./vcx-napi-rs.darwin-universal.node')
      } else {
        nativeBinding = require('@hyperledger/vcx-napi-rs-darwin-universal')
      }
      break
    } catch {}
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync(join(__dirname, 'vcx-napi-rs.darwin-x64.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./vcx-napi-rs.darwin-x64.node')
          } else {
            nativeBinding = require('@hyperledger/vcx-napi-rs-darwin-x64')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'arm64':
        localFileExisted = existsSync(
          join(__dirname, 'vcx-napi-rs.darwin-arm64.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./vcx-napi-rs.darwin-arm64.node')
          } else {
            nativeBinding = require('@hyperledger/vcx-napi-rs-darwin-arm64')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on macOS: ${arch}`)
    }
    break
  case 'freebsd':
    if (arch !== 'x64') {
      throw new Error(`Unsupported architecture on FreeBSD: ${arch}`)
    }
    localFileExisted = existsSync(join(__dirname, 'vcx-napi-rs.freebsd-x64.node'))
    try {
      if (localFileExisted) {
        nativeBinding = require('./vcx-napi-rs.freebsd-x64.node')
      } else {
        nativeBinding = require('@hyperledger/vcx-napi-rs-freebsd-x64')
      }
    } catch (e) {
      loadError = e
    }
    break
  case 'linux':
    switch (arch) {
      case 'x64':
        if (isMusl()) {
          localFileExisted = existsSync(
            join(__dirname, 'vcx-napi-rs.linux-x64-musl.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./vcx-napi-rs.linux-x64-musl.node')
            } else {
              nativeBinding = require('@hyperledger/vcx-napi-rs-linux-x64-musl')
            }
          } catch (e) {
            loadError = e
          }
        } else {
          localFileExisted = existsSync(
            join(__dirname, 'vcx-napi-rs.linux-x64-gnu.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./vcx-napi-rs.linux-x64-gnu.node')
            } else {
              nativeBinding = require('@hyperledger/vcx-napi-rs-linux-x64-gnu')
            }
          } catch (e) {
            loadError = e
          }
        }
        break
      case 'arm64':
        if (isMusl()) {
          localFileExisted = existsSync(
            join(__dirname, 'vcx-napi-rs.linux-arm64-musl.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./vcx-napi-rs.linux-arm64-musl.node')
            } else {
              nativeBinding = require('@hyperledger/vcx-napi-rs-linux-arm64-musl')
            }
          } catch (e) {
            loadError = e
          }
        } else {
          localFileExisted = existsSync(
            join(__dirname, 'vcx-napi-rs.linux-arm64-gnu.node')
          )
          try {
            if (localFileExisted) {
              nativeBinding = require('./vcx-napi-rs.linux-arm64-gnu.node')
            } else {
              nativeBinding = require('@hyperledger/vcx-napi-rs-linux-arm64-gnu')
            }
          } catch (e) {
            loadError = e
          }
        }
        break
      case 'arm':
        localFileExisted = existsSync(
          join(__dirname, 'vcx-napi-rs.linux-arm-gnueabihf.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./vcx-napi-rs.linux-arm-gnueabihf.node')
          } else {
            nativeBinding = require('@hyperledger/vcx-napi-rs-linux-arm-gnueabihf')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on Linux: ${arch}`)
    }
    break
  default:
    throw new Error(`Unsupported OS: ${platform}, architecture: ${arch}`)
}

if (!nativeBinding) {
  if (loadError) {
    throw loadError
  }
  throw new Error(`Failed to load native binding`)
}

const { updateWebhookUrl, createAgencyClientForMainWallet, provisionCloudAgent, messagesUpdateStatus, generatePublicInvitation, connectionCreateInviter, connectionCreateInvitee, connectionGetThreadId, connectionGetPairwiseInfo, connectionGetRemoteDid, connectionGetState, connectionGetInvitation, connectionProcessInvite, connectionProcessRequest, connectionProcessResponse, connectionProcessAck, connectionSendResponse, connectionSendRequest, connectionSendAck, connectionCreateInvite, connectionSerialize, connectionDeserialize, connectionRelease, credentialCreateWithOffer, credentialRelease, credentialSendRequest, credentialDeclineOffer, credentialSerialize, credentialDeserialize, v2CredentialUpdateStateWithMessage, v2CredentialUpdateState, credentialGetState, credentialGetOffers, credentialGetAttributes, credentialGetAttachment, credentialGetTailsLocation, credentialGetTailsHash, credentialGetRevRegId, credentialGetThreadId, credentialdefCreateV2, credentialdefPublish, credentialdefDeserialize, credentialdefRelease, credentialdefSerialize, credentialdefGetCredDefId, credentialdefUpdateState, credentialdefGetState, disclosedProofCreateWithRequest, disclosedProofRelease, disclosedProofSendProof, disclosedProofRejectProof, disclosedProofGetProofMsg, disclosedProofSerialize, disclosedProofDeserialize, v2DisclosedProofUpdateState, v2DisclosedProofUpdateStateWithMessage, disclosedProofGetState, disclosedProofGetRequests, disclosedProofRetrieveCredentials, disclosedProofGetProofRequestAttachment, disclosedProofGenerateProof, disclosedProofDeclinePresentationRequest, disclosedProofGetThreadId, issuerCredentialDeserialize, issuerCredentialSerialize, issuerCredentialUpdateStateV2, issuerCredentialUpdateStateWithMessageV2, issuerCredentialGetState, issuerCredentialGetRevRegId, issuerCredentialCreate, issuerCredentialRevokeLocal, issuerCredentialIsRevokable, issuerCredentialSendCredential, issuerCredentialSendOfferV2, issuerCredentialMarkOfferMsgSent, issuerCredentialBuildOfferMsgV2, issuerCredentialGetOfferMsg, issuerCredentialRelease, issuerCredentialGetThreadId, getLedgerAuthorAgreement, setActiveTxnAuthorAgreementMeta, createService, getServiceFromLedger, getVerkeyFromLedger, getLedgerTxn, initDefaultLogger, mediatedConnectionGeneratePublicInvite, mediatedConnectionGetPwDid, mediatedConnectionGetTheirPwDid, mediatedConnectionGetThreadId, mediatedConnectionGetState, mediatedConnectionGetSourceId, mediatedConnectionCreate, mediatedConnectionCreateWithInvite, mediatedConnectionSendMessage, mediatedConnectionCreateWithConnectionRequestV2, mediatedConnectionSendHandshakeReuse, mediatedConnectionUpdateStateWithMessage, mediatedConnectionHandleMessage, mediatedConnectionUpdateState, mediatedConnectionDeleteConnection, mediatedConnectionConnect, mediatedConnectionSerialize, mediatedConnectionDeserialize, mediatedConnectionRelease, mediatedConnectionInviteDetails, mediatedConnectionSendPing, mediatedConnectionSendDiscoveryFeatures, mediatedConnectionInfo, mediatedConnectionMessagesDownload, mediatedConnectionSignData, mediatedConnectionVerifySignature, outOfBandReceiverCreate, outOfBandReceiverExtractMessage, outOfBandReceiverConnectionExists, outOfBandReceiverBuildConnection, outOfBandReceiverGetThreadId, outOfBandReceiverSerialize, outOfBandReceiverDeserialize, outOfBandReceiverRelease, outOfBandSenderCreate, outOfBandSenderAppendMessage, outOfBandSenderAppendService, outOfBandSenderAppendServiceDid, outOfBandSenderToMessage, outOfBandSenderGetThreadId, outOfBandSenderSerialize, outOfBandSenderDeserialize, outOfBandSenderRelease, openMainPool, closeMainPool, proofCreate, proofGetProofMsg, proofRelease, proofSendRequest, proofGetRequestMsg, proofSerialize, proofDeserialize, v2ProofUpdateState, v2ProofUpdateStateWithMessage, proofGetState, proofGetProofState, proofGetThreadId, markPresentationRequestMsgSent, revocationRegistryCreate, revocationRegistryPublish, revocationRegistryPublishRevocations, revocationRegistryGetRevRegId, revocationRegistryGetTailsHash, revocationRegistrySerialize, revocationRegistryDeserialize, revocationRegistryRelease, schemaGetAttributes, schemaPrepareForEndorser, schemaCreate, schemaGetSchemaId, schemaDeserialize, schemaSerialize, schemaRelease, schemaUpdateState, schemaGetState, enableMocks, shutdown, getVersion, walletOpenAsMain, walletCreateMain, walletCloseMain, vcxInitIssuerConfig, configureIssuerWallet, unpack, createPairwiseInfo, walletImport, walletExport, getVerkeyFromWallet, rotateVerkey, rotateVerkeyStart, rotateVerkeyApply } = nativeBinding

module.exports.updateWebhookUrl = updateWebhookUrl
module.exports.createAgencyClientForMainWallet = createAgencyClientForMainWallet
module.exports.provisionCloudAgent = provisionCloudAgent
module.exports.messagesUpdateStatus = messagesUpdateStatus
module.exports.generatePublicInvitation = generatePublicInvitation
module.exports.connectionCreateInviter = connectionCreateInviter
module.exports.connectionCreateInvitee = connectionCreateInvitee
module.exports.connectionGetThreadId = connectionGetThreadId
module.exports.connectionGetPairwiseInfo = connectionGetPairwiseInfo
module.exports.connectionGetRemoteDid = connectionGetRemoteDid
module.exports.connectionGetState = connectionGetState
module.exports.connectionGetInvitation = connectionGetInvitation
module.exports.connectionProcessInvite = connectionProcessInvite
module.exports.connectionProcessRequest = connectionProcessRequest
module.exports.connectionProcessResponse = connectionProcessResponse
module.exports.connectionProcessAck = connectionProcessAck
module.exports.connectionSendResponse = connectionSendResponse
module.exports.connectionSendRequest = connectionSendRequest
module.exports.connectionSendAck = connectionSendAck
module.exports.connectionCreateInvite = connectionCreateInvite
module.exports.connectionSerialize = connectionSerialize
module.exports.connectionDeserialize = connectionDeserialize
module.exports.connectionRelease = connectionRelease
module.exports.credentialCreateWithOffer = credentialCreateWithOffer
module.exports.credentialRelease = credentialRelease
module.exports.credentialSendRequest = credentialSendRequest
module.exports.credentialDeclineOffer = credentialDeclineOffer
module.exports.credentialSerialize = credentialSerialize
module.exports.credentialDeserialize = credentialDeserialize
module.exports.v2CredentialUpdateStateWithMessage = v2CredentialUpdateStateWithMessage
module.exports.v2CredentialUpdateState = v2CredentialUpdateState
module.exports.credentialGetState = credentialGetState
module.exports.credentialGetOffers = credentialGetOffers
module.exports.credentialGetAttributes = credentialGetAttributes
module.exports.credentialGetAttachment = credentialGetAttachment
module.exports.credentialGetTailsLocation = credentialGetTailsLocation
module.exports.credentialGetTailsHash = credentialGetTailsHash
module.exports.credentialGetRevRegId = credentialGetRevRegId
module.exports.credentialGetThreadId = credentialGetThreadId
module.exports.credentialdefCreateV2 = credentialdefCreateV2
module.exports.credentialdefPublish = credentialdefPublish
module.exports.credentialdefDeserialize = credentialdefDeserialize
module.exports.credentialdefRelease = credentialdefRelease
module.exports.credentialdefSerialize = credentialdefSerialize
module.exports.credentialdefGetCredDefId = credentialdefGetCredDefId
module.exports.credentialdefUpdateState = credentialdefUpdateState
module.exports.credentialdefGetState = credentialdefGetState
module.exports.disclosedProofCreateWithRequest = disclosedProofCreateWithRequest
module.exports.disclosedProofRelease = disclosedProofRelease
module.exports.disclosedProofSendProof = disclosedProofSendProof
module.exports.disclosedProofRejectProof = disclosedProofRejectProof
module.exports.disclosedProofGetProofMsg = disclosedProofGetProofMsg
module.exports.disclosedProofSerialize = disclosedProofSerialize
module.exports.disclosedProofDeserialize = disclosedProofDeserialize
module.exports.v2DisclosedProofUpdateState = v2DisclosedProofUpdateState
module.exports.v2DisclosedProofUpdateStateWithMessage = v2DisclosedProofUpdateStateWithMessage
module.exports.disclosedProofGetState = disclosedProofGetState
module.exports.disclosedProofGetRequests = disclosedProofGetRequests
module.exports.disclosedProofRetrieveCredentials = disclosedProofRetrieveCredentials
module.exports.disclosedProofGetProofRequestAttachment = disclosedProofGetProofRequestAttachment
module.exports.disclosedProofGenerateProof = disclosedProofGenerateProof
module.exports.disclosedProofDeclinePresentationRequest = disclosedProofDeclinePresentationRequest
module.exports.disclosedProofGetThreadId = disclosedProofGetThreadId
module.exports.issuerCredentialDeserialize = issuerCredentialDeserialize
module.exports.issuerCredentialSerialize = issuerCredentialSerialize
module.exports.issuerCredentialUpdateStateV2 = issuerCredentialUpdateStateV2
module.exports.issuerCredentialUpdateStateWithMessageV2 = issuerCredentialUpdateStateWithMessageV2
module.exports.issuerCredentialGetState = issuerCredentialGetState
module.exports.issuerCredentialGetRevRegId = issuerCredentialGetRevRegId
module.exports.issuerCredentialCreate = issuerCredentialCreate
module.exports.issuerCredentialRevokeLocal = issuerCredentialRevokeLocal
module.exports.issuerCredentialIsRevokable = issuerCredentialIsRevokable
module.exports.issuerCredentialSendCredential = issuerCredentialSendCredential
module.exports.issuerCredentialSendOfferV2 = issuerCredentialSendOfferV2
module.exports.issuerCredentialMarkOfferMsgSent = issuerCredentialMarkOfferMsgSent
module.exports.issuerCredentialBuildOfferMsgV2 = issuerCredentialBuildOfferMsgV2
module.exports.issuerCredentialGetOfferMsg = issuerCredentialGetOfferMsg
module.exports.issuerCredentialRelease = issuerCredentialRelease
module.exports.issuerCredentialGetThreadId = issuerCredentialGetThreadId
module.exports.getLedgerAuthorAgreement = getLedgerAuthorAgreement
module.exports.setActiveTxnAuthorAgreementMeta = setActiveTxnAuthorAgreementMeta
module.exports.createService = createService
module.exports.getServiceFromLedger = getServiceFromLedger
module.exports.getVerkeyFromLedger = getVerkeyFromLedger
module.exports.getLedgerTxn = getLedgerTxn
module.exports.initDefaultLogger = initDefaultLogger
module.exports.mediatedConnectionGeneratePublicInvite = mediatedConnectionGeneratePublicInvite
module.exports.mediatedConnectionGetPwDid = mediatedConnectionGetPwDid
module.exports.mediatedConnectionGetTheirPwDid = mediatedConnectionGetTheirPwDid
module.exports.mediatedConnectionGetThreadId = mediatedConnectionGetThreadId
module.exports.mediatedConnectionGetState = mediatedConnectionGetState
module.exports.mediatedConnectionGetSourceId = mediatedConnectionGetSourceId
module.exports.mediatedConnectionCreate = mediatedConnectionCreate
module.exports.mediatedConnectionCreateWithInvite = mediatedConnectionCreateWithInvite
module.exports.mediatedConnectionSendMessage = mediatedConnectionSendMessage
module.exports.mediatedConnectionCreateWithConnectionRequestV2 = mediatedConnectionCreateWithConnectionRequestV2
module.exports.mediatedConnectionSendHandshakeReuse = mediatedConnectionSendHandshakeReuse
module.exports.mediatedConnectionUpdateStateWithMessage = mediatedConnectionUpdateStateWithMessage
module.exports.mediatedConnectionHandleMessage = mediatedConnectionHandleMessage
module.exports.mediatedConnectionUpdateState = mediatedConnectionUpdateState
module.exports.mediatedConnectionDeleteConnection = mediatedConnectionDeleteConnection
module.exports.mediatedConnectionConnect = mediatedConnectionConnect
module.exports.mediatedConnectionSerialize = mediatedConnectionSerialize
module.exports.mediatedConnectionDeserialize = mediatedConnectionDeserialize
module.exports.mediatedConnectionRelease = mediatedConnectionRelease
module.exports.mediatedConnectionInviteDetails = mediatedConnectionInviteDetails
module.exports.mediatedConnectionSendPing = mediatedConnectionSendPing
module.exports.mediatedConnectionSendDiscoveryFeatures = mediatedConnectionSendDiscoveryFeatures
module.exports.mediatedConnectionInfo = mediatedConnectionInfo
module.exports.mediatedConnectionMessagesDownload = mediatedConnectionMessagesDownload
module.exports.mediatedConnectionSignData = mediatedConnectionSignData
module.exports.mediatedConnectionVerifySignature = mediatedConnectionVerifySignature
module.exports.outOfBandReceiverCreate = outOfBandReceiverCreate
module.exports.outOfBandReceiverExtractMessage = outOfBandReceiverExtractMessage
module.exports.outOfBandReceiverConnectionExists = outOfBandReceiverConnectionExists
module.exports.outOfBandReceiverBuildConnection = outOfBandReceiverBuildConnection
module.exports.outOfBandReceiverGetThreadId = outOfBandReceiverGetThreadId
module.exports.outOfBandReceiverSerialize = outOfBandReceiverSerialize
module.exports.outOfBandReceiverDeserialize = outOfBandReceiverDeserialize
module.exports.outOfBandReceiverRelease = outOfBandReceiverRelease
module.exports.outOfBandSenderCreate = outOfBandSenderCreate
module.exports.outOfBandSenderAppendMessage = outOfBandSenderAppendMessage
module.exports.outOfBandSenderAppendService = outOfBandSenderAppendService
module.exports.outOfBandSenderAppendServiceDid = outOfBandSenderAppendServiceDid
module.exports.outOfBandSenderToMessage = outOfBandSenderToMessage
module.exports.outOfBandSenderGetThreadId = outOfBandSenderGetThreadId
module.exports.outOfBandSenderSerialize = outOfBandSenderSerialize
module.exports.outOfBandSenderDeserialize = outOfBandSenderDeserialize
module.exports.outOfBandSenderRelease = outOfBandSenderRelease
module.exports.openMainPool = openMainPool
module.exports.closeMainPool = closeMainPool
module.exports.proofCreate = proofCreate
module.exports.proofGetProofMsg = proofGetProofMsg
module.exports.proofRelease = proofRelease
module.exports.proofSendRequest = proofSendRequest
module.exports.proofGetRequestMsg = proofGetRequestMsg
module.exports.proofSerialize = proofSerialize
module.exports.proofDeserialize = proofDeserialize
module.exports.v2ProofUpdateState = v2ProofUpdateState
module.exports.v2ProofUpdateStateWithMessage = v2ProofUpdateStateWithMessage
module.exports.proofGetState = proofGetState
module.exports.proofGetProofState = proofGetProofState
module.exports.proofGetThreadId = proofGetThreadId
module.exports.markPresentationRequestMsgSent = markPresentationRequestMsgSent
module.exports.revocationRegistryCreate = revocationRegistryCreate
module.exports.revocationRegistryPublish = revocationRegistryPublish
module.exports.revocationRegistryPublishRevocations = revocationRegistryPublishRevocations
module.exports.revocationRegistryGetRevRegId = revocationRegistryGetRevRegId
module.exports.revocationRegistryGetTailsHash = revocationRegistryGetTailsHash
module.exports.revocationRegistrySerialize = revocationRegistrySerialize
module.exports.revocationRegistryDeserialize = revocationRegistryDeserialize
module.exports.revocationRegistryRelease = revocationRegistryRelease
module.exports.schemaGetAttributes = schemaGetAttributes
module.exports.schemaPrepareForEndorser = schemaPrepareForEndorser
module.exports.schemaCreate = schemaCreate
module.exports.schemaGetSchemaId = schemaGetSchemaId
module.exports.schemaDeserialize = schemaDeserialize
module.exports.schemaSerialize = schemaSerialize
module.exports.schemaRelease = schemaRelease
module.exports.schemaUpdateState = schemaUpdateState
module.exports.schemaGetState = schemaGetState
module.exports.enableMocks = enableMocks
module.exports.shutdown = shutdown
module.exports.getVersion = getVersion
module.exports.walletOpenAsMain = walletOpenAsMain
module.exports.walletCreateMain = walletCreateMain
module.exports.walletCloseMain = walletCloseMain
module.exports.vcxInitIssuerConfig = vcxInitIssuerConfig
module.exports.configureIssuerWallet = configureIssuerWallet
module.exports.unpack = unpack
module.exports.createPairwiseInfo = createPairwiseInfo
module.exports.walletImport = walletImport
module.exports.walletExport = walletExport
module.exports.getVerkeyFromWallet = getVerkeyFromWallet
module.exports.rotateVerkey = rotateVerkey
module.exports.rotateVerkeyStart = rotateVerkeyStart
module.exports.rotateVerkeyApply = rotateVerkeyApply
