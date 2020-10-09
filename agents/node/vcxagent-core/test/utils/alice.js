/* eslint-env jest */
const { protocolTypes, createVcxAgent } = require('../../src/index')
const { StateType } = require('@absaoss/node-vcx-wrapper')

module.exports.createAlice = async function createAlice () {
  const agentName = `alice-${Math.floor(new Date() / 1000)}`
  const connectionId = 'connection-alice-to-faber'
  const holderCredentialId = 'credential-of-alice'
  const disclosedProofId = 'proof-from-alice'
  const logger = require('../../../vcxagent-cli/logger')('Alice')

  const aliceAgentConfig = {
    agentName,
    protocolType: protocolTypes.v4,
    agencyUrl: 'http://localhost:8080',
    seed: '000000000000000000000000Alice000',
    webhookUrl: `http://localhost:7209/notifications/${agentName}`,
    usePostgresWallet: false,
    logger
  }
  const vcxAgent = await createVcxAgent(aliceAgentConfig)

  async function acceptInvite (invite) {
    logger.info('Alice establishing connection with Faber')

    await vcxAgent.agentInitVcx()

    const connectionAliceToFaber = await vcxAgent.serviceConnections.inviteeConnectionAcceptFromInvitation(connectionId, invite)
    expect(await connectionAliceToFaber.getState()).toBe(StateType.RequestReceived)

    await vcxAgent.agentShutdownVcx()
  }

  async function updateConnection (expectedNextState) {
    logger.info(`Alice is going to update connection, expecting new state of ${expectedNextState}`)
    await vcxAgent.agentInitVcx()

    const state = await vcxAgent.serviceConnections.connectionUpdate(connectionId)
    expect(state).toBe(expectedNextState)

    await vcxAgent.agentShutdownVcx()
  }

  async function acceptCredentialOffer () {
    await vcxAgent.agentInitVcx()

    logger.info('Alice accepting creadential offer')
    await vcxAgent.serviceCredHolder.waitForCredentialOfferAndAccept({ connectionId, holderCredentialId })

    await vcxAgent.agentShutdownVcx()
  }

  async function sendHolderProof (proofRequest) {
    await vcxAgent.agentInitVcx()

    await vcxAgent.serviceProver.buildDisclosedProof(disclosedProofId, proofRequest)
    const selectedCreds = await vcxAgent.serviceProver.selectCredentials(disclosedProofId)
    const selfAttestedAttrs = { attribute_3: 'Smith' }
    await vcxAgent.serviceProver.generateProof(disclosedProofId, selectedCreds, selfAttestedAttrs)
    const state = await vcxAgent.serviceProver.sendDisclosedProof(disclosedProofId, connectionId)
    expect(state).toBe(StateType.OfferSent)

    await vcxAgent.agentShutdownVcx()
  }

  async function updateStateHolderProofV2 (expectedNextState) {
    logger.info(`Holder updating state of disclosed proof, expecting it to be in state ${expectedNextState}`)
    await vcxAgent.agentInitVcx()

    const state = await vcxAgent.serviceProver.disclosedProofUpdate(disclosedProofId, connectionId)
    expect(state).toBe(expectedNextState)

    await vcxAgent.agentShutdownVcx()
  }

  async function updateStateCredentialV2 (expectedState) {
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

  return {
    signData,
    acceptInvite,
    updateConnection,
    acceptCredentialOffer,
    updateStateCredentialV2,
    sendHolderProof,
    updateStateHolderProofV2
  }
}
