const {
  initRustAPI, initVcxWithConfig, provisionCloudAgent,
  createWallet, openMainWallet, closeMainWallet,
  configureIssuerWallet, provisionAgent
} = require('@hyperledger/node-vcx-wrapper')
const ffi = require('ffi-napi')
const os = require('os')

const extension = { darwin: '.dylib', linux: '.so', win32: '.dll' }
const libPath = { darwin: '/usr/local/lib/', linux: '/usr/lib/', win32: 'c:\\windows\\system32\\' }

function getLibraryPath (libraryName) {
  const platform = os.platform()
  const postfix = extension[platform.toLowerCase()] || extension.linux
  const libDir = libPath[platform.toLowerCase()] || libPath.linux
  return `${libDir}${libraryName}${postfix}`
}

async function loadPostgresPlugin () {
  const myffi = ffi.Library(getLibraryPath('libindystrgpostgres'), { postgresstorage_init: ['void', []] })
  await myffi.postgresstorage_init()
}

async function initLibNullPay () {
  const myffi = ffi.Library(getLibraryPath('libnullpay'), { nullpay_init: ['void', []] })
  await myffi.nullpay_init()
}

async function initRustApiAndLogger (logLevel) {
  const rustApi = initRustAPI()
  await rustApi.vcx_set_default_logger(logLevel)
}

async function initVcxWithProvisionedAgentConfig (config) {
  await initVcxWithConfig(JSON.stringify(config))
}

async function initRustapi (logLevel = 'vcx=error') {
  await initLibNullPay()
  await initRustApiAndLogger(logLevel)
}

async function provisionAgentInAgency (agentName, genesisPath, agencyUrl, seed, usePostgresWallet, logger) {
  logger.info('Provisioning cloud agent')
  if (!agentName) {
    throw Error('agentName not specified')
  }
  if (!genesisPath) {
    throw Error('genesisPath not specified')
  }
  if (!agencyUrl) {
    throw Error('agencyUrl not specified')
  }
  if (!seed) {
    throw Error('seed not specified')
  }

  const walletConfig = {
    wallet_name: agentName,
    wallet_key: '8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY',
    wallet_key_derivation: 'RAW'
  }

  if (usePostgresWallet) {
    logger.info('Will use PostreSQL wallet. Initializing plugin.')
    await loadPostgresPlugin()
    walletConfig.wallet_type = 'postgres_storage'
    walletConfig.storage_config = '{"url":"localhost:5432"}'
    walletConfig.storage_credentials = '{"account":"postgres","password":"mysecretpassword","admin_account":"postgres","admin_password":"mysecretpassword"}'
    logger.info(`Running with PostreSQL wallet enabled! Config = ${walletConfig.storage_config}`)
  } else {
    logger.info('Running with builtin wallet.')
  }

  let agencyConfig = {
    agency_endpoint: agencyUrl,
    agency_did: 'VsKV7grR1BUE29mG2Fm2kX',
    agency_verkey: 'Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR'
  }

  logger.debug(`Creating wallet with config: ${JSON.stringify(walletConfig, null, 2)}`)
  await createWallet(walletConfig)
  logger.debug(`Opening wallet with config: ${JSON.stringify(walletConfig, null, 2)}`)
  await openMainWallet(walletConfig)
  logger.debug(`Configuring issuer's wallet with seed: ${seed}`)
  const issuerConfig = JSON.parse(await configureIssuerWallet(seed))
  issuerConfig.institution_name = agentName
  logger.debug(`Provisioning agent with config: ${JSON.stringify(agencyConfig, null, 2)}`)
  agencyConfig = JSON.parse(await provisionCloudAgent(agencyConfig))
  logger.debug(`Provisined agent with config: ${JSON.stringify(agencyConfig, null, 2)}`)
  await closeMainWallet()

  return { agencyConfig, issuerConfig, walletConfig }
}

async function provisionAgentInAgencyLegacy (agentName, genesisPath, agencyUrl, seed, usePostgresWallet, logger) {
  if (!agentName) {
    throw Error('agentName not specified')
  }
  if (!genesisPath) {
    throw Error('genesisPath not specified')
  }
  if (!agencyUrl) {
    throw Error('agencyUrl not specified')
  }
  if (!seed) {
    throw Error('seed not specified')
  }
  const provisionConfig = {
    agency_url: agencyUrl,
    agency_did: 'VsKV7grR1BUE29mG2Fm2kX',
    agency_verkey: 'Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR',
    wallet_name: agentName,
    wallet_key: '123',
    payment_method: 'null',
    enterprise_seed: seed
  }
  if (usePostgresWallet) {
    logger.info('Will use PostreSQL wallet. Initializing plugin.')
    await loadPostgresPlugin()
    provisionConfig.wallet_type = 'postgres_storage'
    provisionConfig.storage_config = '{"url":"localhost:5432"}'
    provisionConfig.storage_credentials = '{"account":"postgres","password":"mysecretpassword","admin_account":"postgres","admin_password":"mysecretpassword"}'
    logger.info(`Running with PostreSQL wallet enabled! Config = ${provisionConfig.storage_config}`)
  } else {
    logger.info('Running with builtin wallet.')
  }

  logger.info(`Using following config to create agent provision: ${JSON.stringify(provisionConfig, null, 2)}`)
  const agentProvision = JSON.parse(await provisionAgent(JSON.stringify(provisionConfig)))
  agentProvision.institution_name = agentName
  agentProvision.genesis_path = genesisPath
  logger.info(`Agent provision created: ${JSON.stringify(agentProvision, null, 2)}`)
  return agentProvision
}

module.exports.loadPostgresPlugin = loadPostgresPlugin
module.exports.initLibNullPay = initLibNullPay
module.exports.initRustApiAndLogger = initRustApiAndLogger
module.exports.initVcxWithProvisionedAgentConfig = initVcxWithProvisionedAgentConfig
module.exports.provisionAgentInAgency = provisionAgentInAgency
module.exports.provisionAgentInAgencyLegacy = provisionAgentInAgencyLegacy
module.exports.initRustapi = initRustapi
