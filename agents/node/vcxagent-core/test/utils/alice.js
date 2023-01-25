/* eslint-env jest */
const { createVcxAgent } = require('../../src/index')
const { ConnectionStateType, ProverStateType, OutOfBandReceiver, HolderStateType, unpack } = require('@hyperledger/node-vcx-wrapper')

module.exports.createAlice = async function createAlice (serviceEndpoint = 'http://localhost:5401') {
  const agentName = `alice-${Math.floor(new Date() / 1000)}`
  const connectionId = 'connection-alice-to-faber'
  const holderCredentialId = 'credential-of-alice'
  const disclosedProofId = 'proof-from-alice'
  const logger = require('../../demo/logger')('Alice')

  const aliceAgentConfig = {
    agentName,
    agencyUrl: 'http://localhost:8080',
    seed: '000000000000000000000000Alice000',
    webhookUrl: `http://localhost:7209/notifications/${agentName}`,
    endpointInfo: {
      serviceEndpoint,
      routingKeys: []
    },
    logger
  }
  const vcxAgent = await createVcxAgent(aliceAgentConfig)

  async function acceptInvite (invite) {
    logger.info(`Alice establishing connection with Faber using invite ${invite}`)
    await vcxAgent.agentInitVcx()

    await vcxAgent.serviceConnections.inviteeConnectionAcceptFromInvitation(connectionId, invite)
    const connection = await vcxAgent.serviceConnections.getVcxConnection(connectionId)
    expect(await connection.getState()).toBe(ConnectionStateType.Requested)

    await vcxAgent.agentShutdownVcx()
  }

  async function createNonmediatedConnectionFromInvite (invite) {
    logger.info(`Alice establishing connection with Faber using invite ${invite}`)
    await vcxAgent.agentInitVcx()

    await vcxAgent.serviceNonmediatedConnections.inviteeConnectionCreateFromInvite(connectionId, invite)
    expect(await vcxAgent.serviceNonmediatedConnections.getState(connectionId)).toBe(ConnectionStateType.Requested)

    await vcxAgent.agentShutdownVcx()
  }

  async function nonmediatedConnectionProcessResponse (response) {
    logger.info(`Alice processing response ${response}`)
    await vcxAgent.agentInitVcx()

    await vcxAgent.serviceNonmediatedConnections.inviteeConnectionProcessResponse(connectionId, response)
    expect(await vcxAgent.serviceNonmediatedConnections.getState(connectionId)).toBe(ConnectionStateType.Finished)

    await vcxAgent.agentShutdownVcx()
  }

  async function createConnectionUsingOobMessage (oobMsg) {
    logger.info('createConnectionUsingOobMessage >> Alice going to create connection using oob message')
    logger.debug(`createConnectionUsingOobMessage >> oobMsg = ${oobMsg}`)
    await vcxAgent.agentInitVcx()

    await vcxAgent.serviceOutOfBand.createConnectionFromOobMsg(connectionId, oobMsg)

    await vcxAgent.agentShutdownVcx()
  }

  async function createNonmediatedConnectionUsingOobMessage (oobMsg) {
    logger.info('createNonmediatedConnectionUsingOobMessage >> Alice going to create connection using oob message')
    logger.debug(`createNonmediatedConnectionUsingOobMessage >> oobMsg = ${oobMsg}`)
    await vcxAgent.agentInitVcx()

    await vcxAgent.serviceOutOfBand.createNonmediatedConnectionFromOobMsg(connectionId, oobMsg)

    await vcxAgent.agentShutdownVcx()
  }

  async function createOrReuseConnectionUsingOobMsg (oobMsg) {
    logger.info('createOrReuseConnectionUsingOobMsg >> Alice going to create or reuse connection using oob message')
    logger.debug(`createOrReuseConnectionUsingOobMsg >> oobMsg = ${oobMsg}`)
    await vcxAgent.agentInitVcx()
    let reused = false

    if (await vcxAgent.serviceOutOfBand.connectionExists([connectionId], oobMsg)) {
      await vcxAgent.serviceOutOfBand.reuseConnectionFromOobMsg(connectionId, oobMsg)
      reused = true
    } else {
      await vcxAgent.serviceOutOfBand.createConnectionFromOobMsg(connectionId, oobMsg)
    }

    await vcxAgent.agentShutdownVcx()
    return reused
  }

  async function updateConnection (expectedNextState) {
    logger.info(`Alice is going to update connection, expecting new state of ${expectedNextState}`)
    await vcxAgent.agentInitVcx()

    expect(await vcxAgent.serviceConnections.connectionUpdate(connectionId)).toBe(expectedNextState)

    await vcxAgent.agentShutdownVcx()
  }

  async function handleMessage (ariesMsg) {
    logger.info(`Alice is going to try handle incoming messages`)
    await vcxAgent.agentInitVcx()

    await vcxAgent.serviceConnections.handleMessage(connectionId, ariesMsg)

    await vcxAgent.agentShutdownVcx()
  }

  async function acceptCredentialOffer () {
    await vcxAgent.agentInitVcx()
    logger.info('Alice accepting credential offer')

    await vcxAgent.serviceCredHolder.waitForCredentialOfferAndAccept(connectionId, holderCredentialId)

    await vcxAgent.agentShutdownVcx()
  }

  async function acceptOobCredentialOffer (oobCredOfferMsg) {
    await vcxAgent.agentInitVcx()
    logger.info('acceptOobCredentialOffer >>> Alice going to accept oob cred offer.')

    const oobReceiver = await OutOfBandReceiver.createWithMessage(oobCredOfferMsg)
    const credOffer = await oobReceiver.extractMessage()
    logger.info('acceptOobCredentialOffer >>> Extracted attached message')
    logger.debug(`acceptOobCredentialOffer >>> attached message: ${credOffer}`)
    await vcxAgent.serviceCredHolder.createCredentialFromOfferAndSendRequest(connectionId, holderCredentialId, credOffer)
    const state = await vcxAgent.serviceCredHolder.getState(holderCredentialId)
    expect(state).toBe(HolderStateType.RequestSent)

    await vcxAgent.agentShutdownVcx()
  }

  async function sendHolderProof (proofRequest, _mapRevRegId) {
    await vcxAgent.agentInitVcx()

    const mapRevRegId = _mapRevRegId || ((_revRegId) => { throw Error('Tails file should not be needed') })
    await vcxAgent.serviceProver.buildDisclosedProof(disclosedProofId, proofRequest)
    const { selectedCreds } = await vcxAgent.serviceProver.selectCredentials(disclosedProofId, mapRevRegId)
    const selfAttestedAttrs = { attribute_3: 'Smith' }
    await vcxAgent.serviceProver.generateProof(disclosedProofId, selectedCreds, selfAttestedAttrs)
    expect(await vcxAgent.serviceProver.sendDisclosedProof(disclosedProofId, connectionId)).toBe(ProverStateType.PresentationSent)

    await vcxAgent.agentShutdownVcx()
  }

  async function updateStateHolderProof (expectedNextState) {
    logger.info(`Holder updating state of disclosed proof, expecting it to be in state ${expectedNextState}`)
    await vcxAgent.agentInitVcx()

    expect(await vcxAgent.serviceProver.disclosedProofUpdate(disclosedProofId, connectionId)).toBe(expectedNextState)

    await vcxAgent.agentShutdownVcx()
  }

  async function updateStateCredential (expectedState) {
    logger.info('Holder updating state of credential with connection')
    await vcxAgent.agentInitVcx()

    expect(await vcxAgent.serviceCredHolder.credentialUpdate(holderCredentialId, connectionId)).toBe(expectedState)

    await vcxAgent.agentShutdownVcx()
  }

  async function signData (dataBase64) {
    logger.info('Alice is going to sign data')
    await vcxAgent.agentInitVcx()

    const signatureBase64 = await vcxAgent.serviceConnections.signData(connectionId, dataBase64)

    await vcxAgent.agentShutdownVcx()

    logger.debug(`Alice signed data. Data=${dataBase64} signature=${signatureBase64}`)
    return signatureBase64
  }

  async function sendMessage (message) {
    logger.info('Alice is going to send message')
    await vcxAgent.agentInitVcx()

    await vcxAgent.serviceConnections.sendMessage(connectionId, message)

    await vcxAgent.agentShutdownVcx()
  }

  async function nonmediatedConnectionSendMessage (message) {
    logger.info('Alice is going to send message')
    await vcxAgent.agentInitVcx()

    await vcxAgent.serviceNonmediatedConnections.sendMessage(connectionId, message)

    await vcxAgent.agentShutdownVcx()
  }

  async function getTailsLocation () {
    logger.info('Alice is going to get tails location')
    await vcxAgent.agentInitVcx()

    const tailsLocation = await vcxAgent.serviceCredHolder.getTailsLocation(holderCredentialId)
    logger.debug(`Alice obtained tails location ${tailsLocation}`)

    await vcxAgent.agentShutdownVcx()
    return tailsLocation
  }

  async function getTailsHash () {
    logger.info('Alice getting tails hash')
    await vcxAgent.agentInitVcx()

    const tailsHash = await vcxAgent.serviceCredHolder.getTailsHash(holderCredentialId)
    logger.debug(`Alice obtained tails hash ${tailsHash}`)

    await vcxAgent.agentShutdownVcx()
    return tailsHash
  }

  async function downloadReceivedMessagesV2 () {
    logger.info('Alice is going to download messages using getMessagesV2')
    await vcxAgent.agentInitVcx()

    const agencyMessages = await vcxAgent.serviceConnections.getMessagesV2(connectionId, ['MS-103'])

    await vcxAgent.agentShutdownVcx()
    return agencyMessages
  }

  async function sendPing () {
    logger.info('Alice is going to send ping')
    await vcxAgent.agentInitVcx()

    const res = await vcxAgent.serviceConnections.sendPing(connectionId)
    logger.info(`Operation result = ${JSON.stringify(res)}`)

    await vcxAgent.agentShutdownVcx()
  }

  async function discoverTheirFeatures () {
    logger.info('Alice is going to request Faber\'s Aries features.')
    await vcxAgent.agentInitVcx()

    const res = await vcxAgent.serviceConnections.discoverTheirFeatures(connectionId)
    logger.info(`Operation result = ${JSON.stringify(res)}`)

    await vcxAgent.agentShutdownVcx()
  }

  async function unpackMsg (encryptedMsg) {
    logger.info(`Alice is going to unpack message of length ${encryptedMsg.length}`)
    await vcxAgent.agentInitVcx()

    const { message, sender_verkey: senderVerkey } = await unpack(encryptedMsg)

    logger.info(`Decrypted msg has length ${message.length}, sender verkey: ${senderVerkey}`)
    await vcxAgent.agentShutdownVcx()

    return { message, senderVerkey }
  }

  return {
    sendMessage,
    nonmediatedConnectionSendMessage,
    signData,
    acceptInvite,
    createNonmediatedConnectionFromInvite,
    nonmediatedConnectionProcessResponse,
    createConnectionUsingOobMessage,
    createNonmediatedConnectionUsingOobMessage,
    createOrReuseConnectionUsingOobMsg,
    acceptOobCredentialOffer,
    updateConnection,
    handleMessage,
    acceptCredentialOffer,
    updateStateCredential,
    sendHolderProof,
    updateStateHolderProof,
    getTailsLocation,
    getTailsHash,
    downloadReceivedMessagesV2,
    sendPing,
    discoverTheirFeatures,
    unpackMsg
  }
}
