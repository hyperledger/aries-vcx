const { provisionAgentInAgency, initRustapi } = require('./vcx-workflows')
const { Schema, CredentialDef, Connection, StateType, IssuerCredential, Credential, initVcxWithConfig, getLedgerAuthorAgreement, setActiveTxnAuthorAgreementMeta } = require('@absaoss/node-vcx-wrapper')
const { createStorageService } = require('./storage-service')
const { pollFunction, waitUntilAgencyIsReady, getRandomInt } = require('./common')

async function createVcxAgent ({ agentName, protocolType, agencyUrl, seed, webhookUrl, usePostgresWallet, logger, rustLogLevel }) {
  await initRustapi(rustLogLevel)

  await waitUntilAgencyIsReady(agencyUrl, logger)

  const storageService = await createStorageService(agentName)
  if (!await storageService.agentProvisionExists()) {
    const agentProvision = await provisionAgentInAgency(agentName, protocolType, agencyUrl, seed, webhookUrl, usePostgresWallet, logger)
    await storageService.saveAgentProvision(agentProvision)
  }
  const agentProvision = await storageService.loadAgentProvision()

  await initVcx()

  const connections = {}

  function keepConnectionInMemory (connectionName, connection) {
    connections[connectionName] = connection
  }

  async function initVcx (name = agentName) {
    logger.info(`Initializing VCX of ${name}`)
    logger.debug(`Using following agent provision to initialize VCX ${JSON.stringify(agentProvision, null, 2)}`)
    await initVcxWithConfig(JSON.stringify(agentProvision))
  }

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
    logger.info(`Create a new schema on the ledger: ${JSON.stringify(schemaData, null, 2)}`)

    const schema = await Schema.create(schemaData)
    const schemaId = await schema.getSchemaId()
    logger.info(`Created schema with id ${schemaId}`)
    return schema
  }

  async function createCredentialDefinition (schemaId, name) {
    logger.info('Create a new credential definition on the ledger')
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

  async function createConnection (connectionName) {
    logger.info(`InviterConnectionSM creating connection ${connectionName}`)
    const connection = await Connection.create({ id: connectionName })
    logger.debug(`InviterConnectionSM after created connection:\n${JSON.stringify(await connection.serialize())}`)
    await connection.connect('{}')
    await connection.updateState()
    return connection
  }

  async function storeConnection (connection, connectionName) {
    const connSerialized = await connection.serialize()
    await storageService.saveConnection(connectionName, connSerialized)
  }

  async function createInvite (connectionName) {
    const connection = await createConnection(connectionName)
    const invite = await connection.inviteDetails()
    return { invite, connection }
  }

  async function inviterConnectionCreateAndAccept (connectionName, cbInvitation, skipProgress) {
    const { invite, connection } = await createInvite(connectionName)
    logger.debug(`InviterConnectionSM after invitation was generated:\n${JSON.stringify(await connection.serialize())}`)
    if (cbInvitation) {
      cbInvitation(invite)
    }
    if (!skipProgress) {
      await _progressConnectionToAcceptedState(connection, 20, 2000)
      logger.debug(`InviterConnectionSM after connection was accepted:\n${JSON.stringify(await connection.serialize())}`)
    }
    await storeConnection(connection, connectionName)
    logger.info(`InviterConnectionSM has established connection ${connectionName}`)
    return { invite, connection }
  }

  async function inviteeConnectionCreateFromInvite (id, invite) {
    logger.info(`InviteeConnectionSM creating connection ${id} from connection invitation.`)
    const connection = await Connection.createWithInvite({ id, invite })
    logger.debug(`InviteeConnectionSM after created from invitation:\n${JSON.stringify(await connection.serialize())}`)
    await connection.connect({ data: '{}' })
    await connection.updateState()
    return connection
  }

  async function inviteeConnectionAcceptFromInvitation (connectionName, invite, skipProgress) {
    const connection = await inviteeConnectionCreateFromInvite(connectionName, invite)
    logger.debug(`InviteeConnectionSM sending connection request:\n${JSON.stringify(await connection.serialize())}`)
    if (!skipProgress) {
      await _progressConnectionToAcceptedState(connection, 20, 2000)
      logger.debug(`InviteeConnectionSM after connection was accepted:\n${JSON.stringify(await connection.serialize())}`)
    }
    const connSerialized = await connection.serialize()
    await storageService.saveConnection(connectionName, connSerialized)
    logger.info(`InviteeConnectionSM has established connection ${connectionName}`)
    return connection
  }

  async function _progressConnectionToAcceptedState (connection, attemptsThreshold, timeout) {
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
    await _progressConnectionToAcceptedState(connection, updateAttemptsThreshold, timeout)

    logger.info('Success! Connection was progressed to Accepted state.')
    const connSerialized = await connection.serialize()
    await storageService.saveConnection(connectionName, connSerialized)
    return connection
  }

  async function getCredentialOffers (connectionName) {
    const connSerializedBefore = await storageService.loadConnection(connectionName)
    const connection = await Connection.deserialize(connSerializedBefore)
    keepConnectionInMemory(connectionName, connection)
    await connection.updateState()
    const offers = await Credential.getOffers(connection)
    logger.info(`Found ${offers.length} credential offers.`)
  }

  async function credentialIssue (schemaAttrs, credDefName, connectionNameReceiver, revoke, skipProgress) {
    logger.info(`Going to issue credential from credential definition ${credDefName}`)
    const credDefSerialized = await storageService.loadCredentialDefinition(credDefName)
    const credDef = await CredentialDef.deserialize(credDefSerialized)

    const issuerCred = await IssuerCredential.create({
      attr: schemaAttrs,
      sourceId: 'alice_degree',
      credDefHandle: credDef.handle,
      credentialName: 'cred',
      price: '0'
    })

    logger.debug(`IssuerCredential created:\n${JSON.stringify(await issuerCred.serialize())}`)

    const connSerializedBefore = await storageService.loadConnection(connectionNameReceiver)
    const connectionToReceiver = await Connection.deserialize(connSerializedBefore)
    await connectionToReceiver.updateState()

    logger.info('Send credential offer to Alice')
    await issuerCred.sendOffer(connectionToReceiver)
    logger.debug(`IssuerCredential after offer was sent:\n${JSON.stringify(await issuerCred.serialize())}`)

    if (!skipProgress) {
      await _progressIssuerCredentialToState(issuerCred, StateType.RequestReceived, 10, 2000)
      logger.debug(`IssuerCredential after credential request was received:\n${JSON.stringify(await issuerCred.serialize())}`)
    }

    logger.info('Issue credential to Alice')
    await issuerCred.sendCredential(connectionToReceiver)
    logger.debug(`IssuerCredential after credential was sent:\n${JSON.stringify(await issuerCred.serialize())}`)

    const serCredential = await issuerCred.serialize()

    if (!skipProgress) {
      logger.info('Wait for alice to accept credential')
      await _progressIssuerCredentialToState(issuerCred, StateType.Accepted, 10, 2000)
      logger.debug(`IssuerCredential after credential was issued:\n${JSON.stringify(serCredential)}`)
      logger.info('Credential has been issued.')
    }

    if (revoke) {
      logger.info('Revoking credential')
      await issuerCred.revokeCredential()
    }

    return { serCred: serCredential, serConn: await connectionToReceiver.serialize() }
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

  async function _progressCredentialToState (credential, credentialStateTarget, attemptsThreshold, timeout) {
    async function progressToAcceptedState () {
      await credential.updateState()
      const credentialState = await credential.getState()
      if (credentialState !== credentialStateTarget) {
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

  async function _progressIssuerCredentialToState (issuerCredential, credentialStateTarget, attemptsThreshold, timeout) {
    async function progressToAcceptedState () {
      await issuerCredential.updateState()
      const credentialState = await issuerCredential.getState()
      if (credentialState !== credentialStateTarget) {
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

  async function waitForCredentialOfferAndAccept (connectionName, skipProgress = false, attemptsThreshold = 10, timeout = 2000) {
    logger.info('Going to try fetch credential offer and receive credential.')
    const connSerializedBefore = await storageService.loadConnection(connectionName)
    const connection = await Connection.deserialize(connSerializedBefore)

    await connection.updateState()
    const offers = await _getOffers(connection, attemptsThreshold, timeout)
    logger.info(`Found ${offers.length} credential offers.`)

    const pickedOffer = JSON.stringify(offers[0])
    logger.debug(`Picked credential offer = ${pickedOffer}`)

    const credential = await Credential.create({ sourceId: 'credential', offer: pickedOffer })
    logger.debug(`CredentialSM created from credential offer:\n${JSON.stringify(await credential.serialize())}`)

    logger.info('After receiving credential offer, send credential request')
    await credential.sendRequest({ connection, payment: 0 })
    logger.debug(`CredentialSM after credential request was sent:\n${JSON.stringify(await credential.serialize())}`)

    if (!skipProgress) {
      logger.info('Poll agency and accept credential offer from faber')
      await _progressCredentialToState(credential, StateType.Accepted, attemptsThreshold, timeout)
      logger.debug(`CredentialSM after credential was received:\n${JSON.stringify(await credential.serialize())}`)
      logger.info('Credential has been received.')
    }
  }

  function getInstitutionDid () {
    return agentProvision.institution_did
  }

  return {
    inviterConnectionCreateAndAccept,
    inviteeConnectionAcceptFromInvitation,
    connectionAutoupdate,
    acceptTaa,
    createCredentialDefinition,
    createSchema,
    connectionsList,
    credentialIssue,
    waitForCredentialOfferAndAccept,
    connectionPrintInfo,
    getCredentialOffers,
    getInstitutionDid,
    createConnection,
    initVcx,
    createInvite,
    inviteeConnectionCreateFromInvite,
    storeConnection
  }
}

module.exports.createVcxAgent = createVcxAgent
