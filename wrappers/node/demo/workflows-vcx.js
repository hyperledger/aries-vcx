const {
  getRandomInt,
  initLibNullPay,
  initRustApiAndLogger,
  initVcxWithProvisionedAgentConfig,
  loadPostgresPlugin,
  provisionAgentInAgency
} = require('./common')

async function initializeVcxClient (provisionConfig, postgresqlOptions, webhookUrl, acceptTaa, logger, logLevel) {
  await initLibNullPay()

  logger.info('#0 Initialize rust API from NodeJS')
  await initRustApiAndLogger(logLevel)

  if (postgresqlOptions) {
    await loadPostgresPlugin(provisionConfig)
    provisionConfig.wallet_type = 'postgres_storage'
    provisionConfig.storage_config = '{"url":"localhost:5432"}'
    provisionConfig.storage_credentials = '{"account":"postgres","password":"mysecretpassword","admin_account":"postgres","admin_password":"mysecretpassword"}'
    logger.info(`Running with PostreSQL wallet enabled! Config = ${provisionConfig.storage_config}`)
  } else {
    logger.info('Running with builtin wallet.')
  }

  if (await isPortReachable(url.parse(webhookUrl).port, { host: url.parse(webhookUrl).hostname })) { // eslint-disable-line
    provisionConfig.webhook_url = webhookUrl
    logger.info(`Running with webhook notifications enabled! Webhook url = ${webhookUrl}`)
  } else {
    logger.info('Webhook url will not be used')
  }

  logger.info(`#1 Config used to provision agent in agency: ${JSON.stringify(provisionConfig, null, 2)}`)
  const agentProvision = await provisionAgentInAgency(provisionConfig)

  logger.info(`#2 Using following agent provision to initialize VCX ${JSON.stringify(agentProvision, null, 2)}`)
  await initVcxWithProvisionedAgentConfig(agentProvision)

  if (acceptTaa) {
    await acceptTaa()
  }
}

async function createSchema() {
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
}

module.exports.initializeVcxClient = initializeVcxClient
