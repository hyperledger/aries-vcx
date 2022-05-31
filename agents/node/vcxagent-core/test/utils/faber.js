/* eslint-env jest */
const { buildRevocationDetails } = require('../../src')
const { createVcxAgent, getSampleSchemaData } = require('../../src')
const { ConnectionStateType, IssuerStateType, VerifierStateType, generatePublicInvite } = require('@hyperledger/node-vcx-wrapper')
const { getAliceSchemaAttrs, getFaberCredDefName, getFaberProofData } = require('./data')
const sleep = require('sleep-promise')

module.exports.createFaber = async function createFaber () {
  const agentName = `faber-${Math.floor(new Date() / 1000)}`
  const connectionId = 'connection-faber-to-alice'
  const issuerCredId = 'credential-for-alice'
  const agentId = 'faber-public-agent'
  let credDefId, revRegId
  const proofId = 'proof-from-alice'
  const logger = require('../../demo/logger')('Faber')

  const faberAgentConfig = {
    agentName,
    agencyUrl: 'http://localhost:8080',
    seed: '000000000000000000000000Trustee1',
    webhookUrl: `http://localhost:7209/notifications/${agentName}`,
    logger
  }

  const vcxAgent = await createVcxAgent(faberAgentConfig)
  const institutionDid = vcxAgent.getInstitutionDid()
  await vcxAgent.agentInitVcx()
  const agent = await vcxAgent.servicePublicAgents.publicAgentCreate(agentId, institutionDid)
  await vcxAgent.agentShutdownVcx()

  async function createInvite () {
    logger.info('Faber is going to generate invite')
    await vcxAgent.agentInitVcx()

    const invite = await vcxAgent.serviceConnections.inviterConnectionCreate(connectionId, undefined)
    logger.info(`Faber generated invite:\n${invite}`)
    const connection = await vcxAgent.serviceConnections.getVcxConnection(connectionId)
    expect(await connection.getState()).toBe(ConnectionStateType.Invited)

    await vcxAgent.agentShutdownVcx()

    return invite
  }

  async function createPublicInvite () {
    logger.info('Faber is going to generate public invite')
    await vcxAgent.agentInitVcx()

    const institutionDid = vcxAgent.getInstitutionDid()
    logger.info(`Faber creating public agent ${agentId}`)
    await vcxAgent.servicePublicAgents.publicAgentCreate(agentId, institutionDid)
    logger.info(`Faber creating public invitation for did ${institutionDid}`)
    const publicInvitation = await generatePublicInvite(institutionDid, 'Faber')
    logger.info(`Faber generated public invite:\n${publicInvitation}`)

    await vcxAgent.agentShutdownVcx()

    return publicInvitation
  }

  async function createOobMessageWithService (wrappedMessage) {
    logger.info('Faber is going to generate out of band message')
    await vcxAgent.agentInitVcx()

    const service = await agent.getService()
    const oobMsg = await vcxAgent.serviceOutOfBand.createOobMessageWithService(wrappedMessage, 'faber-oob-msg', service)

    await vcxAgent.agentShutdownVcx()

    return oobMsg
  }

  async function createOobMessageWithDid (wrappedMessage) {
    logger.info('Faber is going to generate out of band message')
    await vcxAgent.agentInitVcx()

    const publicDid = vcxAgent.getInstitutionDid()
    const oobMsg = await vcxAgent.serviceOutOfBand.createOobMessageWithDid(wrappedMessage, 'faber-oob-msg', publicDid)

    await vcxAgent.agentShutdownVcx()

    return oobMsg
  }

  async function createOobCredOffer (usePublicDid = true) {
    await vcxAgent.agentInitVcx()
    const schemaAttrs = getAliceSchemaAttrs()
    const credOfferMsg = await vcxAgent.serviceCredIssuer.buildOfferAndMarkAsSent(issuerCredId, credDefId, schemaAttrs)
    await vcxAgent.agentShutdownVcx()
    if (usePublicDid) {
      return await createOobMessageWithDid(credOfferMsg)
    } else {
      return await createOobMessageWithService(credOfferMsg)
    }
  }

  async function createOobProofRequest (usePublicDid = true) {
    await vcxAgent.agentInitVcx()

    const issuerDid = vcxAgent.getInstitutionDid()
    const proofData = getFaberProofData(issuerDid, proofId)
    logger.info(`Faber is sending proof request to connection ${connectionId}`)
    const presentationRequestMsg = await vcxAgent.serviceVerifier.buildProofReqAndMarkAsSent(proofId, proofData)

    await vcxAgent.agentShutdownVcx()
    if (usePublicDid) {
      return await createOobMessageWithDid(presentationRequestMsg)
    } else {
      return await createOobMessageWithService(presentationRequestMsg)
    }
  }

  async function sendConnectionResponse () {
    logger.info('Faber is going to generate invite')
    await vcxAgent.agentInitVcx()

    expect(await vcxAgent.serviceConnections.connectionUpdate(connectionId)).toBe(ConnectionStateType.Responded)

    await vcxAgent.agentShutdownVcx()
  }

  async function updateConnection (expectedNextState) {
    logger.info(`Faber is going to update connection, expecting new state of ${expectedNextState}`)
    await vcxAgent.agentInitVcx()

    expect(await vcxAgent.serviceConnections.connectionUpdate(connectionId)).toBe(expectedNextState)

    await vcxAgent.agentShutdownVcx()
  }

  async function createCredDef (revocationDetails, tailsUrl) {
    revocationDetails = revocationDetails || buildRevocationDetails({ supportRevocation: false })

    await vcxAgent.agentInitVcx()

    logger.info('Faber writing schema on ledger')
    const schemaId = await vcxAgent.serviceLedgerSchema.createSchema(getSampleSchemaData())
    await sleep(500)

    logger.info('Faber writing credential definition on ledger')
    credDefId = getFaberCredDefName()
    await vcxAgent.serviceLedgerCredDef.createCredentialDefinition(
      schemaId,
      credDefId,
      revocationDetails,
      tailsUrl
    )
    await vcxAgent.agentShutdownVcx()
  }

  async function buildLedgerPrimitives (revocationDetails, tailsUrl) {
    await vcxAgent.agentInitVcx()

    logger.info('Faber writing schema on ledger')
    const schemaId = await vcxAgent.serviceLedgerSchema.createSchema(getSampleSchemaData())
    await sleep(500)

    logger.info('Faber writing credential definition on ledger')
    revocationDetails = revocationDetails || buildRevocationDetails({ supportRevocation: false })
    await vcxAgent.serviceLedgerCredDef.createCredentialDefinition(
      schemaId,
      getFaberCredDefName(),
      revocationDetails,
      tailsUrl
    )
    credDefId = getFaberCredDefName()
    await vcxAgent.agentShutdownVcx()
  }

  async function buildLedgerPrimitivesV2 (revocationDetails) {
    await vcxAgent.agentInitVcx()

    logger.info('Faber writing schema on ledger')
    const schemaId = await vcxAgent.serviceLedgerSchema.createSchema(getSampleSchemaData())
    await sleep(500)

    logger.info('Faber writing credential definition on ledger')
    const supportRevocation = !!revocationDetails
    revocationDetails = revocationDetails || buildRevocationDetails({ supportRevocation: false })
    await vcxAgent.serviceLedgerCredDef.createCredentialDefinitionV2(
      schemaId,
      getFaberCredDefName(),
      supportRevocation
    )
    credDefId = getFaberCredDefName()
    const _credDefId = await vcxAgent.serviceLedgerCredDef.getCredDefId(credDefId)
    if (supportRevocation && revocationDetails) {
      const { tailsDir, maxCreds } = revocationDetails
      logger.info('Faber writing revocation registry');
      ({ revRegId } = await vcxAgent.serviceLedgerRevReg.createRevocationRegistry(institutionDid, _credDefId, 1, tailsDir, maxCreds))
    }
    await vcxAgent.agentShutdownVcx()
  }

  async function sendCredentialOffer () {
    await vcxAgent.agentInitVcx()
    const schemaAttrs = getAliceSchemaAttrs()
    await vcxAgent.serviceCredIssuer.sendOffer(issuerCredId, connectionId, credDefId, schemaAttrs)
    await vcxAgent.agentShutdownVcx()
  }

  async function sendCredentialOfferV2 () {
    await vcxAgent.agentInitVcx()

    logger.info('Issuer sending credential offer')
    const schemaAttrs = getAliceSchemaAttrs()
    await vcxAgent.serviceCredIssuer.sendOfferV2(issuerCredId, revRegId, connectionId, credDefId, schemaAttrs)
    logger.debug('Credential offer sent')

    await vcxAgent.agentShutdownVcx()
  }
  async function updateStateCredentialV2 (expectedState) {
    await vcxAgent.agentInitVcx()

    logger.info('Issuer updating state of credential with connection')
    expect(await vcxAgent.serviceCredIssuer.credentialUpdate(issuerCredId, connectionId)).toBe(expectedState)

    await vcxAgent.agentShutdownVcx()
  }

  async function sendCredential () {
    await vcxAgent.agentInitVcx()

    logger.info('Issuer sending credential')
    expect(await vcxAgent.serviceCredIssuer.sendCredential(issuerCredId, connectionId)).toBe(IssuerStateType.CredentialSent)
    logger.info('Credential sent')

    await vcxAgent.agentShutdownVcx()
  }

  async function receiveCredentialAck () {
    await vcxAgent.agentInitVcx()

    logger.info('Issuer waiting for credential ack')
    await vcxAgent.serviceCredIssuer.waitForCredentialAck(issuerCredId, connectionId)
    logger.info('Credential ack received')

    await vcxAgent.agentShutdownVcx()
  }

  async function requestProofFromAlice () {
    logger.info('Faber going to request proof from Alice')
    await vcxAgent.agentInitVcx()
    const issuerDid = vcxAgent.getInstitutionDid()
    const proofData = getFaberProofData(issuerDid, proofId)
    logger.info(`Faber is creating proof ${proofId}`)
    await vcxAgent.serviceVerifier.createProof(proofId, proofData)
    logger.info(`Faber is sending proof request to connection ${connectionId}`)
    const { state, proofRequestMessage } = await vcxAgent.serviceVerifier.sendProofRequest(connectionId, proofId)
    expect(state).toBe(VerifierStateType.PresentationRequestSent)
    await vcxAgent.agentShutdownVcx()
    return proofRequestMessage
  }

  async function updateStateVerifierProofV2 (expectedNextState) {
    logger.info(`Verifier updating state of proof, expecting it to be in state ${expectedNextState}`)
    await vcxAgent.agentInitVcx()

    expect(await vcxAgent.serviceVerifier.proofUpdate(proofId, connectionId)).toBe(expectedNextState)

    await vcxAgent.agentShutdownVcx()
  }

  async function verifySignature (dataBase64, signatureBase64) {
    logger.debug(`Faber is going to verift signed data. Data=${dataBase64} signature=${signatureBase64}`)
    await vcxAgent.agentInitVcx()

    const isValid = await vcxAgent.serviceConnections.verifySignature(connectionId, dataBase64, signatureBase64)

    await vcxAgent.agentShutdownVcx()
    return isValid
  }

  async function downloadReceivedMessages () {
    logger.info('Faber is going to download messages using getMessages')
    await vcxAgent.agentInitVcx()
    const agencyMessages = await vcxAgent.serviceConnections.getMessages(connectionId, ['MS-103'])
    await vcxAgent.agentShutdownVcx()
    return agencyMessages
  }

  async function _downloadConnectionRequests () {
    logger.info('Faber is going to download connection requests')
    const connectionRequests = await vcxAgent.servicePublicAgents.downloadConnectionRequests(agentId)
    logger.info(`Downloaded connection requests: ${connectionRequests}`)
    return JSON.parse(connectionRequests)
  }

  async function createConnectionFromReceivedRequest () {
    logger.info('Faber is going to download connection requests')
    await vcxAgent.agentInitVcx()

    const requests = await _downloadConnectionRequests()
    await vcxAgent.serviceConnections.inviterConnectionCreateFromRequest(connectionId, agentId, JSON.stringify(requests[0]))
    expect(await vcxAgent.serviceConnections.connectionUpdate(connectionId)).toBe(ConnectionStateType.Responded)

    await vcxAgent.agentShutdownVcx()
  }

  async function updateMessageStatus (uids) {
    await vcxAgent.agentInitVcx()
    await vcxAgent.serviceConnections.updateMessagesStatus(connectionId, uids)
    await vcxAgent.agentShutdownVcx()
  }

  async function updateAllReceivedMessages () {
    await vcxAgent.agentInitVcx()
    await vcxAgent.serviceConnections.updateAllReceivedMessages(connectionId)
    await vcxAgent.agentShutdownVcx()
  }

  async function downloadReceivedMessagesV2 () {
    logger.info('Faber is going to download messages using getMessagesV2')
    await vcxAgent.agentInitVcx()
    const agencyMessages = await vcxAgent.serviceConnections.getMessagesV2(connectionId, ['MS-103'])
    await vcxAgent.agentShutdownVcx()
    return agencyMessages
  }

  async function getCredentialRevRegId () {
    logger.info(`Faber is going to obtain rev reg id for cred id ${issuerCredId}`)
    await vcxAgent.agentInitVcx()
    const revRegId = await vcxAgent.serviceCredIssuer.getRevRegId(issuerCredId)
    logger.debug(`Faber obtained rev reg id ${revRegId}`)
    await vcxAgent.agentShutdownVcx()
    return revRegId
  }

  async function getTailsFile () {
    logger.info(`Faber is going to obtain tails file for cred id ${issuerCredId}`)
    await vcxAgent.agentInitVcx()
    const tailsFile = await vcxAgent.serviceLedgerCredDef.getTailsFile(issuerCredId)
    await vcxAgent.agentShutdownVcx()
    logger.debug(`Faber obtained tails file ${tailsFile}`)
    return tailsFile
  }

  async function getTailsHash () {
    logger.info(`Faber is going to obtain tails hash for cred def id ${credDefId}`)
    await vcxAgent.agentInitVcx()
    const tailsHash = await vcxAgent.serviceLedgerCredDef.getTailsHash(credDefId)
    logger.info(`Faber obtained tails hash ${tailsHash}`)
    await vcxAgent.agentShutdownVcx()
    return tailsHash
  }

  async function sendMessage (message) {
    logger.info('Faber is going to send message')
    await vcxAgent.agentInitVcx()
    await vcxAgent.serviceConnections.sendMessage(connectionId, message)
    await vcxAgent.agentShutdownVcx()
  }

  return {
    buildLedgerPrimitives,
    buildLedgerPrimitivesV2,
    createCredDef,
    downloadReceivedMessages,
    downloadReceivedMessagesV2,
    sendMessage,
    verifySignature,
    createInvite,
    createPublicInvite,
    createOobMessageWithDid,
    createOobMessageWithService,
    createOobProofRequest,
    createConnectionFromReceivedRequest,
    updateConnection,
    sendConnectionResponse,
    sendCredentialOffer,
    sendCredentialOfferV2,
    createOobCredOffer,
    updateStateCredentialV2,
    sendCredential,
    receiveCredentialAck,
    requestProofFromAlice,
    updateStateVerifierProofV2,
    getCredentialRevRegId,
    getTailsFile,
    getTailsHash,
    updateMessageStatus,
    updateAllReceivedMessages
  }
}
