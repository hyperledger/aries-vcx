const {createServiceVerifier} = require('./service-verifier')
const {createServiceProver} = require('./service-prover')
const {createServiceCredHolder} = require('./service-cred-holder')
const {createServiceCredIssuer} = require('./service-cred-issuer')
const {createServiceLedger} = require('./service-ledger')
const { createServiceConnections } = require('./service-connections')
const { provisionAgentInAgency } = require('../utils/vcx-workflows')
const {
  initVcxWithConfig,
  initVcxCore,
  openVcxWallet,
  openVcxPool,
  vcxUpdateWebhookUrl,
} = require('@absaoss/node-vcx-wrapper')
const { createStorageService } = require('../state/storage-service')
const { waitUntilAgencyIsReady } = require('../common')

async function createVcxAgent ({ agentName, genesisPath, protocolType, agencyUrl, seed, webhookUrl, usePostgresWallet, logger }) {
  genesisPath = genesisPath || `${__dirname}/../../resources/docker.txn`

  await waitUntilAgencyIsReady(agencyUrl, logger)

  const storageService = await createStorageService(agentName)
  if (!await storageService.agentProvisionExists()) {
    const agentProvision = await provisionAgentInAgency(agentName, genesisPath, protocolType, agencyUrl, seed, usePostgresWallet, logger)
    await storageService.saveAgentProvision(agentProvision)
  }
  const agentProvision = await storageService.loadAgentProvision()

  await initVcx()
  if (webhookUrl) {
    await vcxUpdateWebhookUrl(webhookUrl)
  }

  /**
   * Initializes libvcx configuration, open pool, open wallet, set webhook url if present in agent provision
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

  async function updateWebhookUrl (webhookUrl) {
    logger.info(`Updating webhook url to ${webhookUrl}`)
    await vcxUpdateWebhookUrl({ webhookUrl })
  }

  function getInstitutionDid () {
    return agentProvision.institution_did
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

  const serviceConnections = createServiceConnections(logger, storageService.saveConnection, storageService.loadConnection)

  const serviceLedger = createServiceLedger(logger, storageService.saveSchema, storageService.loadSchema, storageService.saveCredentialDefinition, storageService.loadCredentialDefinition)

  const serviceCredIssuer = createServiceCredIssuer(logger, storageService.loadConnection, storageService.loadCredentialDefinition, storageService.saveCredIssuer, storageService.loadCredIssuer)
  const serviceCredHolder = createServiceCredHolder(logger, storageService.loadConnection, storageService.saveCredHolder, storageService.loadCredHolder)

  const serviceProver = createServiceProver(logger, storageService.loadConnection, storageService.saveDisclosedProof, storageService.loadDisclosedProof)
  const serviceVerifier = createServiceVerifier(logger, storageService.loadConnection, storageService.saveProof, storageService.loadProof)

  return {
    // vcx controls
    initVcxOld,
    initVcx,
    getInstitutionDid,
    updateWebhookUrl,


    // connections
    serviceConnections,
    connectionsList,
    connectionPrintInfo,

    // ledger
    serviceLedger,

    // credex
    serviceCredIssuer,
    serviceCredHolder,

    // proofs
    serviceProver,
    serviceVerifier
  }
}

module.exports.createVcxAgent = createVcxAgent
