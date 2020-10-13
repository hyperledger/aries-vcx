/* eslint-env jest */
require('jest')
const { shutdownVcx } = require('@absaoss/node-vcx-wrapper')
const { createPairedAliceAndFaber } = require('./utils/utils')
const { initRustapi } = require('../src/index')
const { StateType } = require('@absaoss/node-vcx-wrapper')
const sleep = require('sleep-promise')

beforeAll(async () => {
  jest.setTimeout(1000 * 60 * 4)
  await initRustapi(process.env.VCX_LOG_LEVEL || 'vcx=error')
})

describe('test update state', () => {
  it('Faber should fail to update state of the their credential via V1 API', async () => {
    try {
      const { alice, faber } = await createPairedAliceAndFaber()

      await faber.sendCredentialOffer()
      await alice.acceptCredentialOffer()
      await expect(faber.updateStateCredentialV1()).rejects.toThrow('Obj was not found with handle')
      await shutdownVcx()
    } catch (err) {
      console.error(`err = ${err.message} stack = ${err.stack}`)
      await sleep(2000)
      throw Error(err)
    }
  })

  it('Faber should send credential to Alice', async () => {
    try {
      const { alice, faber } = await createPairedAliceAndFaber()

      await faber.sendCredentialOffer()
      await alice.acceptCredentialOffer()

      await faber.updateStateCredentialV2(StateType.RequestReceived)
      await faber.sendCredential()
      await alice.updateStateCredentialV2(StateType.Accepted)

      const request = await faber.requestProofFromAlice()
      await alice.sendHolderProof(JSON.parse(request))
      await faber.updateStateVerifierProofV2(StateType.Accepted)
      await alice.updateStateHolderProofV2(StateType.Accepted)
    } catch (err) {
      console.error(`err = ${err.message} stack = ${err.stack}`)
      await sleep(2000)
      throw Error(err)
    }
  })
})
