const { provisionAgent } = require('../client-vcx/vcx-workflows')
const { DisclosedProof } = require('../dist/src/api/disclosed-proof')
const { StateType } = require('../dist/src')
const readlineSync = require('readline-sync')
const { createVcxClient } = require('../client-vcx/vcxclient')
const { createStorageService } = require('../client-vcx/storage-service')
const { waitUntilAgencyIsReady } = require('../common/common')
const { initRustapi } = require('../client-vcx/vcx-workflows')
const sleepPromise = require('sleep-promise')
const logger = require('../common/logger')
const { runScript } = require('../common/script-comon')
const uuid = require('uuid')
const axios = require('axios')
const isPortReachable = require('is-port-reachable')
const url = require('url')

const allowedProtocolTypes = ['1.0', '2.0', '3.0', '4.0']

async function getInvitationString (fetchInviteUrl) {
  let invitationString
  if (fetchInviteUrl) {
    const fetchInviteAttemptThreshold = 30
    const fetchInviteTimeout = 1000
    let fetchInviteAttemps = 0
    while (!invitationString) {
      if (await isPortReachable(url.parse(fetchInviteUrl).port, { host: url.parse(fetchInviteUrl).hostname })) { // eslint-disable-line
        ({ data: { invitationString } } = await axios.get(fetchInviteUrl))
        logger.info(`Invitation ${invitationString} was loaded from ${fetchInviteUrl}.`)
      } else {
        logger.info(`Invitation fetch url ${fetchInviteUrl} not yet available. ${fetchInviteAttemps}/${fetchInviteAttemptThreshold}`)
        await sleepPromise(fetchInviteTimeout)
      }
      fetchInviteAttemps++
      if (fetchInviteAttemps > 15) {
        throw Error(`Could not reach ${fetchInviteUrl} to fetch connection invitation.`)
      }
    }
  } else {
    invitationString = readlineSync.question('Enter connection invitation:\n')
  }
  return invitationString
}

async function runAlice (options) {
  const testRunId = uuid.v4()
  const seed = '000000000000000000000000Trustee1'
  const protocolType = options.protocolType
  const agentName = `alice-${testRunId}`
  const webhookUrl = `http://localhost:7209/notifications/${agentName}`
  const usePostgresWallet = false
  const logLevel = 'error'

  await initRustapi(logLevel)

  const agencyUrl = 'http://localhost:8080'
  await waitUntilAgencyIsReady(agencyUrl, logger)

  const storageService = await createStorageService(agentName)

  if (!await storageService.agentProvisionExists()) {
    const agentProvision = await provisionAgent(agentName, protocolType, agencyUrl, seed, webhookUrl, usePostgresWallet, logger)
    await storageService.saveAgentProvision(agentProvision)
  }
  const vcxClient = await createVcxClient(storageService, logger)

  const connectionName = `alice-${testRunId}`

  const invitationString = await getInvitationString(options['autofetch-invitation-url'])
  await vcxClient.connectionAccept(connectionName, invitationString)

  const connectionToFaber = await vcxClient.connectionAutoupdate(connectionName)
  if (!connectionToFaber) {
    throw Error('Connection with alice was not established.')
  }
  logger.info('Connection to alice was Accepted!')

  await vcxClient.waitForCredentialOfferAndAccept(connectionName)

  logger.info('#22 Poll agency for a proof request')
  let requests = await DisclosedProof.getRequests(connectionToFaber)
  while (requests.length === 0) {
    await sleepPromise(2000)
    requests = await DisclosedProof.getRequests(connectionToFaber)
  }
  logger.info('#23 Create a Disclosed proof object from proof request')
  logger.debug(`Received proof request = ${JSON.stringify(requests, null, 2)}`)
  const proof = await DisclosedProof.create({ sourceId: 'proof', request: JSON.stringify(requests[0]) })
  const requestInfo = JSON.parse(Buffer.from(requests[0]['request_presentations~attach'][0].data.base64, 'base64').toString('utf8'))
  logger.debug(`Proof request presentation attachment ${JSON.stringify(requestInfo, null, 2)}`)

  logger.info('#24 Query for credentials in the wallet that satisfy the proof request')
  const resolvedCreds = await proof.getCredentials()
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
  const selfAttestedAttrs = { attribute_3: 'Smith' }

  logger.info('#25 Generate the proof.')
  logger.debug(`Proof is using wallet credentials:\n${JSON.stringify(selectedCreds, null, 2)}
  \nProof is using self attested attributes: ${JSON.stringify(selfAttestedAttrs, null, 2)}`)
  await proof.generateProof({ selectedCreds, selfAttestedAttrs })

  logger.info('#26 Send the proof to faber')
  await proof.sendProof(connectionToFaber)

  logger.info('#27 Wait for Faber to receive the proof')
  let proofState = await proof.getState()
  while (proofState !== StateType.Accepted && proofState !== StateType.None) {
    await sleepPromise(2000)
    await proof.updateState()
    proofState = await proof.getState()
  }
  logger.info('Faber received the proof')
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
    name: 'autofetch-invitation-url',
    type: String,
    description: 'If specified, postresql wallet will be used.',
    defaultValue: 'http://localhost:8181'
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

runScript(optionDefinitions, usage, areOptionsValid, runAlice)
