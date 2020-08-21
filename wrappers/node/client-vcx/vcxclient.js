const { getLedgerAuthorAgreement, setActiveTxnAuthorAgreementMeta } = require('../dist/src')
const {
  getRandomInt,
  initVcxWithProvisionedAgentConfig
} = require('../common/common')
const { Schema } = require('../dist/src/api/schema')
const { CredentialDef } = require('../dist/src/api/credential-def')
const { Connection } = require('../dist/src/api/connection')
const { StateType } = require('../dist/src')
const { IssuerCredential } = require('../dist/src/api/issuer-credential')
const { Credential } = require('../dist/src/api/credential')
const sleepPromise = require('sleep-promise')

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

  async function connectionAutoupdate (connectionName, updateAttemptsThreshold = 10, timeout = 2000) {
    const connSerializedBefore = await storageService.loadConnection(connectionName)
    const connection = await Connection.deserialize(connSerializedBefore)
    keepConnectionInMemory(connectionName, connection)
    let connectionState = await connection.getState()
    let attempts = 0
    while (connectionState !== StateType.Accepted) {
      await connection.updateState()
      connectionState = await connection.getState()
      attempts += 1
      logger.info(`Connection autoupdate [${attempts}/${updateAttemptsThreshold}]. State= ${connectionState}`)
      if (attempts === updateAttemptsThreshold) {
        logger.warn('Connection was not progressed to Accepted state.')
        return
      }
      await sleepPromise(timeout)
    }
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

  async function credentialIssue (schemaAttrs, credDefName, connectionNameReceiver, doRevoke) {
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
    if (doRevoke) {
      logger.info('#18.5 Revoking credential')
      await credentialForAlice.revokeCredential()
    }
  }

  async function waitForCredentialOfferAndAccept (connectionName, attemptsThreshold = 10, timeout = 2000) {
    const connSerializedBefore = await storageService.loadConnection(connectionName)
    const connection = await Connection.deserialize(connSerializedBefore)

    let offers = await Credential.getOffers(connection)
    let attemptsFetchCredOffers = 1
    while (offers.length === 0) {
      await sleepPromise(timeout)
      if (attemptsFetchCredOffers > attemptsThreshold) {
        throw Error(`Tried to fetch credential offers ${attemptsFetchCredOffers} times, but none found.`)
      }
      offers = await Credential.getOffers(connection)
      attemptsFetchCredOffers += 1
    }
    logger.info(`Credential offers = ${JSON.stringify(offers)}`)
    // Create a credential object from the credential offer
    const credential = await Credential.create({ sourceId: 'credential', offer: JSON.stringify(offers[0]) })

    logger.info('#15 After receiving credential offer, send credential request')
    await credential.sendRequest({ connection, payment: 0 })

    logger.info('#16 Poll agency and accept credential offer from faber')
    let credentialState = await credential.getState()
    let attemptsProgress = 1
    while (credentialState !== StateType.Accepted) {
      await sleepPromise(timeout)
      if (attemptsProgress > attemptsThreshold) {
        throw Error(`Tried to fetch credential offers ${attemptsProgress} times, but none found.`)
      }
      await credential.updateState()
      credentialState = await credential.getState()
      attemptsProgress += 1
    }
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
