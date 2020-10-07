const { holderSelectCredentialsForProof } = require('../utils/proofs')
const {
  DisclosedProof,
  Connection
} = require('@absaoss/node-vcx-wrapper')

module.exports.createServiceProver = function createServiceProver (logger, loadConnection, storeDisclosedProof, loadDisclosedProof) {
  async function holderGetRequests (connectionName) {
    const serConnection = await loadConnection(connectionName)
    const connection = await Connection.deserialize(serConnection)

    return DisclosedProof.getRequests(connection)
  }

  async function buildDisclosedProof (disclosedProofName, proofRequest) {
    const disclosedProof = await DisclosedProof.create({ sourceId: 'proof', request: proofRequest })

    const serDisclosedProofAfter = await disclosedProof.serialize()
    await storeDisclosedProof(disclosedProofName, serDisclosedProofAfter)
  }

  async function selectCredentials (disclosedProofName) {
    const serDisclosedProof = await loadDisclosedProof(disclosedProofName)
    const disclosedProof = await DisclosedProof.deserialize(serDisclosedProof)

    return holderSelectCredentialsForProof(disclosedProof, logger)
  }

  async function generateProof (disclosedProofName, selectedCreds, selfAttestedAttrs) {
    const serDisclosedProof = await loadDisclosedProof(disclosedProofName)
    const disclosedProof = await DisclosedProof.deserialize(serDisclosedProof)

    await disclosedProof.generateProof({ selectedCreds, selfAttestedAttrs })

    const serDisclosedProofAfter = await disclosedProof.serialize()
    await storeDisclosedProof(disclosedProofName, serDisclosedProofAfter)
  }

  async function sendDisclosedProof (disclosedProofName, connectionName) {
    const serDisclosedProof = await loadDisclosedProof(disclosedProofName)
    const disclosedProof = await DisclosedProof.deserialize(serDisclosedProof)

    const serConnection = await loadConnection(connectionName)
    const connection = await Connection.deserialize(serConnection)

    await disclosedProof.sendProof(connection)
    const state = await disclosedProof.getState()

    const serDisclosedProofAfter = await disclosedProof.serialize()
    await storeDisclosedProof(disclosedProofName, serDisclosedProofAfter)

    return state
  }

  async function disclosedProofUpdate (disclosedProofName, connectionName) {
    const serDisclosedProof = await loadDisclosedProof(disclosedProofName)
    const disclosedProof = await DisclosedProof.deserialize(serDisclosedProof)

    const connSerializedBefore = await loadConnection(connectionName)
    const connection = await Connection.deserialize(connSerializedBefore)

    const state = await disclosedProof.updateStateV2(connection)

    const serDisclosedProofAfter = await disclosedProof.serialize()
    await storeDisclosedProof(disclosedProofName, serDisclosedProofAfter)

    return state
  }

  return {
    generateProof,
    selectCredentials,
    holderGetRequests,
    sendDisclosedProof,
    buildDisclosedProof,
    disclosedProofUpdate
  }
}
