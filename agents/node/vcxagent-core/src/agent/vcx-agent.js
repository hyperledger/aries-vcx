const { getLedgerAuthorAgreement, setActiveTxnAuthorAgreementMeta } = require('@absaoss/node-vcx-wrapper')
const { createServiceLedgerCredDef } = require('./service-ledger-creddef')
const { createServiceLedgerSchema } = require('./service-ledger-schema')
const { createServiceVerifier } = require('./service-verifier')
const { createServiceProver } = require('./service-prover')
const { createServiceCredHolder } = require('./service-cred-holder')
const { createServiceCredIssuer } = require('./service-cred-issuer')
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

  async function acceptTaa () {
    const taa = await getLedgerAuthorAgreement()
    const taaJson = JSON.parse(taa)
    const utime = Math.floor(new Date() / 1000)
    await setActiveTxnAuthorAgreementMeta(taaJson.text, taaJson.version, null, Object.keys(taaJson.aml)[0], utime)
  }

  function getInstitutionDid () {
    return agentProvision.institution_did
  }

  const serviceConnections = createServiceConnections(
    logger,
    storageService.saveConnection,
    storageService.loadConnection,
    storageService.listConnectionKeys
  )
  const serviceLedgerSchema = createServiceLedgerSchema(
    logger,
    storageService.saveSchema,
    storageService.loadSchema,
    storageService.listSchemaKeys
  )
  const serviceLedgerCredDef = createServiceLedgerCredDef(
    logger,
    storageService.saveSchema,
    storageService.loadSchema,
    storageService.saveCredentialDefinition,
    storageService.loadCredentialDefinition,
    storageService.listCredentialDefinitionKeys
  )
  const serviceCredIssuer = createServiceCredIssuer(
    logger,
    storageService.loadConnection,
    storageService.loadCredentialDefinition,
    storageService.saveCredIssuer,
    storageService.loadCredIssuer,
    storageService.listCredIssuerKeys
  )
  const serviceCredHolder = createServiceCredHolder(
    logger,
    storageService.loadConnection,
    storageService.saveCredHolder,
    storageService.loadCredHolder,
    storageService.listCredHolderKeys
  )
  const serviceProver = createServiceProver(
    logger,
    storageService.loadConnection,
    storageService.saveDisclosedProof,
    storageService.loadDisclosedProof,
    storageService.listDisclosedProofKeys
  )
  const serviceVerifier = createServiceVerifier(
    logger,
    storageService.loadConnection,
    storageService.saveProof,
    storageService.loadProof,
    storageService.listProofKeys
  )

  return {
    // vcx controls
    agentInitVcx,
    agentShutdownVcx,
    getInstitutionDid,
    updateWebhookUrl,
    acceptTaa,

    // ledger
    serviceLedgerSchema,
    serviceLedgerCredDef,

    // connections
    serviceConnections,

    // credex
    serviceCredIssuer,
    serviceCredHolder,

    // proofs
    serviceProver,
    serviceVerifier
  }
}

module.exports.createVcxAgent = createVcxAgent
