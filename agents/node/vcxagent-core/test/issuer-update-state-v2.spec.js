/* eslint-env jest */
require('jest')
const { initRustapi, protocolTypes } = require('../../vcxagent-core/vcx-workflows')
const { createVcxAgent } = require('../vcx-agent')
const { shutdownVcx, Connection, IssuerCredential } = require('@absaoss/node-vcx-wrapper')

beforeAll(async () => {
  jest.setTimeout(1000 * 60 * 4)
  await initRustapi(process.env.VCX_LOG_LEVEL || 'vcx=error')
})

async function createVcxControl (agentConfig, name) {
  const vcxControl = await createVcxAgent(agentConfig)
  await shutdownVcx()
  return vcxControl
}

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

  const faberVcxControl = await createVcxControl(faberAgentConfig, 'Faber')

  let connectionFaberToAlice, serConn, serCred

  async function createInvite () {
    logger.info('Faber is going to generate invite')
    await faberVcxControl.initVcx()
    const { invite } = await faberVcxControl.inviterConnectionCreateAndAccept(connectionIdAtFaber, undefined, true)
    logger.info(`Faber generated invite:\n${invite}`)
    logger.info('Shutting down Faber VCX session.')
    await shutdownVcx()

    return invite
  }

  async function sendCredential () {
    logger.info('Faber sending credential')
    await faberVcxControl.initVcx()

    logger.info('Faber writing schema on ledger')
    const schema = await faberVcxControl.createSchema()

    logger.info('Faber writing credential definition on ledger')
    await faberVcxControl.createCredentialDefinition(await schema.getSchemaId(), 'DemoCredential123', logger)

    logger.info('Faber sending credential to Alice')
    const schemaAttrs = {
      name: 'alice',
      last_name: 'clark',
      sex: 'female',
      date: '05-2018',
      degree: 'maths',
      age: '25'
    };
    ({ serCred, serConn } = await faberVcxControl.credentialIssue(schemaAttrs, 'DemoCredential123', connectionIdAtFaber, false, true))

    logger.info('Shutting down Faber VCX session.')
    await shutdownVcx()
  }

  async function updateStateCredentialV1 () {
    logger.info('Issuer updating state of credential')
    await faberVcxControl.initVcx()

    const issuerVcxCred = await IssuerCredential.deserialize(serCred)
    await expect(issuerVcxCred.updateState()).rejects.toThrow('Obj was not found with handle')

    logger.info('Shutting down Faber VCX session.')
    await shutdownVcx()
  }

  async function updateStateCredentialV2 () {
    logger.info('Issuer updating state of credential with connection')
    await faberVcxControl.initVcx()

    const issuerVcxCred = await IssuerCredential.deserialize(serCred)
    connectionFaberToAlice = await Connection.deserialize(serConn)
    await issuerVcxCred.updateStateV2(connectionFaberToAlice)
    expect(await issuerVcxCred.getState()).toBe(3)

    logger.info('Issuer sending credential')
    await issuerVcxCred.sendCredential(connectionFaberToAlice)
    logger.info('Credential sent')

    logger.info('Shutting down Faber VCX session.')
    await shutdownVcx()
  }

  return { createInvite, sendCredential, updateStateCredentialV1, updateStateCredentialV2 }
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

  async function acceptInvite (invite) {
    await aliceVcxControl.initVcx()
    logger.info('Reinitialized Alice VCX session.')

    logger.info('Alice is going to establish connection with Faber')
    await aliceVcxControl.inviteeConnectionAcceptFromInvitation(connectionIdAtAlice, invite, true)

    logger.info('Shutting down Alice VCX session.')
    await shutdownVcx()
  }

  async function acceptCredential () {
    await aliceVcxControl.initVcx()
    logger.info('Reinitialized Alice VCX session.')

    await aliceVcxControl.waitForCredentialOfferAndAccept(connectionIdAtAlice, true)

    logger.info('Shutting down Alice VCX session.')
    await shutdownVcx()
  }

  return { acceptInvite, acceptCredential }
}

describe('test update state', () => {
  it('Faber should fail to update state of the their credential via V1 API', async () => {
    const alice = await createAlice()
    const faber = await createFaber()
    const invite = await faber.createInvite()
    await alice.acceptInvite(invite)
    await faber.sendCredential()
    await alice.acceptCredential()
    await faber.updateStateCredentialV1()
  })

  it('Faber should send credential to Alice', async () => {
    const alice = await createAlice()
    const faber = await createFaber()
    const invite = await faber.createInvite()
    await alice.acceptInvite(invite)
    await faber.sendCredential()
    await alice.acceptCredential()
    await faber.updateStateCredentialV2()
  })
})
