const { filterOffersByAttr } = require('../utils/credentials')
const { filterOffersBySchema } = require('../utils/credentials')
const {
  Connection,
  StateType,
  Credential
} = require('@absaoss/node-vcx-wrapper')
const { pollFunction } = require('../common')

module.exports.createServiceCredHolder = function createServiceCredHolder (logger, loadConnection, storeCredHolder, loadCredHolder) {
  // todo: start storing credential objects...

  async function _getOffers (connection, filter, attemptsThreshold, timeoutMs) {
    async function findSomeCredOffer () {
      let offers = await Credential.getOffers(connection)
      if (filter && filter.schemaIdRegex) {
        offers = filterOffersBySchema(offers, filter.schemaIdRegex)
      }
      if (filter && filter.attrRegex) {
        offers = filterOffersByAttr(offers, filter.attrRegex)
      }
      if (offers.length === 0) {
        return { result: undefined, isFinished: false }
      } else {
        return { result: offers, isFinished: true }
      }
    }

    const [error, offers] = await pollFunction(findSomeCredOffer, 'Get credential offer', logger, attemptsThreshold, timeoutMs)
    if (error) {
      throw Error(`Couldn't get credential offers. ${error}`)
    }
    return offers
  }

  async function _progressCredentialToState (credential, connection, credentialStateTarget, attemptsThreshold, timeoutMs) {
    async function progressToAcceptedState () {
      if (await credential.updateStateV2(connection) !== credentialStateTarget) {
        return { result: undefined, isFinished: false }
      } else {
        return { result: null, isFinished: true }
      }
    }

    const [error] = await pollFunction(progressToAcceptedState, `Progress CredentialSM to state ${credentialStateTarget}`, logger, attemptsThreshold, timeoutMs)
    if (error) {
      throw Error(`Couldn't progress credential to Accepted state. ${error}`)
    }
  }

  async function waitForCredential (connectionName, credHolderName, attemptsThreshold, timeoutMs) {
    const connSerializedBefore = await loadConnection(connectionName)
    const connection = await Connection.deserialize(connSerializedBefore)

    const serCred = await loadCredHolder(credHolderName)
    const credential = await Credential.deserialize(serCred)

    await _progressCredentialToState(credential, connection, StateType.Accepted, attemptsThreshold, timeoutMs)
    logger.debug(`CredentialSM after credential was received:\n${JSON.stringify(await credential.serialize())}`)
    logger.info('Credential has been received.')

    const serCred1 = await credential.serialize()
    await storeCredHolder(credHolderName, serCred1)

    return getCredentialData(credHolderName)
  }

  async function getCredentialData (credHolderName) {
    const serCred = await loadCredHolder(credHolderName)

    return JSON.parse(
      Buffer.from(serCred.data.holder_sm.state.Finished.credential['credentials~attach'][0].data.base64, 'base64')
        .toString('utf8')
    )
  }

  async function createCredentialFromOfferAndSendRequest (connectionName, credHolderName, credentialOffer) {
    const connSerializedBefore = await loadConnection(connectionName)
    const connection = await Connection.deserialize(connSerializedBefore)

    const credential = await Credential.create({ sourceId: 'credential', offer: credentialOffer })

    const serCred1 = await credential.serialize()
    await storeCredHolder(credHolderName, serCred1)

    logger.info('After receiving credential offer, send credential request')
    await credential.sendRequest({ connection, payment: 0 })

    const serCred2 = await credential.serialize()
    await storeCredHolder(credHolderName, serCred2)

    logger.debug(`CredentialSM after credential request was sent:\n${JSON.stringify(serCred2)}`)
    return credential
  }

  async function waitForCredentialOffer (connectionName, credOfferFilter, attemptsThreshold, timeoutMs) {
    logger.info('Going to try fetch credential offer and receive credential.')
    const connSerializedBefore = await loadConnection(connectionName)
    const connection = await Connection.deserialize(connSerializedBefore)

    const offers = await _getOffers(connection, credOfferFilter, attemptsThreshold, timeoutMs)
    logger.info(`Found ${offers.length} credential offers.`)

    const pickedOffer = JSON.stringify(offers[0])
    logger.debug(`Picked credential offer = ${pickedOffer}`)

    return pickedOffer
  }

  async function waitForCredentialOfferAndAccept ({ connectionName, credHolderName, credOfferFilter, attemptsThreshold = 10, timeoutMs = 2000 }) {
    const pickedOffer = await waitForCredentialOffer(connectionName, credOfferFilter, attemptsThreshold, timeoutMs)
    return createCredentialFromOfferAndSendRequest(connectionName, credHolderName, pickedOffer)
  }

  async function waitForCredentialOfferAndAcceptAndProgress ({ connectionName, credHolderName, credOfferFilter, attemptsThreshold = 10, timeoutMs = 2000 }) {
    logger.info('Going to try fetch credential offer and receive credential.')
    await waitForCredentialOfferAndAccept({ connectionName, credHolderName, credOfferFilter, attemptsThreshold, timeoutMs })
    return waitForCredential(connectionName, credHolderName, attemptsThreshold, timeoutMs)
  }

  async function credentialUpdate (credName, connectionName) {
    const connSerializedBefore = await loadConnection(connectionName)
    const connection = await Connection.deserialize(connSerializedBefore)

    const serCred = await loadCredHolder(credName)
    const cred = await Credential.deserialize(serCred)

    const state = await cred.updateStateV2(connection)

    const serCredAfter = await cred.serialize()
    await storeCredHolder(credName, serCredAfter)

    return state
  }

  return {
    waitForCredentialOffer,
    createCredentialFromOfferAndSendRequest,
    waitForCredential,
    getCredentialData,
    waitForCredentialOfferAndAccept,
    waitForCredentialOfferAndAcceptAndProgress,
    credentialUpdate
  }
}
