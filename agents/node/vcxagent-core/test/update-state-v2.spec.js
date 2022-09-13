/* eslint-env jest */
require('jest')
const { createPairedAliceAndFaber } = require('./utils/utils')
const { initRustapi } = require('../src/index')
const { IssuerStateType, HolderStateType, ProverStateType, VerifierStateType } = require('@hyperledger/node-vcx-wrapper')
const sleep = require('sleep-promise')

beforeAll(async () => {
  jest.setTimeout(1000 * 60 * 4)
  await initRustapi(process.env.VCX_LOG_LEVEL || 'vcx=warn')
})

describe('test update state', () => {
  it('Faber should send credential to Alice', async () => {
    try {
      const { alice, faber } = await createPairedAliceAndFaber()
      const tailsDir = `${__dirname}/tmp/faber/tails`
      await faber.buildLedgerPrimitivesV2({ tailsDir, maxCreds: 5 })
      await faber.sendCredentialOfferV2()
      await alice.acceptCredentialOffer()

      await faber.updateStateCredentialV2(IssuerStateType.RequestReceived)
      await faber.sendCredential()
      await alice.updateStateCredentialV2(HolderStateType.Finished)
      await faber.receiveCredentialAck()

      const request = await faber.requestProofFromAlice()
      await alice.sendHolderProof(JSON.parse(request), revRegId => tailsDir)
      await faber.updateStateVerifierProofV2(VerifierStateType.Finished)
      await alice.updateStateHolderProofV2(ProverStateType.Finished)
    } catch (err) {
      console.error(`err = ${err.message} stack = ${err.stack}`)
      await sleep(2000)
      throw Error(err)
    }
  })
})
