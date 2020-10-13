/* eslint-env jest */
const { buildRevocationDetails } = require('../../src')
const { createVcxAgent, getSampleSchemaData } = require('../../src')
const { StateType } = require('@absaoss/node-vcx-wrapper')
const { getAliceSchemaAttrs, getFaberCredDefName, getFaberProofData } = require('./data')

module.exports.createFaber = async function createFaber () {
  const agentName = `faber-${Math.floor(new Date() / 1000)}`
  const connectionId = 'connection-faber-to-alice'
  const issuerCredId = 'credential-for-alice'
  const proofId = 'proof-from-alice'
  const logger = require('../../../vcxagent-cli/logger')('Faber')

  const faberAgentConfig = {
    agentName,
    agencyUrl: 'http://localhost:8080',
    seed: '000000000000000000000000Trustee1',
    webhookUrl: `http://localhost:7209/notifications/${agentName}`,
    usePostgresWallet: false,
    logger
  }

  const vcxAgent = await createVcxAgent(faberAgentConfig)

  async function createInvite () {
    logger.info('Faber is going to generate invite')
    await vcxAgent.agentInitVcx()

    const invite = await vcxAgent.serviceConnections.inviterConnectionCreate(connectionId, undefined)
    logger.info(`Faber generated invite:\n${invite}`)
    const connection = await vcxAgent.serviceConnections.getVcxConnection(connectionId)
    expect(await connection.getState()).toBe(StateType.OfferSent)

    await vcxAgent.agentShutdownVcx()

    return invite
  }

  async function sendConnectionResponse () {
    logger.info('Faber is going to generate invite')
    await vcxAgent.agentInitVcx()

    expect(await vcxAgent.serviceConnections.connectionUpdate(connectionId)).toBe(StateType.RequestReceived)

    await vcxAgent.agentShutdownVcx()
  }

  async function updateConnection (expectedNextState) {
    logger.info(`Faber is going to update connection, expecting new state of ${expectedNextState}`)
    await vcxAgent.agentInitVcx()

    expect(await vcxAgent.serviceConnections.connectionUpdate(connectionId)).toBe(expectedNextState)

    await vcxAgent.agentShutdownVcx()
  }

  async function sendCredentialOffer () {
    await vcxAgent.agentInitVcx()

    logger.info('Faber writing schema on ledger')
    const schemaId = await vcxAgent.serviceLedgerSchema.createSchema(getSampleSchemaData())

    logger.info('Faber writing credential definition on ledger')
    await vcxAgent.serviceLedgerCredDef.createCredentialDefinition(
      schemaId,
      getFaberCredDefName(),
      buildRevocationDetails({ supportRevocation: false })
    )

    logger.info('Faber sending credential to Alice')
    const schemaAttrs = getAliceSchemaAttrs()
    const credDefId = getFaberCredDefName()
    await vcxAgent.serviceCredIssuer.sendOffer(issuerCredId, connectionId, credDefId, schemaAttrs)

    await vcxAgent.agentShutdownVcx()
  }

  async function updateStateCredentialV1 () {
    logger.info('Issuer updating state of credential')
    await vcxAgent.agentInitVcx()

    logger.info('Issuer updating state of credential with connection')
    await vcxAgent.serviceCredIssuer.credentialUpdateV1(issuerCredId)

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
    expect(await vcxAgent.serviceCredIssuer.sendCredential(issuerCredId, connectionId)).toBe(StateType.Accepted)
    logger.info('Credential sent')

    await vcxAgent.agentShutdownVcx()
  }

  async function requestProofFromAlice () {
    logger.info('Faber going to request proof from Alice')
    await vcxAgent.agentInitVcx()
    const issuerDid = vcxAgent.getInstitutionDid()
    const proofData = getFaberProofData(issuerDid, proofId)
    await vcxAgent.serviceVerifier.createProof(proofId, proofData)
    const { state, proofRequestMessage } = await vcxAgent.serviceVerifier.sendProofRequest(connectionId, proofId)
    expect(state).toBe(StateType.OfferSent)
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
