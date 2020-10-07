const { pollFunction } = require('../common')
const { holderSelectCredentialsForProof } = require('../utils/proofs')
const {
  DisclosedProof,
  Connection,
  StateType
} = require('@absaoss/node-vcx-wrapper')

module.exports.createServiceProver = function createServiceProver (logger, loadConnection, storeDisclosedProof, loadDisclosedProof) {
  async function _progressProofToState (proof, connection, targetStates, attemptsThreshold, timeoutMs) {
    async function progressToAcceptedState () {
      if (!Array.isArray(targetStates)) {
        throw Error('Argument targetStates should be array.')
      }
      const state = await proof.updateStateV2(connection)
      if (targetStates.includes(state)) {
        return { result: null, isFinished: true }
      } else {
        return { result: undefined, isFinished: false }
      }
    }

    const [error] = await pollFunction(progressToAcceptedState, `Progress CredentialSM to one of states ${JSON.stringify(targetStates)}`, logger, attemptsThreshold, timeoutMs)
    if (error) {
      throw Error(`Couldn't progress credential to Accepted state. ${error}`)
    }
  }

  async function _getProofRequests (connection, attemptsThreshold, timeoutMs) {
    async function findSomeRequests () {
      const requests = await DisclosedProof.getRequests(connection)
      if (requests.length === 0) {
        return { result: undefined, isFinished: false }
      } else {
        return { result: requests, isFinished: true }
      }
    }

    const [error, proofRequests] = await pollFunction(findSomeRequests, 'Get credential offer', logger, attemptsThreshold, timeoutMs)
    if (error) {
      throw Error(`Couldn't get credential offers. ${error}`)
    }
    return proofRequests
  }

  async function waitForProofRequests ({ connectionName, attemptsThreshold = 10, timeoutMs = 2000 }) {
    const connSerializedBefore = await loadConnection(connectionName)
    const connection = await Connection.deserialize(connSerializedBefore)

    const proofRequests = await _getProofRequests(connection, attemptsThreshold, timeoutMs)
    logger.info(`Found ${proofRequests.length} credential offers.`)

    return proofRequests
  }

  async function getProofRequests (connectionName) {
    const serConnection = await loadConnection(connectionName)
    const connection = await Connection.deserialize(serConnection)

    return DisclosedProof.getRequests(connection)
  }

  async function buildDisclosedProof (disclosedProofName, proofRequest) {
    const disclosedProof = await DisclosedProof.create({ sourceId: 'proof', request: JSON.stringify(proofRequest) })

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

  async function sendDisclosedProofAndProgress (disclosedProofName, connectionName) {
    await sendDisclosedProof(disclosedProofName, connectionName)

    const serDisclosedProof = await loadDisclosedProof(disclosedProofName)
    const disclosedProof = await DisclosedProof.deserialize(serDisclosedProof)

    const serConnection = await loadConnection(connectionName)
    const connection = await Connection.deserialize(serConnection)

    await _progressProofToState(disclosedProof, connection, [StateType.Accepted, StateType.None])
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
    getProofRequests,
    waitForProofRequests,
    sendDisclosedProof,
    sendDisclosedProofAndProgress,
    buildDisclosedProof,
    disclosedProofUpdate
  }
}
