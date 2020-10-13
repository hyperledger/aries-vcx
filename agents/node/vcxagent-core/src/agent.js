const { getLedgerAuthorAgreement, setActiveTxnAuthorAgreementMeta } = require('@absaoss/node-vcx-wrapper')
const { createServiceLedgerCredDef } = require('./services/service-ledger-creddef')
const { createServiceLedgerSchema } = require('./services/service-ledger-schema')
const { createServiceVerifier } = require('./services/service-verifier')
const { createServiceProver } = require('./services/service-prover')
const { createServiceCredHolder } = require('./services/service-cred-holder')
const { createServiceCredIssuer } = require('./services/service-cred-issuer')
const { createServiceConnections } = require('./services/service-connections')
const { provisionAgentInAgency } = require('./utils/vcx-workflows')
const {
  initVcxCore,
  openVcxWallet,
  openVcxPool,
  vcxUpdateWebhookUrl,
  shutdownVcx
} = require('@absaoss/node-vcx-wrapper')
const { createStorageService } = require('./storage/storage-service')
const { waitUntilAgencyIsReady } = require('./common')

async function createVcxAgent ({ agentName, genesisPath, agencyUrl, seed, usePostgresWallet, logger }) {
  genesisPath = genesisPath || `${__dirname}/../resources/docker.txn`

  await waitUntilAgencyIsReady(agencyUrl, logger)

  const storageService = await createStorageService(agentName)
  if (!await storageService.agentProvisionExists()) {
    const agentProvision = await provisionAgentInAgency(agentName, genesisPath, agencyUrl, seed, usePostgresWallet, logger)
    await storageService.saveAgentProvision(agentProvision)
  }
  const agentProvision = await storageService.loadAgentProvision()

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
    const acceptanceMechanism = Object.keys(taaJson.aml)[0]
    logger.info(`Accepting TAA using mechanism ${acceptanceMechanism} at time ${utime}`)
    await setActiveTxnAuthorAgreementMeta(taaJson.text, taaJson.version, null, acceptanceMechanism, utime)
  }

  function getInstitutionDid () {
    return agentProvision.institution_did
  }

  const serviceConnections = createServiceConnections({
    logger,
    saveConnection: storageService.saveConnection,
    loadConnection: storageService.loadConnection,
    listConnectionIds: storageService.listConnectionKeys
  })
  const serviceLedgerSchema = createServiceLedgerSchema({
    logger,
    saveSchema: storageService.saveSchema,
    loadSchema: storageService.loadSchema,
    listSchemaIds: storageService.listSchemaKeys
  })
  const serviceLedgerCredDef = createServiceLedgerCredDef({
    logger,
    saveCredDef: storageService.saveCredentialDefinition,
    loadCredDef: storageService.loadCredentialDefinition,
    listCredDefIds: storageService.listCredentialDefinitionKeys
  })
  const serviceCredIssuer = createServiceCredIssuer({
    logger,
    loadConnection: storageService.loadConnection,
    loadCredDef: storageService.loadCredentialDefinition,
    saveIssuerCredential: storageService.saveCredIssuer,
    loadIssuerCredential: storageService.loadCredIssuer,
    listIssuerCredentialIds: storageService.listCredIssuerKeys
  })
  const serviceCredHolder = createServiceCredHolder({
    logger,
    loadConnection: storageService.loadConnection,
    saveHolderCredential: storageService.saveCredHolder,
    loadHolderCredential: storageService.loadCredHolder,
    listHolderCredentialIds: storageService.listCredHolderKeys
  })
  const serviceProver = createServiceProver({
    logger,
    loadConnection: storageService.loadConnection,
    saveDisclosedProof: storageService.saveDisclosedProof,
    loadDisclosedProof: storageService.loadDisclosedProof,
    listDislosedProofIds: storageService.listDisclosedProofKeys
  })
  const serviceVerifier = createServiceVerifier({
    logger,
    loadConnection: storageService.loadConnection,
    saveProof: storageService.saveProof,
    loadProof: storageService.loadProof,
    listProofIds: storageService.listProofKeys
  })

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
