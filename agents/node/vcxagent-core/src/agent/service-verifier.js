const {
  Proof,
  Connection
} = require('@absaoss/node-vcx-wrapper')
const sleep = require('sleep-promise')

module.exports.createServiceVerifier = function createServiceVerifier (logger, loadConnection, storeProof, loadProof, listProofIds) {
  async function createProof ({ proofId, proofData }) {
    logger.info(`Verifier creating proof ${proofId}, proofData=${JSON.stringify(proofData)}`)
    await sleep(1000)
    const proof = await Proof.create(proofData)
    const serProof = await proof.serialize()
    await storeProof(proofId, serProof)
    return proof
  }

  async function sendProofRequest (connectionId, proofId) {
    const serConnection = await loadConnection(connectionId)
    const connection = await Connection.deserialize(serConnection)

    const serProof = await loadProof(proofId)
    const proof = await Proof.deserialize(serProof)

    await proof.requestProof(connection)
    const state = await proof.getState()

    const serProofAfter = await proof.serialize()
    await storeProof(proofId, serProofAfter)

    const proofRequestMessage = proof.getProofRequestMessage()
    return { state, proofRequestMessage }
  }

  async function proofUpdate (proofId, connectionId) {
    const serProof = await loadProof(proofId)
    const proof = await Proof.deserialize(serProof)

    const connSerializedBefore = await loadConnection(connectionId)
    const connection = await Connection.deserialize(connSerializedBefore)

    const state = await proof.updateStateV2(connection)

    const serProofAfter = await proof.serialize()
    await storeProof(proofId, serProofAfter)

    return state
  }

  async function getState (proofId) {
    const serProof = await loadProof(proofId)
    const proof = await Proof.deserialize(serProof)
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
