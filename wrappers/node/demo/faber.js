import { initializeVcxClient } from './workflows-vcx'

const { CredentialDef } = require('../dist/src/api/credential-def')
const { IssuerCredential } = require('../dist/src/api/issuer-credential')
const { Proof } = require('../dist/src/api/proof')
const { Connection } = require('../dist/src/api/connection')
const { Schema } = require('./../dist/src/api/schema')
const { StateType, ProofState } = require('../dist/src')
const sleepPromise = require('sleep-promise')
const { getRandomInt } = require('./common')
const logger = require('./logger')
const { runScript } = require('./script-comon')
const assert = require('assert')

const utime = Math.floor(new Date() / 1000)
const webhookUrl = 'http://localhost:7209/notifications/faber'

const TAA_ACCEPT = process.env.TAA_ACCEPT === 'true' || false

const provisionConfig = {
  agency_url: 'http://localhost:8080',
  agency_did: 'VsKV7grR1BUE29mG2Fm2kX',
  agency_verkey: 'Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR',
  wallet_name: `node_vcx_demo_faber_wallet_${utime}`,
  wallet_key: '123',
  payment_method: 'null',
  enterprise_seed: '000000000000000000000000Trustee1'
}

const logLevel = 'error'

async function runFaber (options) {
  provisionConfig.protocol_type = options.protocolType
  await initializeVcxClient(provisionConfig, options.postgresql, webhookUrl, TAA_ACCEPT, logger, logLevel)

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

  logger.info('#4 Create a new credential definition on the ledger')
  const data = {
    name: 'DemoCredential123',
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
  const credDefId = await credDef.getCredDefId()
  const credDefHandle = credDef.handle
  logger.info(`Created credential with id ${credDefId} and handle ${credDefHandle}`)

  logger.info('#5 Create a connection to alice and print out the invite details')
  const connectionToAlice = await Connection.create({ id: 'alice' })
  await connectionToAlice.connect('{}')
  await connectionToAlice.updateState()
  const details = await connectionToAlice.inviteDetails(false)
  logger.info('\n\n**invite details**')
  logger.info("**You'll ge queried to paste this data to alice side of the demo. This is invitation to connect.**")
  logger.info("**It's assumed this is obtained by Alice from Faber by some existing secure channel.**")
  logger.info('**Could be on website via HTTPS, QR code scanned at Faber institution, ...**')
  logger.info('\n******************\n\n')
  logger.info(JSON.stringify(JSON.parse(details)))
  logger.info('\n\n******************\n\n')

  logger.info('#6 Polling agency and waiting for alice to accept the invitation. (start alice.py now)')
  let connectionState = await connectionToAlice.getState()
  while (connectionState !== StateType.Accepted) {
    await sleepPromise(2000)
    await connectionToAlice.updateState()
    connectionState = await connectionToAlice.getState()
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

  logger.info('#12 Create an IssuerCredential object using the schema and credential definition')

  const credentialForAlice = await IssuerCredential.create({
    attr: schemaAttrs,
    sourceId: 'alice_degree',
    credDefHandle,
    credentialName: 'cred',
    price: '0'
  })

  logger.info('#13 Issue credential offer to alice')
  await credentialForAlice.sendOffer(connectionToAlice)
  await credentialForAlice.updateState()

  logger.info('#14 Poll agency and wait for alice to send a credential request')
  let credentialState = await credentialForAlice.getState()
  while (credentialState !== StateType.RequestReceived) {
    await sleepPromise(2000)
    await credentialForAlice.updateState()
    credentialState = await credentialForAlice.getState()
  }

  logger.info('#17 Issue credential to alice')
  await credentialForAlice.sendCredential(connectionToAlice)

  logger.info('#18 Wait for alice to accept credential')
  await credentialForAlice.updateState()
  credentialState = await credentialForAlice.getState()
  while (credentialState !== StateType.Accepted) {
    await sleepPromise(2000)
    await credentialForAlice.updateState()
    credentialState = await credentialForAlice.getState()
  }

  const proofAttributes = [
    {
      names: ['name', 'last_name', 'sex'],
      restrictions: [{ issuer_did: agentProvision.institution_did }]
    },
    {
      name: 'date',
      restrictions: { issuer_did: agentProvision.institution_did }
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

  if (options.revocation) {
    logger.info('#18.5 Revoking credential')
    await credentialForAlice.revokeCredential()
  }
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
