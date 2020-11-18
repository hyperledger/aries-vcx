const {
  Proof
} = require('@hyperledger/node-vcx-wrapper')
const sleep = require('sleep-promise')

module.exports.createServiceVerifier = function createServiceVerifier ({ logger, loadConnection, saveProof, loadProof, listProofIds }) {
  async function createProof (proofId, proofData) {
    logger.info(`Verifier creating proof ${proofId}, proofData=${JSON.stringify(proofData)}`)
    await sleep(1000)
    const proof = await Proof.create(proofData)
    await saveProof(proofId, proof)
    return proof
  }

  async function sendProofRequest (connectionId, proofId) {
    const connection = await loadConnection(connectionId)
    const proof = await loadProof(proofId)
    logger.info(`Requesting proof. Loaded connection(handle=${connection.handle}) proof(handle=${proof.handle})`)
    await proof.requestProof(connection)
    logger.info(`Proof requested, getting state...`)
    const state = await proof.getState()
    logger.info(`Proof state is=${state}, saving proof`)
    const proofRequestMessage = await proof.getProofRequestMessage()
    logger.info(`getProofRequestMessage was called`)
    await saveProof(proofId, proof)
    logger.info(`Proof saved`)
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

  async function listIds () {
    return listProofIds()
  }

  async function printInfo (connectionIds) {
    for (const id of connectionIds) {
      const state = await getState(id)
      logger.info(`Proof ${id} state=${state}`)
    }
  }

  return {
    createProof,
    sendProofRequest,
    proofUpdate,

    listIds,
    printInfo,
    getState
  }
}
