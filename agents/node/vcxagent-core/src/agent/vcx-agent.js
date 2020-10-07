const { createServiceVerifier } = require('./service-verifier')
const { createServiceProver } = require('./service-prover')
const { createServiceCredHolder } = require('./service-cred-holder')
const { createServiceCredIssuer } = require('./service-cred-issuer')
const { createServiceLedger } = require('./service-ledger')
const { createServiceConnections } = require('./service-connections')
const { provisionAgentInAgency } = require('../utils/vcx-workflows')
const {
  initVcxCore,
  openVcxWallet,
  openVcxPool,
  vcxUpdateWebhookUrl,
  shutdownVcx
} = require('@absaoss/node-vcx-wrapper')
const { createStorageService } = require('../state/storage-service')
const { waitUntilAgencyIsReady } = require('../common')

async function createVcxAgent ({ agentName, genesisPath, protocolType, agencyUrl, seed, usePostgresWallet, logger }) {
  genesisPath = genesisPath || `${__dirname}/../../resources/docker.txn`

  await waitUntilAgencyIsReady(agencyUrl, logger)

  const storageService = await createStorageService(agentName)
  if (!await storageService.agentProvisionExists()) {
    const agentProvision = await provisionAgentInAgency(agentName, genesisPath, protocolType, agencyUrl, seed, usePostgresWallet, logger)
    await storageService.saveAgentProvision(agentProvision)
  }
  const agentProvision = await storageService.loadAgentProvision()

  /**
   * Performs the same as initVcxOld, except for the fact it ignores webhook_url in agent provision. You have to
   * update webhook_url by calling function vcxUpdateWebhookUrl.
   */
  async function agentInitVcx () {
    logger.info(`Initializing ${agentName} vcx session.`)
    logger.silly(`Using following agent provision to initialize VCX settings ${JSON.stringify(agentProvision, null, 2)}`)
    await initVcxCore(JSON.stringify(agentProvision))
    logger.silly('Opening pool')
    await openVcxPool()
    logger.silly('Opening wallet')
    await openVcxWallet()
    logger.silly('LibVCX fully initialized')
  }

  async function agentShutdownVcx () {
    logger.debug(`Shutting down ${agentName} vcx session.`)
    shutdownVcx()
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
    const state = await serviceConnections.connectionGetState(connectionName)
    logger.info(`Connection ${connectionName} state=${state}`)
  }

  const serviceConnections = createServiceConnections(logger, storageService.saveConnection, storageService.loadConnection)

  const serviceLedger = createServiceLedger(logger, storageService.saveSchema, storageService.loadSchema, storageService.saveCredentialDefinition, storageService.loadCredentialDefinition)

  const serviceCredIssuer = createServiceCredIssuer(logger, storageService.loadConnection, storageService.loadCredentialDefinition, storageService.saveCredIssuer, storageService.loadCredIssuer)
  const serviceCredHolder = createServiceCredHolder(logger, storageService.loadConnection, storageService.saveCredHolder, storageService.loadCredHolder)

  const serviceProver = createServiceProver(logger, storageService.loadConnection, storageService.saveDisclosedProof, storageService.loadDisclosedProof)
  const serviceVerifier = createServiceVerifier(logger, storageService.loadConnection, storageService.saveProof, storageService.loadProof)

  return {
    // vcx controls
    agentInitVcx,
    agentShutdownVcx,
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
