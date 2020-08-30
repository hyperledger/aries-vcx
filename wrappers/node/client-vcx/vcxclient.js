const { getLedgerAuthorAgreement, setActiveTxnAuthorAgreementMeta } = require('../dist/src')
const {
  getRandomInt,
  initVcxWithProvisionedAgentConfig
} = require('./common')
const { Schema } = require('../dist/src/api/schema')
const { CredentialDef } = require('../dist/src/api/credential-def')
const { Connection } = require('../dist/src/api/connection')
const { StateType } = require('../dist/src')
const { IssuerCredential } = require('../dist/src/api/issuer-credential')
const { Credential } = require('../dist/src/api/credential')
const sleepPromise = require('sleep-promise')
const { pollFunction } = require('./common')

async function createVcxClient (storageService, logger) {
  const connections = {}

  function keepConnectionInMemory (connectionName, connection) {
    connections[connectionName] = connection
  }

  const agentProvision = await storageService.loadAgentProvision()
  logger.info(`Using following agent provision to initialize VCX ${JSON.stringify(agentProvision, null, 2)}`)
  await initVcxWithProvisionedAgentConfig(agentProvision)
  logger.info('VCX Client initialized.')

  async function acceptTaa () {
    const taa = await getLedgerAuthorAgreement()
    const taaJson = JSON.parse(taa)
    const utime = Math.floor(new Date() / 1000)
    await setActiveTxnAuthorAgreementMeta(taaJson.text, taaJson.version, null, Object.keys(taaJson.aml)[0], utime)
  }

  async function createSchema () {
    const version = `${getRandomInt(1, 101)}.${getRandomInt(1, 101)}.${getRandomInt(1, 101)}`
    const schemaData = {
      data: {
        attrNames: ['name', 'last_name', 'sex', 'date', 'degree', 'age'],
        name: 'FaberVcx',
        version
      },
      paymentHandle: 0,
      sourceId: `your-identifier-fabervcx-${version}`
    }
    logger.info(`#3 Create a new schema on the ledger: ${JSON.stringify(schemaData, null, 2)}`)

    const schema = await Schema.create(schemaData)
    const schemaId = await schema.getSchemaId()
    logger.info(`Created schema with id ${schemaId}`)
    return schema
  }

  async function createCredentialDefinition (schemaId, name) {
    logger.info('#4 Create a new credential definition on the ledger')
    const data = {
      name,
      paymentHandle: 0,
      revocationDetails: {
        supportRevocation: true,
        tailsFile: '/tmp/tails',
        maxCreds: 5
      },
      schemaId: schemaId,
      sourceId: 'testCredentialDefSourceId123'
    }
    const credDef = await CredentialDef.create(data)
    const credDefSer = await credDef.serialize()
    await storageService.saveCredentialDefinition(name, credDefSer)
    logger.info(`Created credentialDefinition ${name}.`)
    return credDef
  }

  async function connectionsList () {
    const connectionsNames = await storageService.listConnectionNames()
    for (const connectionsName of connectionsNames) {
      await connectionPrintInfo(connectionsName)
    }
  }

  async function connectionPrintInfo (connectionName) {
    const connSerialized = await storageService.loadConnection(connectionName)
    const connection = await Connection.deserialize(connSerialized)
    const connectionState = await connection.getState()
    logger.info(`Connection ${connectionName} state=${connectionState}`)
  }

  async function connectionCreate (connectionName) {
    const connection = await Connection.create({ id: connectionName })
    await connection.connect('{}')
    await connection.updateState()
    keepConnectionInMemory(connectionName, connection)
    const invitationString = await connection.inviteDetails()
    const connSerialized = await connection.serialize()
    await storageService.saveConnection(connectionName, connSerialized)
    return invitationString
  }

  async function connectionAccept (connectionName, invitationString) {
    const connection = await Connection.createWithInvite({ id: connectionName, invite: invitationString })
    keepConnectionInMemory(connectionName, connection)
    await connection.connect({ data: '{}' })
    const connectionState = await connection.getState()
    logger.info(`Created connection from invitation. Connection state = ${connectionState}`)
    const connSerialized = await connection.serialize()
    await storageService.saveConnection(connectionName, connSerialized)
  }

  async function _progressConnection (connection, attemptsThreshold, timeout) {
    async function progressToAcceptedState () {
      await connection.updateState()
      const connectionState = await connection.getState()
      if (connectionState !== StateType.Accepted) {
        return { result: undefined, isFinished: false }
      } else {
        return { result: null, isFinished: true }
      }
    }
    const [error] = await pollFunction(progressToAcceptedState, 'Progress connection', logger, attemptsThreshold, timeout)
    if (error) {
      throw Error(`Couldn't progress connection to Accepted state. ${error}`)
    }
  }

  async function connectionAutoupdate (connectionName, updateAttemptsThreshold = 10, timeout = 2000) {
    const connSerializedBefore = await storageService.loadConnection(connectionName)
    const connection = await Connection.deserialize(connSerializedBefore)
    keepConnectionInMemory(connectionName, connection)
    await _progressConnection(connection, updateAttemptsThreshold, timeout)

    logger.info('Success! Connection was progressed to Accepted state.')
    const connSerialized = await connection.serialize()
    await storageService.saveConnection(connectionName, connSerialized)
    return connection
  }

  async function connectionUpdate (connectionName) {
    const connSerializedBefore = await storageService.loadConnection(connectionName)
    const connection = await Connection.deserialize(connSerializedBefore)
    keepConnectionInMemory(connectionName, connection)
    await connection.updateState()
    const details = await connection.inviteDetails()
    const connSerializedAfter = await connection.serialize()
    await storageService.saveConnection(connectionName, connSerializedAfter)
    logger.info(`Created connection, invitation\n:${JSON.stringify(details)}`)
  }

  async function getCredentialOffers (connectionName) {
    const connSerializedBefore = await storageService.loadConnection(connectionName)
    const connection = await Connection.deserialize(connSerializedBefore)
    keepConnectionInMemory(connectionName, connection)
    await connection.updateState()
    const offers = await Credential.getOffers(connection)
    logger.info(`Found ${offers.length} credential offers.`)
  }

  async function credentialIssue (schemaAttrs, credDefName, connectionNameReceiver, revoke) {
    const credDefSerialized = await storageService.loadCredentialDefinition(credDefName)
    const credDef = await CredentialDef.deserialize(credDefSerialized)
    logger.info('#12 Create an IssuerCredential object using the schema and credential definition')

    const credentialForAlice = await IssuerCredential.create({
      attr: schemaAttrs,
      sourceId: 'alice_degree',
      credDefHandle: credDef.handle,
      credentialName: 'cred',
      price: '0'
    })

    const connSerializedBefore = await storageService.loadConnection(connectionNameReceiver)
    const connectionToReceiver = await Connection.deserialize(connSerializedBefore)

    logger.info('#13 Issue credential offer to alice')
    await credentialForAlice.sendOffer(connectionToReceiver)
    await credentialForAlice.updateState()

    logger.info('#14 Poll agency and wait for alice to send a credential request')
    let credentialState = await credentialForAlice.getState()
    while (credentialState !== StateType.RequestReceived) {
      await sleepPromise(2000)
      await credentialForAlice.updateState()
      credentialState = await credentialForAlice.getState()
    }

    logger.info('#17 Issue credential to alice')
    await credentialForAlice.sendCredential(connectionToReceiver)

    logger.info('#18 Wait for alice to accept credential')
    await credentialForAlice.updateState()
    credentialState = await credentialForAlice.getState()
    while (credentialState !== StateType.Accepted) {
      await sleepPromise(2000)
      await credentialForAlice.updateState()
      credentialState = await credentialForAlice.getState()
    }
    logger.info('Credential was issued.')
    if (revoke) {
      logger.info('#18.5 Revoking credential')
      await credentialForAlice.revokeCredential()
    }
  }

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

  async function _progressCredential (credential, attemptsThreshold, timeout) {
    async function progressToAcceptedState () {
      await credential.updateState()
      const credentialState = await credential.getState()
      if (credentialState !== StateType.Accepted) {
        return { result: undefined, isFinished: false }
      } else {
        return { result: null, isFinished: true }
      }
    }
    const [error] = await pollFunction(progressToAcceptedState, 'Get credential', logger, attemptsThreshold, timeout)
    if (error) {
      throw Error(`Couldn't progress credential to Accepted state. ${error}`)
    }
  }

  async function waitForCredentialOfferAndAccept (connectionName, attemptsThreshold = 10, timeout = 2000) {
    const connSerializedBefore = await storageService.loadConnection(connectionName)
    const connection = await Connection.deserialize(connSerializedBefore)

    const offers = await _getOffers(connection, attemptsThreshold, timeout)
    logger.info(`Credential offers = ${JSON.stringify(offers)}`)
    const credential = await Credential.create({ sourceId: 'credential', offer: JSON.stringify(offers[0]) })

    logger.info('#15 After receiving credential offer, send credential request')
    await credential.sendRequest({ connection, payment: 0 })

    logger.info('#16 Poll agency and accept credential offer from faber')
    await _progressCredential(credential, attemptsThreshold, timeout)
  }

  return {
    connectionAccept,
    connectionAutoupdate,
    acceptTaa,
    createCredentialDefinition,
    createSchema,
    connectionCreate,
    connectionUpdate,
    connectionsList,
    credentialIssue,
    waitForCredentialOfferAndAccept,
    connectionPrintInfo,
    getCredentialOffers
  }
}

module.exports.createVcxClient = createVcxClient
