/* eslint-env jest */
require('jest')
const sleep = require('sleep-promise')

const { initRustapi } = require('../src/index')
const { VerifierStateType } = require('@hyperledger/node-vcx-wrapper')
const { createPairedAliceAndFaberViaOobMsg, createAliceAndFaber, connectViaOobMessage, createPairedAliceAndFaber } = require('./utils/utils')
const { IssuerStateType, HolderStateType, OutOfBandReceiver } = require('@hyperledger/node-vcx-wrapper')

beforeAll(async () => {
  jest.setTimeout(1000 * 60 * 4)
  await initRustapi(process.env.VCX_LOG_LEVEL || 'trace')
})

describe('test out of band communication', () => {
  it('Faber establishes connection with Alice via DID OOB message', async () => {
    await createPairedAliceAndFaberViaOobMsg()
  })

  it('Faber establishes connection with Alice via OOB message and they exchange messages', async () => {
    const { alice, faber } = await createPairedAliceAndFaberViaOobMsg()

    await alice.sendMessage('Hello Faber')
    const msgsFaber = await faber.downloadReceivedMessagesV2()
    expect(msgsFaber.length).toBe(1)
    expect(msgsFaber[0].uid).toBeDefined()
    expect(msgsFaber[0].statusCode).toBe('MS-103')
    const payloadFaber = JSON.parse(msgsFaber[0].decryptedMsg)
    expect(payloadFaber['@id']).toBeDefined()
    expect(payloadFaber['@type']).toBeDefined()
    expect(payloadFaber.content).toBe('Hello Faber')

    await faber.sendMessage('Hello Alice')
    const msgsAlice = await alice.downloadReceivedMessagesV2()
    expect(msgsAlice.length).toBe(1)
    expect(msgsAlice[0].uid).toBeDefined()
    expect(msgsAlice[0].statusCode).toBe('MS-103')
    const payloadAlice = JSON.parse(msgsAlice[0].decryptedMsg)
    expect(payloadAlice['@id']).toBeDefined()
    expect(payloadAlice['@type']).toBeDefined()
    expect(payloadAlice.content).toBe('Hello Alice')
  })

  it('Alice reuses a connection already established by Faber', async () => {
    const { alice, faber } = await createPairedAliceAndFaberViaOobMsg()
    const msg = await faber.createOobMessageWithDid()
    const reused = await alice.createOrReuseConnectionUsingOobMsg(msg)
    expect(reused).toBe(true)
  })

  it('Faber issues credential via OOB', async () => {
    try {
      const { alice, faber } = await createAliceAndFaber()
      const tailsDir = `${__dirname}/tmp/faber/tails`
      await faber.buildLedgerPrimitives({ tailsDir, maxCreds: 5 })
      const oobCredOfferMsg = await faber.createOobCredOffer()

      await connectViaOobMessage(alice, faber, oobCredOfferMsg)

      await alice.acceptOobCredentialOffer(oobCredOfferMsg)
      await faber.updateStateCredential(IssuerStateType.RequestReceived)
      await faber.sendCredential()
      await alice.updateStateCredential(HolderStateType.Finished)
    } catch (e) {
      console.error(e.stack)
      await sleep(1000)
    }
  })

  it('Faber requests proof via OOB', async () => {
    try {
      const { alice, faber } = await createPairedAliceAndFaber()
      const tailsDir = `${__dirname}/tmp/faber/tails`
      await faber.buildLedgerPrimitives({ tailsDir, maxCreds: 5 })
      await faber.sendCredentialOffer()
      await alice.acceptCredentialOffer()
      await faber.updateStateCredential(IssuerStateType.RequestReceived)
      await faber.sendCredential()
      await alice.updateStateCredential(HolderStateType.Finished)

      const oobPresentationRequestMsg = await faber.createOobProofRequest()

      const oobReceiver = await OutOfBandReceiver.createWithMessage(oobPresentationRequestMsg)
      const presentationRequest = await oobReceiver.extractMessage()
      await alice.sendHolderProof(presentationRequest, revRegId => tailsDir)
      await faber.updateStateVerifierProof(VerifierStateType.Finished)
    } catch (e) {
      console.error(e.stack)
      await sleep(1000)
    }
  })
})
