const {
  IssuerStateType,
  IssuerCredential
} = require('@hyperledger/node-vcx-wrapper')
const { pollFunction } = require('../common')
const assert = require('assert')

module.exports.createServiceCredIssuer = function createServiceCredIssuer ({ logger, loadConnection, loadCredDef, loadRevReg, saveIssuerCredential, loadIssuerCredential, listIssuerCredentialIds, issuerDid }) {
  async function buildOfferAndMarkAsSent (issuerCredId, credDefId, revRegId, schemaAttrs) {
    const credDef = await loadCredDef(credDefId)
    const revReg = await loadRevReg(revRegId)
    logger.debug('Building issuer credential')
    const issuerCred = await IssuerCredential.create('alice_degree')
    logger.info(`Per issuer credential ${issuerCredId}, building cred offer.`)
    await issuerCred.buildCredentialOfferMsgV2({
      credDef,
      revReg,
      attr: schemaAttrs
    })
    const state1 = await issuerCred.getState()
    assert.equal(state1, IssuerStateType.OfferSet)
    const credOfferMsg = await issuerCred.getCredentialOfferMsg()
    await issuerCred.markCredentialOfferMsgSent()
    const state2 = await issuerCred.getState()
    assert.equal(state2, IssuerStateType.OfferSent)
    await saveIssuerCredential(issuerCredId, issuerCred)

    return credOfferMsg
  }

  async function sendOfferV2 (issuerCredId, revRegId, connectionId, credDefId, schemaAttrs) {
    assert(revRegId)
    const connection = await loadConnection(connectionId)
    const credDef = await loadCredDef(credDefId)
    const revReg = revRegId ? await loadRevReg(revRegId) : undefined
    logger.debug('Building issuer credential')
    const issuerCred = await IssuerCredential.create('alice_degree')
    logger.info(`Per issuer credential ${issuerCredId}, sending cred offer to connection ${connectionId}`)
    await issuerCred.buildCredentialOfferMsgV2({
      credDef,
      attr: schemaAttrs,
      revReg
    })
    const state1 = await issuerCred.getState()
    assert.equal(state1, IssuerStateType.OfferSet)
    await issuerCred.sendOfferV2(connection)
    const state2 = await issuerCred.getState()
    assert.equal(state2, IssuerStateType.OfferSent)
    await saveIssuerCredential(issuerCredId, issuerCred)
  }

  async function sendCredential (issuerCredId, connectionId) {
    const connection = await loadConnection(connectionId)
    const issuerCred = await loadIssuerCredential(issuerCredId)
    logger.info(`Sending credential '${issuerCredId}' to ${connectionId}`)
    await issuerCred.sendCredential(connection)
    const state = await issuerCred.getState()
    await saveIssuerCredential(issuerCredId, issuerCred)
    return state
  }

  async function waitForCredentialAck (issuerCredId, connectionId, attemptThreshold = 20, timeoutMs = 500) {
    const connection = await loadConnection(connectionId)
    const issuerCred = await loadIssuerCredential(issuerCredId)
    logger.info(`Waiting for ack on issuer credential '${issuerCredId}' with connection ${connectionId}`)
    await _progressIssuerCredentialToState(issuerCred, connection, IssuerStateType.Finished, attemptThreshold, timeoutMs)
    await saveIssuerCredential(issuerCredId, issuerCred)
  }

  async function sendOfferAndWaitForCredRequest (issuerCredId, revRegId, connectionId, credDefId, schemaAttrs, attemptThreshold = 20, timeoutMs = 500) {
    await sendOfferV2(issuerCredId, revRegId, connectionId, credDefId, schemaAttrs)
    const issuerCred = await loadIssuerCredential(issuerCredId)
    const connection = await loadConnection(connectionId)
    logger.debug('Going to wait until credential request is received.')
    await _progressIssuerCredentialToState(issuerCred, connection, IssuerStateType.RequestReceived, attemptThreshold, timeoutMs)
    await saveIssuerCredential(issuerCredId, issuerCred)
  }

  async function sendCredentialAndProgress (issuerCredId, connectionId, attemptThreshold = 20, timeoutMs = 500) {
    await sendCredential(issuerCredId, connectionId)
    const connection = await loadConnection(connectionId)
    const issuerCred = await loadIssuerCredential(issuerCredId)
    logger.info('Going to wait until counterparty accepts the credential.')
    await _progressIssuerCredentialToState(issuerCred, connection, IssuerStateType.Finished, attemptThreshold, timeoutMs)
    await saveIssuerCredential(issuerCredId, issuerCred)
  }

  async function sendOfferAndCredential (issuerCredId, revRegId, connectionId, credDefId, schemaAttrs) {
    await sendOfferAndWaitForCredRequest(issuerCredId, revRegId, connectionId, credDefId, schemaAttrs)
    await sendCredentialAndProgress(issuerCredId, connectionId)
  }

  async function revokeCredentialLocal (issuerCredId) {
    const issuerCred = await loadIssuerCredential(issuerCredId)
    logger.info(`Revoking credential ${issuerCredId}`)
    await issuerCred.revokeCredentialLocal()
  }

  async function getRevRegId (issuerCredId) {
    const issuerCred = await loadIssuerCredential(issuerCredId)
    logger.info(`Getting rev reg id for credential ${issuerCredId}`)
    return issuerCred.getRevRegId()
  }

  async function _progressIssuerCredentialToState (issuerCredential, connection, credentialStateTarget, attemptsThreshold, timeoutMs) {
    async function progressToAcceptedState () {
      if (await issuerCredential.updateStateV2(connection) !== credentialStateTarget) {
        return { result: undefined, isFinished: false }
      } else {
        return { result: null, isFinished: true }
      }
    }

    const [error, offers] = await pollFunction(progressToAcceptedState, `Progress IssuerCredentialSM to state ${credentialStateTarget}`, logger, attemptsThreshold, timeoutMs)
    if (error) {
      throw Error(`Couldn't find suitable message to progress issuerCredential to state ${credentialStateTarget}. ${error}`)
    }
    return offers
  }

  async function credentialUpdate (issuerCredId, connectionId) {
    const connection = await loadConnection(connectionId)
    const issuerCred = await loadIssuerCredential(issuerCredId)
    const state = await issuerCred.updateStateV2(connection)
    await saveIssuerCredential(issuerCredId, issuerCred)
    return state
  }

  async function getState (issuerCredentialId) {
    const issuerCredential = await loadIssuerCredential(issuerCredentialId)
    return issuerCredential.getState()
  }

  async function listIds () {
    return listIssuerCredentialIds()
  }

  async function printInfo (issuerCredentialIds) {
    for (const id of issuerCredentialIds) {
      const state = await getState(id)
      logger.info(`IssuerCredential ${id} state=${state}`)
    }
  }

  async function getVcxCredentialIssuer (issuerCredentialId) {
    logger.warn('Usage of getVcxCredentialIssuer is not recommended. You should use vcxagent-core API rather than work with vcx object directly.')
    return loadIssuerCredential(issuerCredentialId)
  }

  return {
    sendOfferV2,
    buildOfferAndMarkAsSent,
    sendOfferAndWaitForCredRequest,
    sendCredential,
    waitForCredentialAck,
    sendCredentialAndProgress,
    sendOfferAndCredential,
    revokeCredentialLocal,
    credentialUpdate,
    getVcxCredentialIssuer,

    listIds,
    printInfo,
    getState,
    getRevRegId
  }
}
