const {
  Connection,
  StateType,
  Credential,
} = require('@absaoss/node-vcx-wrapper')
const { pollFunction } = require('../common')

module.exports.createServiceCredHolder = function createServiceCredHolder(logger, loadConnection, storeCredHolder, _loadCredHolder) {
  // todo: start storing credential objects...

  async function _getOffers (connection, attemptsThreshold, timeout) {
    async function findSomeCredOffer () {
      const offers = await Credential.getOffers(connection)
      if (offers.length === 0) {
        return { result: undefined, isFinished: false }
      } else {
        return { result: offers, isFinished: true }
      }
    }

    const [error, offers] = await pollFunction(findSomeCredOffer, 'Get credential offer', logger, attemptsThreshold, timeout)
    if (error) {
      throw Error(`Couldn't get credential offers. ${error}`)
    }
    return offers
  }

  async function _progressCredentialToState (credential, connection, credentialStateTarget, attemptsThreshold, timeout) {
    async function progressToAcceptedState () {
      if (await credential.updateStateV2(connection) !== credentialStateTarget) {
        return { result: undefined, isFinished: false }
      } else {
        return { result: null, isFinished: true }
      }
    }

    const [error] = await pollFunction(progressToAcceptedState, `Progress CredentialSM to state ${credentialStateTarget}`, logger, attemptsThreshold, timeout)
    if (error) {
      throw Error(`Couldn't progress credential to Accepted state. ${error}`)
    }
  }

  async function getCredentialOffers (connectionName) {
    const connSerializedBefore = await loadConnection(connectionName)
    const connection = await Connection.deserialize(connSerializedBefore)
    const offers = await Credential.getOffers(connection)
    logger.info(`Found ${offers.length} credential offers.`)
  }

  async function _waitForCredentialOfferAndAccept(connection, attemptsThreshold, timeout) {
    const offers = await _getOffers(connection, attemptsThreshold, timeout)
    logger.info(`Found ${offers.length} credential offers.`)

    const pickedOffer = JSON.stringify(offers[0])
    logger.debug(`Picked credential offer = ${pickedOffer}`)

    const credential = await Credential.create({ sourceId: 'credential', offer: pickedOffer })

    logger.info('After receiving credential offer, send credential request')
    await credential.sendRequest({ connection: connection, payment: 0 })

    const serCred = await credential.serialize()
    // storeCredential()
    logger.debug(`CredentialSM after credential request was sent:\n${JSON.stringify(serCred)}`)

    return credential
  }

  async function waitForCredentialOfferAndAccept ({ connectionName, attemptsThreshold = 10, timeout = 2000 }) {
    logger.info('Going to try fetch credential offer and receive credential.')
    const connSerializedBefore = await loadConnection(connectionName)
    const connection = await Connection.deserialize(connSerializedBefore)
    return _waitForCredentialOfferAndAccept(connection, attemptsThreshold, timeout)
  }

  async function waitForCredentialOfferAndAcceptAndProgress ({ connectionName, attemptsThreshold = 10, timeout = 2000 }) {
    logger.info('Going to try fetch credential offer and receive credential.')
    const connSerializedBefore = await loadConnection(connectionName)
    const connection = await Connection.deserialize(connSerializedBefore)
    let credential = await  _waitForCredentialOfferAndAccept(connection, attemptsThreshold, timeout)

    await _progressCredentialToState(credential, connection, StateType.Accepted, attemptsThreshold, timeout)
    logger.debug(`CredentialSM after credential was received:\n${JSON.stringify(await credential.serialize())}`)
    logger.info('Credential has been received.')

    return credential
  }

  return {
    waitForCredentialOfferAndAccept,
    waitForCredentialOfferAndAcceptAndProgress,
    getCredentialOffers,
  }

}
