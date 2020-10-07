/* eslint-env jest */
const { protocolTypes, createVcxAgent, getSampleSchemaData } = require('../../src/index')
const { StateType } = require('@absaoss/node-vcx-wrapper')
const { getAliceSchemaAttrs, getFaberCredDefName, getFaberProofData } = require('./data')

module.exports.createFaber = async function createFaber () {
  const faberId = `faber-${Math.floor(new Date() / 1000)}`
  const connectionName = 'connection-faber-to-alice'
  const issuerCredName = 'credential-for-alice'
  const proofName = 'proof-from-alice'
  const logger = require('../../../vcxagent-cli/logger')('Faber')

  const faberAgentConfig = {
    agentName: faberId,
    protocolType: protocolTypes.v4,
    agencyUrl: 'http://localhost:8080',
    seed: '000000000000000000000000Trustee1',
    webhookUrl: `http://localhost:7209/notifications/${faberId}`,
    usePostgresWallet: false,
    logger
  }

  const vcxAgent = await createVcxAgent(faberAgentConfig)

  async function createInvite () {
    logger.info('Faber is going to generate invite')
    await vcxAgent.agentInitVcx()

    const { invite, connection } = await vcxAgent.serviceConnections.inviterConnectionCreate(connectionName, undefined)
    expect(await connection.getState()).toBe(StateType.OfferSent)
    logger.info(`Faber generated invite:\n${invite}`)

    await vcxAgent.agentShutdownVcx()

    return invite
  }

  async function sendConnectionResponse () {
    logger.info('Faber is going to generate invite')
    await vcxAgent.agentInitVcx()

    const state = await vcxAgent.serviceConnections.connectionUpdate(connectionName)
    expect(state).toBe(StateType.RequestReceived)

    await vcxAgent.agentShutdownVcx()
  }

  async function updateConnection (expectedNextState) {
    logger.info(`Faber is going to update connection, expecting new state of ${expectedNextState}`)
    await vcxAgent.agentInitVcx()

    const state = await vcxAgent.serviceConnections.connectionUpdate(connectionName)
    expect(state).toBe(expectedNextState)

    await vcxAgent.agentShutdownVcx()
  }

  async function sendCredentialOffer () {
    logger.info('Faber sending credential')
    await vcxAgent.agentInitVcx()

    logger.info('Faber writing schema on ledger')
    const schemaId = await vcxAgent.serviceLedger.createSchema(getSampleSchemaData())

    logger.info('Faber writing credential definition on ledger')
    await vcxAgent.serviceLedger.createCredentialDefinition(schemaId, getFaberCredDefName())

    logger.info('Faber sending credential to Alice')
    const schemaAttrs = getAliceSchemaAttrs()
    await vcxAgent.serviceCredIssuer.sendOffer({ issuerCredName, connectionName, credDefName: getFaberCredDefName(), schemaAttrs })

    await vcxAgent.agentShutdownVcx()
  }

  async function updateStateCredentialV1 () {
    logger.info('Issuer updating state of credential')
    await vcxAgent.agentInitVcx()

    logger.info('Issuer updating state of credential with connection')
    expect(vcxAgent.serviceCredIssuer.credentialUpdateV1(issuerCredName)).rejects.toThrow('Obj was not found with handle')

    await vcxAgent.agentShutdownVcx()
  }

  async function updateStateCredentialV2 (expectedState) {
    await vcxAgent.agentInitVcx()

    logger.info('Issuer updating state of credential with connection')
    expect(await vcxAgent.serviceCredIssuer.credentialUpdate(issuerCredName, connectionName)).toBe(expectedState)

    await vcxAgent.agentShutdownVcx()
  }

  async function sendCredential () {
    await vcxAgent.agentInitVcx()

    logger.info('Issuer sending credential')
    // await issuerVcxCred.sendCredential(connectionName)
    expect(await vcxAgent.serviceCredIssuer.sendCredential({ issuerCredName, connectionName })).toBe(StateType.Accepted)
    logger.info('Credential sent')

    await vcxAgent.agentShutdownVcx()
  }

  async function requestProofFromAlice () {
    logger.info('Faber going to request proof from Alice')
    await vcxAgent.agentInitVcx()
    const issuerDid = vcxAgent.getInstitutionDid()
    const proofData = getFaberProofData(issuerDid, proofName)
    await vcxAgent.serviceVerifier.createProof({ proofName, proofData })
    const { state, proofRequestMessage } = await vcxAgent.serviceVerifier.sendProofRequest(connectionName, proofName)
    expect(state).toBe(StateType.OfferSent)
    await vcxAgent.agentShutdownVcx()
    return proofRequestMessage
  }

  async function updateStateVerifierProofV2 (expectedNextState) {
    logger.info(`Verifier updating state of proof, expecting it to be in state ${expectedNextState}`)
    await vcxAgent.agentInitVcx()

    const state = await vcxAgent.serviceVerifier.proofUpdate(proofName, connectionName)
    expect(state).toBe(expectedNextState)

    await vcxAgent.agentShutdownVcx()
  }

  async function verifySignature (dataBase64, signatureBase64) {
    logger.debug(`Faber is going to verift signed data. Data=${dataBase64} signature=${signatureBase64}`)
    await vcxAgent.agentInitVcx()

    const isValid = await vcxAgent.serviceConnections.verifySignature(connectionName, dataBase64, signatureBase64)

    await vcxAgent.agentShutdownVcx()
    return isValid
  }

  return {
    verifySignature,
    createInvite,
    updateConnection,
    sendConnectionResponse,
    sendCredentialOffer,
    updateStateCredentialV1,
    updateStateCredentialV2,
    sendCredential,
    requestProofFromAlice,
    updateStateVerifierProofV2
  }
}
