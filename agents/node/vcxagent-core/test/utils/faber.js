/* eslint-env jest */
const { createVcxAgent, getSampleSchemaData } = require('../../src')
const { ConnectionStateType, IssuerStateType, VerifierStateType, generatePublicInvite,
  createPwInfo, createService, getServiceFromLedger, unpack
} = require('@hyperledger/node-vcx-wrapper')
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
  let revRegTagNo = 1

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

  async function publishService (endpoint) {
    logger.info('Faber is going to write nonmediated service on the ledger')
    await vcxAgent.agentInitVcx()

    logger.info(`Faber creating pairwise info`)
    const pwInfo = await createPwInfo();
    logger.info(`Faber creating service for endpoint ${endpoint} and recipient key ${pwInfo.pw_vk}`)
    await createService(institutionDid, endpoint, [pwInfo.pw_vk], [])

    await vcxAgent.agentShutdownVcx()

    return pwInfo
  }

  async function readServiceFromLedger () {
    logger.info('Faber is going to read service from the ledger')
    await vcxAgent.agentInitVcx()

    const service = await getServiceFromLedger(institutionDid)

    await vcxAgent.agentShutdownVcx()

    return service
  }

  async function unpackMsg (encryptedMsg) {
    logger.info('Faber is going to unpack message')
    await vcxAgent.agentInitVcx()

    const { message, sender_verkey: senderVerkey } = await unpack(encryptedMsg);

    await vcxAgent.agentShutdownVcx()

    return { message, senderVerkey }
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
    const credOfferMsg = await vcxAgent.serviceCredIssuer.buildOfferAndMarkAsSent(issuerCredId, credDefId, revRegId, schemaAttrs)
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

  async function handleMessage (ariesMsg) {
    logger.info(`Faber is going to try handle incoming messages`)
    await vcxAgent.agentInitVcx()

    await vcxAgent.serviceConnections.handleMessage(connectionId, ariesMsg)

    await vcxAgent.agentShutdownVcx()
  }

  async function buildLedgerPrimitives (revocationDetails) {
    await vcxAgent.agentInitVcx()

    logger.info('Faber writing schema on ledger')
    const schemaId = await vcxAgent.serviceLedgerSchema.createSchema(getSampleSchemaData())
    await sleep(500)

    logger.info('Faber writing credential definition on ledger')
    const supportRevocation = !!revocationDetails
    await vcxAgent.serviceLedgerCredDef.createCredentialDefinitionV2(
      schemaId,
      getFaberCredDefName(),
      supportRevocation
    )
    credDefId = getFaberCredDefName()
    const credDefLedgerId = await vcxAgent.serviceLedgerCredDef.getCredDefId(credDefId)
    if (supportRevocation) {
      const { tailsDir, maxCreds, tailsUrl } = revocationDetails
      logger.info('Faber writing revocation registry');
      ({ revRegId } = await vcxAgent.serviceLedgerRevReg.createRevocationRegistry(institutionDid, credDefLedgerId, revRegTagNo, tailsDir, maxCreds, tailsUrl))
    }
    await vcxAgent.agentShutdownVcx()
  }

  async function rotateRevReg (tailsDir, maxCreds) {
    await vcxAgent.agentInitVcx()

    logger.info('Faber rotating revocation registry');
    const credDefLedgerId = await vcxAgent.serviceLedgerCredDef.getCredDefId(credDefId);
    ({ revRegId } = await vcxAgent.serviceLedgerRevReg.createRevocationRegistry(institutionDid, credDefLedgerId, revRegTagNo + 1, tailsDir, maxCreds))
    revRegTagNo += 1

    await vcxAgent.agentShutdownVcx()
  }

  async function sendCredentialOffer () {
    await vcxAgent.agentInitVcx()

    logger.info('Issuer sending credential offer')
    const schemaAttrs = getAliceSchemaAttrs()
    await vcxAgent.serviceCredIssuer.sendOfferV2(issuerCredId, revRegId, connectionId, credDefId, schemaAttrs)
    logger.debug('Credential offer sent')

    await vcxAgent.agentShutdownVcx()
  }
  async function updateStateCredential (expectedState) {
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

  async function updateStateVerifierProof (expectedNextState) {
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

  async function createConnectionFromReceivedRequestV2 (pwInfo, request) {
    logger.info('Faber is going to create a connection from a request')
    await vcxAgent.agentInitVcx()

    await vcxAgent.serviceConnections.inviterConnectionCreateFromRequestV2(connectionId, pwInfo, request)
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
    logger.info(`Faber is going to obtain tails file for rev reg id ${revRegId}`)
    await vcxAgent.agentInitVcx()
    const tailsFile = await vcxAgent.serviceLedgerCredDef.getTailsFile(issuerCredId)
    await vcxAgent.agentShutdownVcx()
    logger.debug(`Faber obtained tails file ${tailsFile}`)
    return tailsFile
  }

  async function getTailsHash () {
    logger.info(`Faber is going to obtain tails hash for rev reg id ${revRegId}`)
    await vcxAgent.agentInitVcx()
    const tailsHash = await vcxAgent.serviceLedgerRevReg.getTailsHash(revRegId)
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
    rotateRevReg,
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
    createConnectionFromReceivedRequestV2,
    updateConnection,
    handleMessage,
    sendConnectionResponse,
    sendCredentialOffer,
    createOobCredOffer,
    updateStateCredential,
    sendCredential,
    receiveCredentialAck,
    requestProofFromAlice,
    updateStateVerifierProof,
    getCredentialRevRegId,
    getTailsFile,
    getTailsHash,
    updateMessageStatus,
    updateAllReceivedMessages,
    publishService,
    readServiceFromLedger,
    unpackMsg
  }
}
