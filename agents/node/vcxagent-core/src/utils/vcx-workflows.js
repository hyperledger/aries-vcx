const {
  initRustAPI, initThreadpool, initVcxWithConfig, provisionCloudAgent,
  createWallet, openMainWallet, closeMainWallet,
  configureIssuerWallet
} = require('@hyperledger/node-vcx-wrapper')
const axios = require('axios')

async function initRustApiAndLogger (logLevel) {
  const rustApi = initRustAPI()
  await rustApi.vcx_set_default_logger(logLevel)
}

async function initVcxWithProvisionedAgentConfig (config) {
  await initVcxWithConfig(JSON.stringify(config))
}

async function initRustapi (logLevel = 'vcx=error', num_threads = 4) {
  await initRustApiAndLogger(logLevel)
  await initThreadpool({ num_threads })
}

async function provisionAgentInAgency (agentName, genesisPath, agencyConfig, seed, walletExtraConfigs, logger) {
  logger.info('Provisioning cloud agent')
  if (!agentName) {
    throw Error('agentName not specified')
  }
  if (!genesisPath) {
    throw Error('genesisPath not specified')
  }
  if (!agencyConfig) {
    throw Error('agencyConfig not specified')
  }
  if (!seed) {
    throw Error('seed not specified')
  }

  const walletConfig = {
    wallet_name: agentName,
    wallet_key: '8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY',
    wallet_key_derivation: 'RAW'
  }
  walletExtraConfigs = walletExtraConfigs || { }
  for (const key of Object.keys(walletExtraConfigs)) {
    const value = walletExtraConfigs[key]
    if (typeof value === 'object') {
      walletConfig[key] = JSON.stringify(value)
    } else {
      walletConfig[key] = value
    }
  }
  logger.info(`Using wallet config ${JSON.stringify(walletConfig)}`)

  logger.debug(`Creating wallet with config: ${JSON.stringify(walletConfig, null, 2)}`)
  await createWallet(walletConfig)
  logger.debug(`Opening wallet with config: ${JSON.stringify(walletConfig, null, 2)}`)
  await openMainWallet(walletConfig)
  logger.debug(`Configuring issuer's wallet with seed: ${seed}`)
  const issuerConfig = JSON.parse(await configureIssuerWallet(seed))
  issuerConfig.institution_name = agentName
  logger.debug(`Configured issuer wallet with config: ${JSON.stringify(issuerConfig, null, 2)}`)
  logger.debug(`Provisioning agent with config: ${JSON.stringify(agencyConfig, null, 2)}`)
  agencyConfig = JSON.parse(await provisionCloudAgent(agencyConfig))
  logger.debug(`Provisioned agent with config: ${JSON.stringify(agencyConfig, null, 2)}`)
  await closeMainWallet()

  return { agencyConfig, issuerConfig, walletConfig }
}

module.exports.initRustApiAndLogger = initRustApiAndLogger
module.exports.initVcxWithProvisionedAgentConfig = initVcxWithProvisionedAgentConfig
module.exports.provisionAgentInAgency = provisionAgentInAgency
module.exports.initRustapi = initRustapi
