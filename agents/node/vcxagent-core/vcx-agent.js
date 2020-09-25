const { getMessagesForPwDid } = require('./messages')
const { provisionAgentInAgency, initRustapi } = require('./vcx-workflows')
const {
  Schema,
  CredentialDef,
  Connection,
  StateType,
  IssuerCredential,
  Credential,
  Proof,
  DisclosedProof,
  initVcxWithConfig,
  initVcxCore,
  openVcxWallet,
  openVcxPool,
  vcxUpdateWebhookUrl,
  getLedgerAuthorAgreement,
  setActiveTxnAuthorAgreementMeta
} = require('@absaoss/node-vcx-wrapper')
const { createStorageService } = require('./storage-service')
const { pollFunction, waitUntilAgencyIsReady, getRandomInt } = require('./common')
const { getFaberSchemaData } = require('./test/data')

async function createVcxAgent ({ agentName, protocolType, agencyUrl, seed, webhookUrl, usePostgresWallet, logger }) {
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

  /**
   * Initializes libvcx configuration, open pool, open wallet, set webhook url if present in agent provison
   */
  async function initVcxOld (name = agentName) {
    logger.info(`Initializing VCX agent ${name}`)
    logger.debug(`Using following agent provision to initialize VCX ${JSON.stringify(agentProvision, null, 2)}`)
    await initVcxWithConfig(JSON.stringify(agentProvision))
  }

  /**
   * Performs the same as initVcxOld, except for the fact it ignores webhook_url in agent provision. You have to
   * update webhook_url by calling function vcxUpdateWebhookUrl.
   */
  async function initVcx (name = agentName) {
    logger.info(`Initializing VCX agent ${name}`)
    logger.debug(`Using following agent provision to initialize VCX settings ${JSON.stringify(agentProvision, null, 2)}`)
    await initVcxCore(JSON.stringify(agentProvision))
    logger.debug('Opening wallet and pool')
    const promises = []
    promises.push(openVcxPool())
    promises.push(openVcxWallet())
    await Promise.all(promises)
    logger.debug('LibVCX fully initialized')
  }

  async function acceptTaa () {
    const taa = await getLedgerAuthorAgreement()
    const taaJson = JSON.parse(taa)
    const utime = Math.floor(new Date() / 1000)
    await setActiveTxnAuthorAgreementMeta(taaJson.text, taaJson.version, null, Object.keys(taaJson.aml)[0], utime)
  }

  async function updateWebhookUrl (webhookUrl) {
    logger.info(`Updating webhook url to ${webhookUrl}`)
    await vcxUpdateWebhookUrl({ webhookUrl })
  }

  async function createSchema (_schemaData) {
    const schemaData = _schemaData || getFaberSchemaData()
    logger.info(`Creating a new schema on the ledger: ${JSON.stringify(schemaData, null, 2)}`)

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
      if (await connection.updateState() !== StateType.Accepted) {
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

  async function sendMessage (connectionName, payload) {
    const serConnection = await storageService.loadConnection(connectionName)
    const connection = await Connection.deserialize(serConnection)
    await connection.sendMessage({msg: payload, msg_title: "msg_title", msg_type: "msg_type", ref_msg_id: "ref_msg_id"})
  }

  async function getMessages (connectionName, filterStatuses= [], filterUids= []) {
    const serConnection = await storageService.loadConnection(connectionName)
    const pwDid = serConnection.data.pw_did
    return getMessagesForPwDid(pwDid, [], filterStatuses, filterUids)
  }

  async function getCredentialOffers (connectionName) {
    const connSerializedBefore = await storageService.loadConnection(connectionName)
    const connection = await Connection.deserialize(connSerializedBefore)
    keepConnectionInMemory(connectionName, connection)
    await connection.updateState()
    const offers = await Credential.getOffers(connection)
    logger.info(`Found ${offers.length} credential offers.`)
  }

  async function sendOffer ({ schemaAttrs, credDefName, connectionNameReceiver, connection, revoke, skipProgress }) {
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

    let connectionToReceiver
    if (connectionNameReceiver && !connection) {
      const connSerializedBefore = await storageService.loadConnection(connectionNameReceiver)
      connectionToReceiver = await Connection.deserialize(connSerializedBefore)
    } else if (!connectionNameReceiver && connection) {
      connectionToReceiver = connection
    } else {
      throw Error('Either connection or connection name are allowed as parameters, not both or none')
    }
    await connectionToReceiver.updateState()

    logger.info('Send credential offer to Alice')
    await issuerCred.sendOffer(connectionToReceiver)
    logger.debug(`IssuerCredential after offer was sent:\n${JSON.stringify(await issuerCred.serialize())}`)

    if (!skipProgress) {
      await _progressIssuerCredentialToState(issuerCred, StateType.RequestReceived, 10, 2000)
      logger.debug(`IssuerCredential after credential request was received:\n${JSON.stringify(await issuerCred.serialize())}`)
    }

    return { issuerCred, connectionToReceiver }
  }

  async function sendCredential ({ issuerCred, connectionToReceiver, revoke, skipProgress }) {
    logger.info('Issuing credential to Alice')
    await issuerCred.sendCredential(connectionToReceiver)
    logger.debug(`IssuerCredential after credential was sent:\n${JSON.stringify(await issuerCred.serialize())}`)

    const serCredential = await issuerCred.serialize()

    if (!skipProgress) {
      logger.info('Wait for alice to accept credential')
      await _progressIssuerCredentialToState(issuerCred, StateType.Accepted, 10, 2000)
      logger.debug(`IssuerCredential after credential was issued:\n${JSON.stringify(await issuerCred.serialize())}`)
      logger.info('Credential has been issued.')
    }

    if (revoke) {
      logger.info('Revoking credential')
      await issuerCred.revokeCredential()
    }
  }

  async function credentialIssue ({ schemaAttrs, credDefName, connectionNameReceiver, revoke, skipProgress }) {
    const { issuerCred, connectionToReceiver } = await sendOffer({ schemaAttrs, credDefName, connectionNameReceiver, skipProgress })
    await sendCredential({ issuerCred, connectionToReceiver, revoke, skipProgress })

    return { serCred: await issuerCred.serialize(), serConn: await connectionToReceiver.serialize() }
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
      if (await credential.updateState() !== credentialStateTarget) {
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
      if (await issuerCredential.updateState() !== credentialStateTarget) {
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

  async function waitForCredentialOfferAndAccept ({ connectionName, connection, skipProgress, attemptsThreshold = 10, timeout = 2000 }) {
    logger.info('Going to try fetch credential offer and receive credential.')
    let _connection
    if (connectionName && !_connection) {
      const connSerializedBefore = await storageService.loadConnection(connectionName)
      _connection = await Connection.deserialize(connSerializedBefore)
    } else if (!connectionName && connection) {
      _connection = connection
    } else {
      throw Error('Either connection or connection name are allowed as parameters, not both or none')
    }

    await _connection.updateState()
    const offers = await _getOffers(_connection, attemptsThreshold, timeout)
    logger.info(`Found ${offers.length} credential offers.`)

    const pickedOffer = JSON.stringify(offers[0])
    logger.debug(`Picked credential offer = ${pickedOffer}`)

    const credential = await Credential.create({ sourceId: 'credential', offer: pickedOffer })

    logger.info('After receiving credential offer, send credential request')
    await credential.sendRequest({ connection: _connection, payment: 0 })
    logger.debug(`CredentialSM after credential request was sent:\n${JSON.stringify(await credential.serialize())}`)

    if (!skipProgress) {
      logger.info('Poll agency and accept credential offer from faber')
      await _progressCredentialToState(credential, StateType.Accepted, attemptsThreshold, timeout)
      logger.debug(`CredentialSM after credential was received:\n${JSON.stringify(await credential.serialize())}`)
      logger.info('Credential has been received.')
    }

    return { credential, serCred: await credential.serialize() }
  }

  function getInstitutionDid () {
    return agentProvision.institution_did
  }

  async function verifierCreateProof ({ sourceId, attrs, preds, name, revocationInterval }) {
    return Proof.create({ sourceId, attrs, preds, name, revocationInterval })
  }

  async function verifierCreateProofRequest (connection, verifierProof) {
    await verifierProof.requestProof(connection)
    return verifierProof.getProofRequestMessage()
  }

  async function holderCreateProofFromRequest (request, sourceId = '123') {
    return DisclosedProof.create({ request, sourceId })
  }

  async function holderGetRequests (connection) {
    return DisclosedProof.getRequests(connection)
  }

  async function holderSelectCredentialsForProof (holderProof) {
    const resolvedCreds = await holderProof.getCredentials()
    const selectedCreds = { attrs: {} }
    logger.debug(`Resolved credentials for proof = ${JSON.stringify(resolvedCreds, null, 2)}`)

    for (const attrName of Object.keys(resolvedCreds.attrs)) {
      const attrCredInfo = resolvedCreds.attrs[attrName]
      if (Array.isArray(attrCredInfo) === false) {
        throw Error('Unexpected data, expected attrCredInfo to be an array.')
      }
      if (attrCredInfo.length > 0) {
        selectedCreds.attrs[attrName] = {
          credential: resolvedCreds.attrs[attrName][0]
        }
        selectedCreds.attrs[attrName].tails_file = '/tmp/tails'
      } else {
        logger.info(`No credential was resolved for requested attribute key ${attrName}, will have to be supplied via self-attested attributes.`)
      }
    }
    logger.debug(`Selected credentials:\n${JSON.stringify(selectedCreds, null, 2)}`)
    return selectedCreds
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
    sendOffer,
    sendCredential,
    waitForCredentialOfferAndAccept,
    connectionPrintInfo,
    getCredentialOffers,
    getInstitutionDid,
    createConnection,
    initVcx: initVcxOld,
    createInvite,
    inviteeConnectionCreateFromInvite,
    storeConnection,
    updateWebhookUrl,
    sendMessage,
    getMessages,
    verifierCreateProof,
    verifierCreateProofRequest,
    holderCreateProofFromRequest,
    holderSelectCredentialsForProof,
    holderGetRequests
  }
}

module.exports.createVcxAgent = createVcxAgent
