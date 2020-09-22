/* eslint-env jest */
require('jest')
const { initRustapi, protocolTypes } = require('../../vcxagent-core/vcx-workflows')
const { createVcxAgent } = require('../vcx-agent')
const { shutdownVcx, Connection, IssuerCredential, Proof, DisclosedProof, Credential, StateType } = require('@absaoss/node-vcx-wrapper')
const { getAliceSchemaAttrs, getFaberCredDefName, getFaberProofData } = require('./data')

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

  let serConn, serCred, serProof

  async function createInvite () {
    logger.info('Faber is going to generate invite')
    await faberVcxControl.initVcx()

    const { invite, connection } = await faberVcxControl.inviterConnectionCreateAndAccept(connectionIdAtFaber, undefined, true)
    serConn = await connection.serialize()
    expect(await connection.getState()).toBe(StateType.OfferSent)
    logger.info(`Faber generated invite:\n${invite}`)

    logger.info('Shutting down Faber VCX session.')
    await shutdownVcx()

    return invite
  }

  async function sendConnectionResponse () {
    logger.info('Faber is going to generate invite')
    await faberVcxControl.initVcx()

    const connectionFaberToAlice = await Connection.deserialize(serConn)
    await connectionFaberToAlice.updateState()
    serConn = await connectionFaberToAlice.serialize()
    expect(await connectionFaberToAlice.getState()).toBe(StateType.RequestReceived)

    logger.info('Shutting down Faber VCX session.')
    await shutdownVcx()
  }

  async function finishConnection () {
    logger.info('Faber finishing estabilishing connection with Faber')

    await faberVcxControl.initVcx()
    logger.info('Reinitialized Faber VCX session.')

    const connectionFaberToAlice = await Connection.deserialize(serConn)
    await connectionFaberToAlice.updateState()
    expect(await connectionFaberToAlice.getState()).toBe(StateType.Accepted)
    serConn = await connectionFaberToAlice.serialize()

    logger.info('Shutting down Faber VCX session.')
    await shutdownVcx()
  }

  async function sendCredentialOffer () {
    logger.info('Faber sending credential')
    await faberVcxControl.initVcx()

    logger.info('Faber writing schema on ledger')
    const schema = await faberVcxControl.createSchema()

    logger.info('Faber writing credential definition on ledger')
    await faberVcxControl.createCredentialDefinition(await schema.getSchemaId(), getFaberCredDefName(), logger)

    logger.info('Faber sending credential to Alice')
    const schemaAttrs = getAliceSchemaAttrs()
    const connectionFaberToAlice = await Connection.deserialize(serConn)
    const { issuerCred, connectionToReceiver } = await faberVcxControl.sendOffer({ schemaAttrs, credDefName: getFaberCredDefName(), connection: connectionFaberToAlice, skipProgress: true })
    serCred = await issuerCred.serialize()
    serConn = await connectionToReceiver.serialize()

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
    const connectionFaberToAlice = await Connection.deserialize(serConn)
    await issuerVcxCred.updateStateV2(connectionFaberToAlice)
    expect(await issuerVcxCred.getState()).toBe(3)

    logger.info('Issuer sending credential')
    await issuerVcxCred.sendCredential(connectionFaberToAlice)
    logger.info('Credential sent')
    expect(await issuerVcxCred.getState()).toBe(4)

    logger.info('Shutting down Faber VCX session.')
    await shutdownVcx()
  }

  async function requestProofFromAlice () {
    logger.info('Faber going to request  proof from Alice\nCreating Proof object')
    await faberVcxControl.initVcx()

    const verifierProof = await faberVcxControl.verifierCreateProof(getFaberProofData(faberVcxControl.getInstitutionDid()))
    logger.info('Requesting proof from alice')
    const connectionFaberToAlice = await Connection.deserialize(serConn)
    const request = await faberVcxControl.verifierCreateProofRequest(connectionFaberToAlice, verifierProof)
    expect(await verifierProof.getState()).toBe(StateType.OfferSent)

    serProof = await verifierProof.serialize()

    logger.info('Shutting down Faber VCX session.')
    await shutdownVcx()

    return request
  }

  async function updateStateVerifierProofV2 () {
    logger.info('Verifier updating state of proof')
    await faberVcxControl.initVcx()

    const connectionFaberToAlice = await Connection.deserialize(serConn)
    expect(await connectionFaberToAlice.getState()).toBe(StateType.Accepted)
    const verifierProof = await Proof.deserialize(serProof)
    await verifierProof.updateStateV2(connectionFaberToAlice)
    expect(await verifierProof.getState()).toBe(StateType.Accepted)

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

  let serConn, serCred, serProof

  async function acceptInvite (invite) {
    logger.info('Alice estabilishing connection with Faber')

    await aliceVcxControl.initVcx()
    logger.info('Reinitialized Alice VCX session.')

    const connectionAliceToFaber = await aliceVcxControl.inviteeConnectionAcceptFromInvitation(connectionIdAtAlice, invite, true)
    await connectionAliceToFaber.updateState()
    expect(await connectionAliceToFaber.getState()).toBe(StateType.RequestReceived)
    serConn = await connectionAliceToFaber.serialize()

    logger.info('Shutting down Alice VCX session.')
    await shutdownVcx()
  }

  async function finishConnection () {
    logger.info('Alice finish estabilishing connection with Faber')

    await aliceVcxControl.initVcx()
    logger.info('Reinitialized Alice VCX session.')

    const connectionAliceToFaber = await Connection.deserialize(serConn)
    await connectionAliceToFaber.updateState()
    expect(await connectionAliceToFaber.getState()).toBe(StateType.Accepted)
    serConn = await connectionAliceToFaber.serialize()

    logger.info('Shutting down Alice VCX session.')
    await shutdownVcx()
  }

  async function acceptCredentialOffer () {
    logger.info('Alice accepting creadential offer')
    await aliceVcxControl.initVcx()
    logger.info('Reinitialized Alice VCX session.')

    const connectionAliceToFaber = await Connection.deserialize(serConn);
    ({ serCred } = await aliceVcxControl.waitForCredentialOfferAndAccept({ connection: connectionAliceToFaber, skipProgress: true }))

    logger.info('Shutting down Alice VCX session.')
    await shutdownVcx()
  }

  async function sendHolderProof (request) {
    logger.info('Alice creating and sending proof')

    await aliceVcxControl.initVcx()
    logger.info('Reinitialized Alice VCX session.')

    const connectionAliceToFaber = await Connection.deserialize(serConn)
    const holderProof = await DisclosedProof.create({ sourceId: 'proof', request })
    const selectedCreds = await aliceVcxControl.holderSelectCredentialsForProof(holderProof)
    const selfAttestedAttrs = { attribute_3: 'Smith' }
    await holderProof.generateProof({ selectedCreds, selfAttestedAttrs })
    await holderProof.sendProof(connectionAliceToFaber)
    expect(await holderProof.getState()).toBe(StateType.OfferSent)
    serProof = await holderProof.serialize()

    logger.info('Shutting down Alice VCX session.')
    await shutdownVcx()

    return holderProof
  }

  async function updateStateHolderProofV2 () {
    logger.info('Holder updating state of proof')
    await aliceVcxControl.initVcx()

    const holderProof = await DisclosedProof.deserialize(serProof)
    const connectionAliceToFaber = await Connection.deserialize(serConn)
    await holderProof.updateStateV2(connectionAliceToFaber)
    expect(await holderProof.getState()).toBe(StateType.Accepted)

    logger.info('Shutting down Alice VCX session.')
    await shutdownVcx()
  }

  async function updateStateCredentialV2 () {
    logger.info('Holder updating state of credential with connection')
    await aliceVcxControl.initVcx()

    const holderVcxCred = await Credential.deserialize(serCred)
    const connectionAliceToFaber = await Connection.deserialize(serConn)
    expect(await holderVcxCred.getState()).toBe(StateType.OfferSent)
    await holderVcxCred.updateStateV2(connectionAliceToFaber)
    expect(await holderVcxCred.getState()).toBe(StateType.Accepted)

    logger.info('Shutting down Alice VCX session.')
    await shutdownVcx()
  }

  return { acceptInvite, finishConnection, acceptCredentialOffer, updateStateCredentialV2, sendHolderProof, updateStateHolderProofV2 }
}

describe('test update state', () => {
  it('Faber should fail to update state of the their credential via V1 API', async () => {
    const alice = await createAlice()
    const faber = await createFaber()
    const invite = await faber.createInvite()
    await alice.acceptInvite(invite)
    await faber.sendCredentialOffer()
    await alice.acceptCredentialOffer()
    await faber.updateStateCredentialV1()
  })

  it('Faber should send credential to Alice', async () => {
    const alice = await createAlice()
    const faber = await createFaber()
    const invite = await faber.createInvite()
    await alice.acceptInvite(invite)
    await faber.sendConnectionResponse()
    await alice.finishConnection()
    await faber.finishConnection()

    await faber.sendCredentialOffer()
    await alice.acceptCredentialOffer()
    await faber.updateStateCredentialV2()
    await alice.updateStateCredentialV2()

    const request = await faber.requestProofFromAlice()
    await alice.sendHolderProof(request)
    await faber.updateStateVerifierProofV2()
    await alice.updateStateHolderProofV2()
  })
})
