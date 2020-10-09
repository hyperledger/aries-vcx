const {
  CredentialDef,
  Connection,
  StateType,
  IssuerCredential
} = require('@absaoss/node-vcx-wrapper')
const { pollFunction } = require('../common')

module.exports.createServiceCredIssuer = function createServiceCredIssuer (logger, loadConnection, loadCredDef, storeIssuerCredential, loadIssuerCredential, listIssuerCredentialIds) {
  // Creates issuer credential, sends offer
  async function sendOffer (issuerCredId, connectionId, credDefId, schemaAttrs) {
    logger.debug(`sendOffer >> issuerCredId=${issuerCredId} schemaAttrs=${schemaAttrs} credDefId=${credDefId} connectionId=${connectionId}`)
    const serConnection = await loadConnection(connectionId)
    const credDefSerialized = await loadCredDef(credDefId)
    if (!serConnection) {
      throw Error(`Connection ${connectionId} was not found.`)
    }
    if (!credDefSerialized) {
      throw Error(`Credential definition ${credDefId} was not found.`)
    }

    const connection = await Connection.deserialize(serConnection)
    const credDef = await CredentialDef.deserialize(credDefSerialized)

    logger.debug('Building issuer credential')
    const issuerCred = await IssuerCredential.create({
      attr: schemaAttrs,
      sourceId: 'alice_degree',
      credDefHandle: credDef.handle,
      credentialName: 'cred',
      price: '0'
    })

    logger.debug(`IssuerCredential created:\n${JSON.stringify(await issuerCred.serialize())}`)

    logger.info(`Per issuer credential ${issuerCredId}, sending cred offer to connection ${connectionId}`)
    await issuerCred.sendOffer(connection)

    const serIssuerCred = await issuerCred.serialize()
    await storeIssuerCredential(issuerCredId, serIssuerCred)

    logger.debug(`IssuerCredential after offer was sent:\n${JSON.stringify(serIssuerCred)}`)

    return issuerCred
  }

  // Assuming issuer credential is in state Requested, tries to send the actual credential
  async function sendCredential (issuerCredId, connectionId) {
    logger.debug('sendCredential >> ')
    const serConnection = await loadConnection(connectionId)
    const connection = await Connection.deserialize(serConnection)

    const serIssuerCredBefore = await loadIssuerCredential(issuerCredId)
    const issuerCred = await IssuerCredential.deserialize(serIssuerCredBefore)

    logger.info(`Sending credential ${issuerCredId} to ${connectionId}`)
    await issuerCred.sendCredential(connection)
    const state = await issuerCred.getState()

    const serIssuerCredAfter = await issuerCred.serialize()
    await storeIssuerCredential(issuerCredId, serIssuerCredAfter)

    logger.debug(`IssuerCredential after credential was sent:\n${JSON.stringify(serIssuerCredAfter)}`)
    return state
  }

  // Creates issuer credential, sends offer and waits to receive credential request
  async function sendOfferAndWaitForCredRequest (issuerCredId, connectionId, credDefId, schemaAttrs) {
    logger.debug('sendOfferAndWaitForCredRequest >> ')
    const issuerCred = await sendOffer(issuerCredId, connectionId, credDefId, schemaAttrs)

    logger.debug('Going to wait until credential request is received.')
    const serConnection = await loadConnection(connectionId)
    const connection = await Connection.deserialize(serConnection)
    await _progressIssuerCredentialToState(issuerCred, connection, StateType.RequestReceived, 10, 2000)

    const serIssuerCred2 = await issuerCred.serialize()
    await storeIssuerCredential(issuerCredId, serIssuerCred2)

    logger.debug(`IssuerCredential after credential request was received:\n${JSON.stringify(serIssuerCred2)}`)
  }

  // Assuming issuer credential is in state "Requested", sends credential and wait to receive Ack
  async function sendCredentialAndProgress (issuerCredId, connectionId) {
    logger.debug('sendCredentialAndProgress >> ')
    await sendCredential(issuerCredId, connectionId)

    const serConnection = await loadConnection(connectionId)
    const connection = await Connection.deserialize(serConnection)

    const serIssuerCredBefore = await loadIssuerCredential(issuerCredId)
    const issuerCred = await IssuerCredential.deserialize(serIssuerCredBefore)

    logger.info('Going to wait until counterparty accepts the credential.')
    await _progressIssuerCredentialToState(issuerCred, connection, StateType.Accepted, 10, 2000)

    const serIssuerCred = await issuerCred.serialize()
    await storeIssuerCredential(issuerCredId, serIssuerCred)

    logger.debug(`IssuerCredential after credential was issued:\n${JSON.stringify(serIssuerCred)}`)
  }

  // Creates issuer credential, sends offer and waits to receive credential request, then sends credential
  async function sendOfferAndCredential (issuerCredId, connectionId, credDefId, schemaAttrs) {
    logger.debug('sendOfferAndCredential >> ')
    await sendOfferAndWaitForCredRequest(issuerCredId, connectionId, credDefId, schemaAttrs)
    await sendCredentialAndProgress(issuerCredId, connectionId)
  }

  // Assuming the credential has been issued, tries to revoke the credential on ledger
  async function revokeCredential (issuerCredId) {
    const serIssuerCred = await loadIssuerCredential(issuerCredId)
    const issuerCred = await IssuerCredential.deserialize(serIssuerCred)

    logger.info('Revoking credential')
    await issuerCred.revokeCredential()
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
      throw Error(`Couldn't get credential offers. ${error}`)
    }
    return offers
  }

  async function credentialUpdate (issuerCredId, connectionId) {
    const connSerializedBefore = await loadConnection(connectionId)
    const connection = await Connection.deserialize(connSerializedBefore)

    const serIssuerCred = await loadIssuerCredential(issuerCredId)
    const issuerCred = await IssuerCredential.deserialize(serIssuerCred)

    const state = await issuerCred.updateStateV2(connection)

    const serIssuerCredAfter = await issuerCred.serialize()
    await storeIssuerCredential(issuerCredId, serIssuerCredAfter)

    return state
  }

  // deprecated
  async function credentialUpdateV1 (issuerCredId) {
    const serIssuerCred = await loadIssuerCredential(issuerCredId)
    const issuerCred = await IssuerCredential.deserialize(serIssuerCred)

    const state = await issuerCred.updateState()

    const serIssuerCredAfter = await issuerCred.serialize()
    await storeIssuerCredential(issuerCredId, serIssuerCredAfter)

    return state
  }

  async function getState (issuerCredentialId) {
    const serIssuerCredential = await loadIssuerCredential(issuerCredentialId)
    const issuerCredential = await IssuerCredential.deserialize(serIssuerCredential)
    return await issuerCredential.getState()
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

  return {
    sendOffer,
    sendOfferAndWaitForCredRequest,
    sendCredential,
    sendCredentialAndProgress,
    sendOfferAndCredential,
    revokeCredential,
    credentialUpdate,
    credentialUpdateV1,

    listIds,
    printInfo,
    getState
  }
}
