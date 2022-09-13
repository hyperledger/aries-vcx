const readlineSync = require('readline-sync')
const { initRustapi, getSampleSchemaData, createVcxAgent } = require('@hyperledger/vcxagent-core')
const logger = require('./logger')('VCX Client')

async function createInteractiveClient (agentName, seed, acceptTaa, rustLogLevel, agencyUrl) {
  logger.info(`Creating interactive client ${agentName} seed=${seed}`)

  await initRustapi(rustLogLevel)

  const ariesAgent = await createVcxAgent({
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
        const invitationString = readlineSync.question('Enter invitation:\n')
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
  await createInteractiveClient(agentName, options.seed, options.acceptTaa, options.RUST_LOG, options.agencyUrl)
}

module.exports.runInteractive = runInteractive
