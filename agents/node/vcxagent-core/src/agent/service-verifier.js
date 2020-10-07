const {
  Proof,
  Connection
} = require('@absaoss/node-vcx-wrapper')
const sleep = require('sleep-promise')

module.exports.createServiceVerifier = function createServiceVerifier (logger, loadConnection, storeProof, loadProof) {
  async function createProof ({ proofName, proofData }) {
    logger.info(`Verifier creating proof ${proofName}, proofData=${JSON.stringify(proofData)}`)
    await sleep(1000)
    const proof = await Proof.create(proofData)
    const serProof = await proof.serialize()
    await storeProof(proofName, serProof)
    return proof
  }

  async function sendProofRequest (connectionName, proofName) {
    const serConnection = await loadConnection(connectionName)
    const connection = await Connection.deserialize(serConnection)

    const serProof = await loadProof(proofName)
    const proof = await Proof.deserialize(serProof)

    await proof.requestProof(connection)
    const state = await proof.getState()

    const serProofAfter = await proof.serialize()
    await storeProof(proofName, serProofAfter)

    const proofRequestMessage = proof.getProofRequestMessage()
    return { state, proofRequestMessage }
  }

  async function proofUpdate (proofName, connectionName) {
    const serProof = await loadProof(proofName)
    const proof = await Proof.deserialize(serProof)

    const connSerializedBefore = await loadConnection(connectionName)
    const connection = await Connection.deserialize(connSerializedBefore)

    const state = await proof.updateStateV2(connection)

    const serProofAfter = await proof.serialize()
    await storeProof(proofName, serProofAfter)

    return state
  }

  return {
    createProof,
    sendProofRequest,
    proofUpdate
  }
}
