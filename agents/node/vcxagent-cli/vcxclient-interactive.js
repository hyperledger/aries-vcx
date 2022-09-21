const readlineSync = require('readline-sync')
const { initRustapi, createVcxAgent } = require('@hyperledger/vcxagent-core')
const logger = require('./logger')('VCX Client')
const { getGenesisFile } = require('sovrin-networks')
const uuid = require('uuid')

async function createInteractiveClient (agentName, seed, acceptTaa, rustLogLevel, agencyUrl, indyNetwork) {
  logger.info(`Creating interactive client ${agentName} seed=${seed}, rustLogLevel=${rustLogLevel}, indyNetwork: ${indyNetwork}`)

  await initRustapi(rustLogLevel)
  const genesisPath = (indyNetwork) ? getGenesisFile(indyNetwork) : undefined
  console.log(`Resolved genesis path: ${genesisPath}`)
  const ariesAgent = await createVcxAgent({
    genesisPath,
    agentName,
    agencyUrl,
    seed,
    logger,
    rustLogLevel
  })
  await ariesAgent.agentInitVcx()

  if (acceptTaa) {
    await ariesAgent.acceptTaa()
  }

  const commands = {
    0: 'ACCEPT_TAA',
    1: 'CREATE_SCHEMA',
    2: 'CREATE_CRED_DEF',
    10: 'CONNECTION_INVITER_CREATE',
    11: 'CONNECTION_INVITEE_ACCEPT',
    12: 'CONNECTION_PROGRESS',
    13: 'CONNECTION_INFO',
    14: 'CONNECTIONS_INFO',
    20: 'GET_CREDENTIAL_OFFERS',
    21: 'ACCEPT_CREDENTIAL_OFFER',
    22: 'DISPLAY_CREDENTIAL',
    30: 'SEND_MESSAGE',
    31: 'GET_MESSAGE',
    32: 'GET_MESSAGE_V2'
  }

  while (true) {
    const cmd = readlineSync.question(`Select action: ${JSON.stringify(commands, null, 2)}\n`)
    if (cmd) {
      if (cmd === '0') {
        logger.info('Going to accept taa.\n')
        await ariesAgent.acceptTaa()
        logger.info('Taa accepted.\n')
      } else if (cmd === '10') {
        const connectionId = readlineSync.question('Enter connection id:\n')
        await ariesAgent.serviceConnections.inviterConnectionCreateAndAccept(connectionId, (invitationString) => {
          logger.info(`Connection ${connectionId} created. Invitation: ${invitationString}`)
        })
      } else if (cmd === '11') {
        const connectionId = readlineSync.question('Enter connection id:\n')
        let invitationString = readlineSync.question('Enter invitation:\n')
        try {
          const url = new URL(invitationString)
          const base64Invitation = url.searchParams.get('c_i') || url.searchParams.get('oob')
          if (base64Invitation) {
            invitationString = Buffer.from(base64Invitation, 'base64').toString()
          }
        } catch (err) {
          logger.debug("Invitation string is not URL, will assume it's an Aries message as JSON")
        }
        await ariesAgent.serviceConnections.inviteeConnectionAcceptFromInvitationAndProgress(connectionId, invitationString)
      } else if (cmd === '12') {
        const connectionId = readlineSync.question('Enter connection id:\n')
        await ariesAgent.serviceConnections.connectionAutoupdate(connectionId)
      } else if (cmd === '13') {
        const connectionId = readlineSync.question('Enter connection id:\n')
        await ariesAgent.serviceConnections.printInfo([connectionId])
      } else if (cmd === '14') {
        const connectionIds = await ariesAgent.serviceConnections.listIds()
        await ariesAgent.serviceConnections.printInfo(connectionIds)
      } else if (cmd === '20') {
        const connectionId = readlineSync.question('Enter connection id:\n')
        await ariesAgent.serviceCredHolder.waitForCredentialOffer(connectionId, null, 1, 0)
      } else if (cmd === '21') {
        const connectionId = readlineSync.question('Enter connection id:\n')
        const id = uuid.v4()
        await ariesAgent.serviceCredHolder.waitForCredentialOfferAndAcceptAndProgress(connectionId, id, null, 20, 1000)
        logger.info(`Received credential, assigned it local id ${id}`)
      } else if (cmd === '22') {
        const id = readlineSync.question('Enter credential id:\n')
        const data = await ariesAgent.serviceCredHolder.getCredentialData(id)
        logger.info(`Resolved credential object: ${JSON.stringify(data)}`)
      } else if (cmd === '30') {
        const connectionId = readlineSync.question('Enter connection id:\n')
        const message = readlineSync.question('Enter message to send:\n')
        await ariesAgent.serviceConnections.sendMessage(connectionId, message)
      } else if (cmd === '31') {
        const connectionId = readlineSync.question('Enter connection id:\n')
        const messages = await ariesAgent.serviceConnections.getMessages(connectionId, [], [])
        logger.info(`Found messages\n:${JSON.stringify(messages, null, 2)}`)
      } else if (cmd === '32') {
        const connectionId = readlineSync.question('Enter connection id:\n')
        const messages = await ariesAgent.serviceConnections.getMessagesV2(connectionId, [], [])
        logger.info(`Found messages\n:${JSON.stringify(messages, null, 2)}`)
      } else {
        logger.error(`Unknown command ${cmd}`)
      }
    }
  }
}

async function runInteractive (options) {
  logger.debug(`Going to build interactive client using options ${JSON.stringify(options)}`)
  const agentName = options.name || readlineSync.question('Enter agent\'s name:\n')
  await createInteractiveClient(agentName, options.seed, options.acceptTaa, options.rustLog, options.agencyUrl, options.indyNetwork)
}

module.exports.runInteractive = runInteractive
