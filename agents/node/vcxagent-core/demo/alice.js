const { initRustapi, allowedProtocolTypes } = require('../vcx-workflows')
const { StateType, DisclosedProof } = require('@absaoss/node-vcx-wrapper')
const readlineSync = require('readline-sync')
const { createVcxAgent } = require('../vcx-agent')
const sleepPromise = require('sleep-promise')
const logger = require('./logger')('Alice')
const { runScript } = require('./script-common')
const uuid = require('uuid')
const axios = require('axios')
const isPortReachable = require('is-port-reachable')
const url = require('url')

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
      if (fetchInviteAttemps > fetchInviteAttemptThreshold) {
        throw Error(`Could not reach ${fetchInviteUrl} to fetch connection invitation.`)
      }
    }
  } else {
    invitationString = readlineSync.question('Enter connection invitation:\n')
  }
  return invitationString
}

async function runAlice (options) {
  logger.info('Starting.')

  await initRustapi(process.env.VCX_LOG_LEVEL || 'vcx=error')
  const agentName = `alice-${uuid.v4()}`
  const vcxClient = await createVcxAgent({
    agentName,
    protocolType: options.protocolType,
    agencyUrl: 'http://localhost:8080',
    seed: '000000000000000000000000Trustee1',
    webhookUrl: `http://localhost:7209/notifications/${agentName}`,
    usePostgresWallet: false,
    logger
  })

  const invitationString = await getInvitationString(options['autofetch-invitation-url'])
  const connectionToFaber = await vcxClient.inviteeConnectionAcceptFromInvitation(agentName, invitationString)

  if (!connectionToFaber) {
    throw Error('Connection with alice was not established.')
  }
  logger.info('Connection to alice was Accepted!')

  await vcxClient.waitForCredentialOfferAndAccept({ connectionName: agentName })

  logger.info('Poll agency for a proof request')
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
  const selectedCreds = await vcxClient.holderSelectCredentialsForProof(proof)
  const selfAttestedAttrs = { attribute_3: 'Smith' }

  logger.info('Generate the proof.')
  logger.debug(`Proof is using wallet credentials:\n${JSON.stringify(selectedCreds, null, 2)}
  \nProof is using self attested attributes: ${JSON.stringify(selfAttestedAttrs, null, 2)}`)
  await proof.generateProof({ selectedCreds, selfAttestedAttrs })

  logger.info('Send the proof to faber')
  await proof.sendProof(connectionToFaber)

  logger.info('Wait for Faber to receive the proof')
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
    description: 'If specified, the script will try to download invitation from specified url.'
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
