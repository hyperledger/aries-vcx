const {
  DisclosedProof
} = require('@absaoss/node-vcx-wrapper')
const { pollFunction } = require('../common')

module.exports.createServiceProver = function createServiceProver(logger, loadConnection, storeDisclosedProof, loadDisclosedProof) {

  // async function holderCreateProofFromRequest (proofName, request, sourceId = '123') {
  //   const disclosedProof = DisclosedProof.create({ request, sourceId })
  //   const serDisclosedProof = await disclosedProof.serialize()
  // }
  //
  // async function holderGetRequests (connectionName) {
  //   const serConnection = await loadConnection(connectionName)
  //   const connection = await Connection.deserialize(serConnection)
  //
  //   return DisclosedProof.getRequests(connection)
  // }

  // return {
  //   holderGetRequests,
  //   holderCreateProofFromRequest
  // }

  return {}
}
