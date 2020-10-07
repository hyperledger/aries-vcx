const {
  CredentialDef,
  Connection,
  StateType,
  IssuerCredential
} = require('@absaoss/node-vcx-wrapper')
const { pollFunction } = require('../common')

module.exports.createServiceCredIssuer = function createServiceCredIssuer (logger, loadConnection, loadCredDef, storeCredential, loadCredential) {
  // Creates issuer credential, sends offer
  async function sendOffer ({ issuerCredName, connectionName, credDefName, schemaAttrs }) {
    logger.debug(`sendOffer >> issuerCredName=${issuerCredName} schemaAttrs=${schemaAttrs} credDefName=${credDefName} connectionName=${connectionName}`)
    const serConnection = await loadConnection(connectionName)
    const credDefSerialized = await loadCredDef(credDefName)
    if (!serConnection) {
      throw Error(`Connection ${connectionName} was not found.`)
    }
    if (!credDefSerialized) {
      throw Error(`Credential definition ${credDefName} was not found.`)
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

    logger.info(`Per issuer credential ${issuerCredName}, sending cred offer to connection ${connectionName}`)
    await issuerCred.sendOffer(connection)

    const serIssuerCred = await issuerCred.serialize()
    await storeCredential(issuerCredName, serIssuerCred)

    logger.debug(`IssuerCredential after offer was sent:\n${JSON.stringify(serIssuerCred)}`)

    return issuerCred
  }

  // Assuming issuer credential is in state Requested, tries to send the actual credential
  async function sendCredential ({ issuerCredName, connectionName }) {
    logger.debug('sendCredential >> ')
    const serConnection = await loadConnection(connectionName)
    const connection = await Connection.deserialize(serConnection)

    const serIssuerCredBefore = await loadCredential(issuerCredName)
    const issuerCred = await IssuerCredential.deserialize(serIssuerCredBefore)

    logger.info(`Sending credential ${issuerCredName} to ${connectionName}`)
    await issuerCred.sendCredential(connection)
    const state = await issuerCred.getState()

    const serIssuerCredAfter = await issuerCred.serialize()
    await storeCredential(issuerCredName, serIssuerCredAfter)

    logger.debug(`IssuerCredential after credential was sent:\n${JSON.stringify(serIssuerCredAfter)}`)
    return state
  }

  // Creates issuer credential, sends offer and waits to receive credential request
  async function sendOfferAndWaitForCredRequest ({ issuerCredName, connectionName, credDefName, schemaAttrs }) {
    logger.debug('sendOfferAndWaitForCredRequest >> ')
    const issuerCred = await sendOffer({ issuerCredName, schemaAttrs, credDefName, connectionName })

    logger.debug('Going to wait until credential request is received.')
    const serConnection = await loadConnection(connectionName)
    const connection = await Connection.deserialize(serConnection)
    await _progressIssuerCredentialToState(issuerCred, connection, StateType.RequestReceived, 10, 2000)

    const serIssuerCred2 = await issuerCred.serialize()
    await storeCredential(issuerCredName, serIssuerCred2)

    logger.debug(`IssuerCredential after credential request was received:\n${JSON.stringify(serIssuerCred2)}`)
  }

  // Assuming issuer credential is in state "Requested", sends credential and wait to receive Ack
  async function sendCredentialAndProgress ({ issuerCredName, connectionName }) {
    logger.debug('sendCredentialAndProgress >> ')
    await sendCredential({ issuerCredName, connectionName })

    const serConnection = await loadConnection(connectionName)
    const connection = await Connection.deserialize(serConnection)

    const serIssuerCredBefore = await loadCredential(issuerCredName)
    const issuerCred = await IssuerCredential.deserialize(serIssuerCredBefore)

    logger.info('Going to wait until counterparty accepts the credential.')
    await _progressIssuerCredentialToState(issuerCred, connection, StateType.Accepted, 10, 2000)

    const serIssuerCred = await issuerCred.serialize()
    await storeCredential(issuerCredName, serIssuerCred)

    logger.debug(`IssuerCredential after credential was issued:\n${JSON.stringify(serIssuerCred)}`)
  }

  // Creates issuer credential, sends offer and waits to receive credential request, then sends credential
  async function sendOfferAndCredential ({ issuerCredName, connectionName, credDefName, schemaAttrs }) {
    logger.debug('sendOfferAndCredential >> ')
    await sendOfferAndWaitForCredRequest({ issuerCredName, connectionName, credDefName, schemaAttrs })
    await sendCredentialAndProgress({ issuerCredName, connectionName })
  }

  // Assuming the credential has been issued, tries to revoke the credential on ledger
  async function revokeCredential (issuerCredName) {
    const serIssuerCred = await loadCredential(issuerCredName)
    const issuerCred = await IssuerCredential.deserialize(serIssuerCred)

    logger.info('Revoking credential')
    await issuerCred.revokeCredential()
  }

  async function _progressIssuerCredentialToState (issuerCredential, connection, credentialStateTarget, attemptsThreshold, timeout) {
    async function progressToAcceptedState () {
      if (await issuerCredential.updateStateV2(connection) !== credentialStateTarget) {
        return { result: undefined, isFinished: false }
      } else {
        return { result: null, isFinished: true }
      }
    }

    const [error, offers] = await pollFunction(progressToAcceptedState, `Progress IssuerCredentialSM to state ${credentialStateTarget}`, logger, attemptsThreshold, timeout)
    if (error) {
      throw Error(`Couldn't get credential offers. ${error}`)
    }
    return offers
  }

  async function credentialUpdate (issuerCredName, connectionName) {
    const connSerializedBefore = await loadConnection(connectionName)
    const connection = await Connection.deserialize(connSerializedBefore)

    const serIssuerCred = await loadCredential(issuerCredName)
    const issuerCred = await IssuerCredential.deserialize(serIssuerCred)

    const state = await issuerCred.updateStateV2(connection)

    const serIssuerCredAfter = await issuerCred.serialize()
    await storeCredential(issuerCredName, serIssuerCredAfter)

    return state
  }

  // deprecated
  async function credentialUpdateV1 (issuerCredName) {
    const serIssuerCred = await loadCredential(issuerCredName)
    const issuerCred = await IssuerCredential.deserialize(serIssuerCred)

    const state = await issuerCred.updateState()

    const serIssuerCredAfter = await issuerCred.serialize()
    await storeCredential(issuerCredName, serIssuerCredAfter)

    return state
  }

  return {
    sendOffer,
    sendOfferAndWaitForCredRequest,
    sendCredential,
    sendCredentialAndProgress,
    sendOfferAndCredential,
    revokeCredential,
    credentialUpdate,
    credentialUpdateV1
  }
}
