const {
  Proof, IssuerCredential, IssuerStateType
} = require('@hyperledger/node-vcx-wrapper')
const sleep = require('sleep-promise')
const {VerifierStateType} = require('../../../../../wrappers/node')

module.exports.createServiceVerifier = function createServiceVerifier ({ logger, loadConnection, saveProof, loadProof, listProofIds }) {
  async function createProof (proofId, proofData) {
    logger.info(`Verifier creating proof ${proofId}, proofData=${JSON.stringify(proofData)}`)
    await sleep(1000)
    const proof = await Proof.create(proofData)
    await saveProof(proofId, proof)
    return proof
  }

  async function buildProofReqAndMarkAsSent (proofId, proofData) {
    logger.debug(`Building proof request ${proofId}`)
    const proof = await Proof.create(proofData)
    const presentationRequest = await proof.getProofRequestMessage()
    const state1 = await proof.getState()
    expect(state1).toBe(VerifierStateType.PresentationRequestSet)

    await proof.markPresentationRequestMsgSent()
    const state2 = await proof.getState()
    expect(state2).toBe(VerifierStateType.PresentationRequestSent)

    await saveProof(proofId, proof)
    return presentationRequest
  }

  async function sendProofRequest (connectionId, proofId) {
    logger.debug(`Verifier sending proof request proofId=${proofId}, connectionId=${connectionId}`)
    const connection = await loadConnection(connectionId)
    const proof = await loadProof(proofId)
    await proof.requestProof(connection)
    const state = await proof.getState()
    await saveProof(proofId, proof)
    const proofRequestMessage = await proof.getProofRequestMessage()
    return { state, proofRequestMessage }
  }

  async function proofUpdate (proofId, connectionId) {
    const proof = await loadProof(proofId)
    const connection = await loadConnection(connectionId)
    const state = await proof.updateStateV2(connection)
    await saveProof(proofId, proof)
    return state
  }

  async function getState (proofId) {
    const proof = await loadProof(proofId)
    return await proof.getState()
  }

  async function getProofState (proofId) {
    const proof = await loadProof(proofId)
    const { proofState } = await proof.getProof()
    return proofState
  }

  async function listIds () {
    return listProofIds()
  }

  async function printInfo (connectionIds) {
    for (const id of connectionIds) {
      const state = await getState(id)
      logger.info(`Proof ${id} state=${state}`)
    }
  }

  async function getVcxProof (proofId) {
    logger.warn('Usage of getVcxProof is not recommended. You should use vcxagent-core API rather than work with vcx object directly.')
    return loadProof(proofId)
  }

  return {
    createProof,
    buildProofReqAndMarkAsSent,
    sendProofRequest,
    proofUpdate,
    getVcxProof,

    listIds,
    printInfo,
    getState,
    getProofState
  }
}
