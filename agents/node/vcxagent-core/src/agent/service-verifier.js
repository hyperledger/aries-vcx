const {
  Proof
} = require('@absaoss/node-vcx-wrapper')

module.exports.createServiceVerifier = function createServiceVerifier(logger, loadConnection, storeProof, loadProof) {

  async function verifierCreateProof ({ proofName, sourceId, attrs, preds, name, revocationInterval }) {
    let proof = await Proof.create({ sourceId, attrs, preds, name, revocationInterval })
    let serProof = await proof.serialize();
    await storeProof(proofName, serProof)
    return proof
  }

  async function verifierCreateProofRequest (connectionName, proofName) {
    const serConnection = await loadConnection(connectionName)
    const connection = await Connection.deserialize(serConnection)

    const serProof = await loadProof(proofName)
    const proof = await IssuerCredential.deserialize(serProof)

    await proof.requestProof(connection)
    return proof.getProofRequestMessage()
  }

  return {
    verifierCreateProof,
    verifierCreateProofRequest
  }

}
