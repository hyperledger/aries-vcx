const { filterOffersByAttr } = require('../utils/credentials')
const { filterOffersBySchema } = require('../utils/credentials')
const {
  Connection,
  StateType,
  Credential
} = require('@absaoss/node-vcx-wrapper')
const { pollFunction } = require('../common')

module.exports.createServiceCredHolder = function createServiceCredHolder (logger, loadConnection, storeHolderCredential, loadHolderCredential, listHolderCredentialIds) {
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

  async function _progressCredentialToState (credential, connection, credentialStateTarget, attemptsThreshold = 10, timeoutMs = 2000) {
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

  async function waitForCredential (connectionId, holderCredentialId, attemptsThreshold = 10, timeoutMs = 2000) {
    const connSerializedBefore = await loadConnection(connectionId)
    const connection = await Connection.deserialize(connSerializedBefore)

    const serCred = await loadHolderCredential(holderCredentialId)
    const credential = await Credential.deserialize(serCred)

    await _progressCredentialToState(credential, connection, StateType.Accepted, attemptsThreshold, timeoutMs)
    logger.debug(`CredentialSM after credential was received:\n${JSON.stringify(await credential.serialize())}`)
    logger.info('Credential has been received.')

    const serCred1 = await credential.serialize()
    await storeHolderCredential(holderCredentialId, serCred1)

    return getCredentialData(holderCredentialId)
  }

  async function getCredentialData (holderCredentialId) {
    const serCred = await loadHolderCredential(holderCredentialId)

    return JSON.parse(
      Buffer.from(serCred.data.holder_sm.state.Finished.credential['credentials~attach'][0].data.base64, 'base64')
        .toString('utf8')
    )
  }

  async function createCredentialFromOfferAndSendRequest (connectionId, holderCredentialId, credentialOffer) {
    const connSerializedBefore = await loadConnection(connectionId)
    const connection = await Connection.deserialize(connSerializedBefore)

    const credential = await Credential.create({ sourceId: 'credential', offer: credentialOffer })

    const serCred1 = await credential.serialize()
    await storeHolderCredential(holderCredentialId, serCred1)

    logger.info('After receiving credential offer, send credential request')
    await credential.sendRequest({ connection, payment: 0 })

    const serCred2 = await credential.serialize()
    await storeHolderCredential(holderCredentialId, serCred2)

    logger.debug(`CredentialSM after credential request was sent:\n${JSON.stringify(serCred2)}`)
    return credential
  }

  async function waitForCredentialOffer (connectionId, credOfferFilter = null, attemptsThreshold = 10, timeoutMs = 2000) {
    logger.info('Going to try fetch credential offer and receive credential.')
    const connSerializedBefore = await loadConnection(connectionId)
    const connection = await Connection.deserialize(connSerializedBefore)

    const offers = await _getOffers(connection, credOfferFilter, attemptsThreshold, timeoutMs)
    logger.info(`Found ${offers.length} credential offers.`)

    const pickedOffer = JSON.stringify(offers[0])
    logger.debug(`Picked credential offer = ${pickedOffer}`)

    return pickedOffer
  }

  async function waitForCredentialOfferAndAccept (connectionId, holderCredentialId, credOfferFilter = null, attemptsThreshold = 10, timeoutMs = 2000) {
    const pickedOffer = await waitForCredentialOffer(connectionId, credOfferFilter, attemptsThreshold, timeoutMs)
    return createCredentialFromOfferAndSendRequest(connectionId, holderCredentialId, pickedOffer)
  }

  async function waitForCredentialOfferAndAcceptAndProgress (connectionId, holderCredentialId, credOfferFilter = null, attemptsThreshold = 10, timeoutMs = 2000) {
    logger.info('Going to try fetch credential offer and receive credential.')
    await waitForCredentialOfferAndAccept(connectionId, holderCredentialId, credOfferFilter, attemptsThreshold, timeoutMs)
    return waitForCredential(connectionId, holderCredentialId, attemptsThreshold, timeoutMs)
  }

  async function credentialUpdate (holderCredentialId, connectionId) {
    const connSerializedBefore = await loadConnection(connectionId)
    const connection = await Connection.deserialize(connSerializedBefore)

    const serCred = await loadHolderCredential(holderCredentialId)
    const cred = await Credential.deserialize(serCred)

    const state = await cred.updateStateV2(connection)

    const serCredAfter = await cred.serialize()
    await storeHolderCredential(holderCredentialId, serCredAfter)

    return state
  }

  async function getState (credHolderId) {
    const serCred = await loadHolderCredential(credHolderId)
    const credential = await Credential.deserialize(serCred)
    return await credential.getState()
  }

  async function listIds () {
    return listHolderCredentialIds()
  }

  async function printInfo (credHolderIds) {
    for (const id of credHolderIds) {
      const state = await getState(id)
      logger.info(`Credential ${id} state=${state}`)
    }
  }

  return {
    waitForCredentialOffer,
    createCredentialFromOfferAndSendRequest,
    waitForCredential,
    getCredentialData,
    waitForCredentialOfferAndAccept,
    waitForCredentialOfferAndAcceptAndProgress,
    credentialUpdate,

    listIds,
    printInfo,
    getState
  }
}
