/* eslint-env jest */
require('jest')
const { initRustapi, protocolTypes } = require('../src/index')
const { createVcxAgent } = require('../src/index')
const { shutdownVcx, Connection, StateType } = require('@absaoss/node-vcx-wrapper')

beforeAll(async () => {
  jest.setTimeout(1000 * 60 * 4)
  await initRustapi(process.env.VCX_LOG_LEVEL || 'vcx=error')
})


async function createFaber () {
  const faberId = `faber-${Math.floor(new Date() / 1000)}`
  const connectionIdAtFaber = `connectionAt-${faberId}`
  const logger = require('../../vcxagent-cli/logger')('Faber')
  const faberAgentConfig = {
    agentName: faberId,
    protocolType: protocolTypes.v4,
    agencyUrl: 'http://localhost:8080',
    seed: '000000000000000000000000Trustee1',
    webhookUrl: `http://localhost:7209/notifications/${faberId}`,
    usePostgresWallet: false,
    logger
  }

  let agentFaber = await createVcxAgent(faberAgentConfig)
  await shutdownVcx()

  async function createInvite () {
    logger.info('Faber is going to generate invite')
    await agentFaber.initVcx()

    const { invite, connection } = await agentFaber.inviterConnectionCreate(connectionIdAtFaber)
    expect(await connection.getState()).toBe(StateType.OfferSent)
    logger.info(`Faber generated invite:\n${invite}`)

    logger.info('Shutting down Faber VCX session.')
    await shutdownVcx()

    return invite
  }

  async function sendConnectionResponse () {
    logger.info('Faber is going to generate invite')
    await agentFaber.initVcx()

    const connectionFaberToAlice = await Connection.deserialize(serConn)
    expect(await connectionFaberToAlice.updateState()).toBe(StateType.RequestReceived)
    serConn = await connectionFaberToAlice.serialize()

    logger.info('Shutting down Faber VCX session.')
    await shutdownVcx()
  }

  async function finishConnection () {
    logger.info('Faber finishing estabilishing connection with Faber')

    await agentFaber.initVcx()
    logger.info('Reinitialized Faber VCX session.')

    const connectionFaberToAlice = await Connection.deserialize(serConn)
    expect(await connectionFaberToAlice.updateState()).toBe(StateType.Accepted)
    serConn = await connectionFaberToAlice.serialize()

    logger.info('Shutting down Faber VCX session.')
    await shutdownVcx()
  }

  return { createInvite, sendConnectionResponse, finishConnection, sendCredentialOffer, updateStateCredentialV1, updateStateCredentialV2, requestProofFromAlice, updateStateVerifierProofV2 }
}

async function createAlice () {
  const aliceId = `alice-${Math.floor(new Date() / 1000)}`
  const connectionIdAtAlice = `connectionAt-${aliceId}`
  const logger = require('../../vcxagent-cli/logger')('Alice')
  const aliceAgentConfig = {
    agentName: aliceId,
    protocolType: protocolTypes.v4,
    agencyUrl: 'http://localhost:8080',
    seed: '000000000000000000000000Alice000',
    webhookUrl: `http://localhost:7209/notifications/${aliceId}`,
    usePostgresWallet: false,
    logger
  }
  const aliceVcxControl = await createVcxControl(aliceAgentConfig, 'Alice')

  let serConn

  async function acceptInvite (invite) {
    logger.info('Alice estabilishing connection with Faber')

    await aliceVcxControl.initVcx()
    logger.info('Reinitialized Alice VCX session.')

    const connectionAliceToFaber = await aliceVcxControl.inviteeConnectionAcceptFromInvitation(connectionIdAtAlice, invite, true)
    expect(await connectionAliceToFaber.updateState()).toBe(StateType.RequestReceived)
    serConn = await connectionAliceToFaber.serialize()

    logger.info('Shutting down Alice VCX session.')
    await shutdownVcx()
  }

  async function finishConnection () {
    logger.info('Alice finish estabilishing connection with Faber')

    await aliceVcxControl.initVcx()
    logger.info('Reinitialized Alice VCX session.')

    const connectionAliceToFaber = await Connection.deserialize(serConn)
    expect(await connectionAliceToFaber.updateState()).toBe(StateType.Accepted)
    serConn = await connectionAliceToFaber.serialize()

    logger.info('Shutting down Alice VCX session.')
    await shutdownVcx()
  }

  return { acceptInvite, finishConnection }
}

describe('test update state', () => {
  it('Faber should send credential to Alice', async () => {
    const alice = await createAlice()
    const faber = await createFaber()
    const invite = await faber.createInvite()
    await alice.acceptInvite(invite)
    await faber.sendConnectionResponse()
    await alice.finishConnection()
    await faber.finishConnection()
  })
})
