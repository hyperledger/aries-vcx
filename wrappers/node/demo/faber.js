// todo: fix next line
const { provisionAgent } = require('../client-vcx/vcx-workflows')
const { Proof } = require('../dist/src/api/proof')
const { StateType, ProofState } = require('../dist/src')
const sleepPromise = require('sleep-promise')
const { runScript } = require('../common/script-comon')
const { createVcxClient } = require('../client-vcx/vcxclient')
const logger = require('../common/logger')
const assert = require('assert')
const { initRustapi } = require('../client-vcx/vcx-workflows')
const uuid = require('uuid')
const { waitUntilAgencyIsReady } = require('../common/common')
const { createStorageService } = require('../client-vcx/storage-service')
const allowedProtocolTypes = ['1.0', '2.0', '3.0', '4.0']

async function runFaber (options) {
  const testRunId = uuid.v4()
  const seed = '000000000000000000000000Trustee1'
  const protocolType = options.protocolType
  const agentName = `alice-${testRunId}`
  const webhookUrl = `http://localhost:7209/notifications/${agentName}`
  const usePostgresWallet = false
  const acceptTaa = process.env.ACCEPT_TAA || false
  const logLevel = 'error'

  await initRustapi(logLevel)

  const agencyUrl = 'http://localhost:8080'
  await waitUntilAgencyIsReady(agencyUrl, logger)

  const storageService = await createStorageService(agentName)

  if (!await storageService.agentProvisionExists()) {
    const agentProvision = await provisionAgent(agentName, protocolType, agencyUrl, seed, webhookUrl, usePostgresWallet, logger)
    await storageService.saveAgentProvision(agentProvision)
  }
  const agentProvision = await storageService.loadAgentProvision()
  const issuerDid = agentProvision.institution_did
  const vcxClient = await createVcxClient(storageService, logger)

  if (acceptTaa) {
    await vcxClient.acceptTaa()
  }

  const schema = await vcxClient.createSchema()
  const schemaId = await schema.getSchemaId()
  await vcxClient.createCredentialDefinition(schemaId, 'DemoCredential123', logger)

  const connectionName = `alice-${testRunId}`
  const invitationString = await vcxClient.connectionCreate(connectionName)
  logger.info('\n\n**invite details**')
  logger.info("**You'll ge queried to paste this data to alice side of the demo. This is invitation to connect.**")
  logger.info("**It's assumed this is obtained by Alice from Faber by some existing secure channel.**")
  logger.info('**Could be on website via HTTPS, QR code scanned at Faber institution, ...**')
  logger.info('\n******************\n\n')
  logger.info(invitationString)
  logger.info('\n\n******************\n\n')

  const connectionToAlice = await vcxClient.connectionAutoupdate(connectionName, 30, 3000)
  if (!connectionToAlice) {
    throw Error('Connection with alice was not established.')
  }
  logger.info('Connection to alice was Accepted!')

  const schemaAttrs = {
    name: 'alice',
    last_name: 'clark',
    sex: 'female',
    date: '05-2018',
    degree: 'maths',
    age: '25'
  }

  await vcxClient.credentialIssue(schemaAttrs, 'DemoCredential123', connectionName, options.revocation)

  const proofAttributes = [
    {
      names: ['name', 'last_name', 'sex'],
      restrictions: [{ issuer_did: issuerDid }]
    },
    {
      name: 'date',
      restrictions: { issuer_did: issuerDid }
    },
    {
      name: 'degree',
      restrictions: { 'attr::degree::value': 'maths' }
    },
    {
      name: 'nickname',
      self_attest_allowed: true
    }
  ]

  const proofPredicates = [
    { name: 'age', p_type: '>=', p_value: 20, restrictions: [{ issuer_did: agentProvision.institution_did }] }
  ]

  logger.info('#19 Create a Proof object')
  const vcxProof = await Proof.create({
    sourceId: '213',
    attrs: proofAttributes,
    preds: proofPredicates,
    name: 'proofForAlice',
    revocationInterval: { to: Date.now() }
  })

  logger.info('#20 Request proof of degree from alice')
  await vcxProof.requestProof(connectionToAlice)

  logger.info('#21 Poll agency and wait for alice to provide proof')
  let proofProtocolState = await vcxProof.getState()
  logger.info(`vcxProof = ${JSON.stringify(vcxProof)}`)
  logger.info(`proofState = ${proofProtocolState}`)
  while (proofProtocolState !== StateType.Accepted) {
    // even if revoked credential was used, vcxProof.getState() should in final state return StateType.Accepted
    await sleepPromise(2000)
    await vcxProof.updateState()
    proofProtocolState = await vcxProof.getState()
    logger.info(`proofState=${proofProtocolState}`)
  }

  logger.info('#27 Process the proof provided by alice.')
  const { proofState, proof } = await vcxProof.getProof(connectionToAlice)
  assert(proofState)
  assert(proof)
  logger.info(`proofState = ${JSON.stringify(proofProtocolState)}`)
  logger.info(`vcxProof = ${JSON.stringify(vcxProof)}`)

  logger.info('#28 Check if proof is valid.')
  if (proofState === ProofState.Verified) {
    logger.warn('Proof is verified.')
    assert(options.revocation === false)
  } else if (proofState === ProofState.Invalid) {
    logger.warn('Proof verification failed, credential has been revoked.')
    assert(options.revocation === true)
  } else {
    logger.error(`Unexpected proof state '${proofState}'.`)
    process.exit(-1)
  }
  logger.info(`Serialized proof ${JSON.stringify(await vcxProof.serialize())}`)
  process.exit(0)
}

const optionDefinitions = [
  {
    name: 'help',
    alias: 'h',
    type: Boolean,
    description: 'Display this usage guide.'
  },
  {
    name: 'protocolType',
    type: String,
    description: 'Protocol type. Possible values: "1.0" "2.0" "3.0" "4.0". Default is 4.0',
    defaultValue: '4.0'
  },
  {
    name: 'postgresql',
    type: Boolean,
    description: 'If specified, postresql wallet will be used.',
    defaultValue: false
  },
  {
    name: 'revocation',
    type: Boolean,
    description: 'If specified, the issued credential will be revoked',
    defaultValue: false
  }
]

const usage = [
  {
    header: 'Options',
    optionList: optionDefinitions
  },
  {
    content: 'Project home: {underline https://github.com/AbsaOSS/libvcx}'
  }
]

function areOptionsValid (options) {
  if (!(allowedProtocolTypes.includes(options.protocolType))) {
    console.error(`Unknown protocol type ${options.protocolType}. Only ${JSON.stringify(allowedProtocolTypes)} are allowed.`)
    return false
  }
  return true
}
runScript(optionDefinitions, usage, areOptionsValid, runFaber)
