const readlineSync = require('readline-sync')
const sleepPromise = require('sleep-promise')
const { initRustapi, allowedProtocolTypes } = require('../src/index')
const { createVcxAgent } = require('../src/index')
const logger = require('./logger')('Alice')
const { runScript } = require('./script-common')
const uuid = require('uuid')
const axios = require('axios')
const isPortReachable = require('is-port-reachable')
const url = require('url')
const { extractProofRequestAttachement } = require('../src/utils/proofs')

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
  const connectionId = 'alice-to-faber'
  const holderCredentialId = 'alice-credential'
  const disclosedProofId = 'alice-proof'
  const vcxAgent = await createVcxAgent({
    agentName,
    protocolType: options.protocolType,
    agencyUrl: 'http://localhost:8080',
    seed: '000000000000000000000000Trustee1',
    usePostgresWallet: false,
    logger
  })
  await vcxAgent.agentInitVcx()
  await vcxAgent.updateWebhookUrl(`http://localhost:7209/notifications/${agentName}`)

  const invitationString = await getInvitationString(options['autofetch-invitation-url'])
  const connectionToFaber = await vcxAgent.serviceConnections.inviteeConnectionAcceptFromInvitationAndProgress(connectionId, invitationString)

  if (!connectionToFaber) {
    throw Error('Connection with alice was not established.')
  }
  logger.info('Connection to alice was Accepted!')

  await vcxAgent.serviceCredHolder.waitForCredentialOfferAndAcceptAndProgress(connectionId, holderCredentialId)

  const proofRequest = (await vcxAgent.serviceProver.waitForProofRequests(connectionId))[0]
  if (!proofRequest) {
    throw Error('No proof request found.')
  }

  await vcxAgent.serviceProver.buildDisclosedProof(disclosedProofId, proofRequest)
  const requestInfo = extractProofRequestAttachement(proofRequest)
  logger.debug(`Proof request presentation attachment ${JSON.stringify(requestInfo, null, 2)}`)

  const selectedCreds = await vcxAgent.serviceProver.selectCredentials(disclosedProofId)
  const selfAttestedAttrs = { attribute_3: 'Smith' }
  await vcxAgent.serviceProver.generateProof(disclosedProofId, selectedCreds, selfAttestedAttrs)
  await vcxAgent.serviceProver.sendDisclosedProofAndProgress(disclosedProofId, connectionId)
  logger.info('Faber received the proof')

  await vcxAgent.agentShutdownVcx()
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
