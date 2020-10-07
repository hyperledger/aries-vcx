/* eslint-env jest */
require('jest')
const {createPairedAliceAndFaber} = require('./utils/utils')
const { initRustapi } = require('../src/index')
const { StateType } = require('@absaoss/node-vcx-wrapper')

beforeAll(async () => {
  jest.setTimeout(1000 * 60 * 4)
  await initRustapi(process.env.VCX_LOG_LEVEL || 'vcx=error')
})

describe('test update state', () => {
  it('Faber should fail to update state of the their credential via V1 API', async () => {
    const { alice, faber } = await createPairedAliceAndFaber()

    await faber.sendCredentialOffer()
    await alice.acceptCredentialOffer()
    await faber.updateStateCredentialV1()
  })

  it('Faber should send credential to Alice', async () => {
    const { alice, faber } = await createPairedAliceAndFaber()

    await faber.sendCredentialOffer()
    await alice.acceptCredentialOffer()

    await faber.updateStateCredentialV2(StateType.RequestReceived)
    await faber.sendCredential()
    await alice.updateStateCredentialV2(StateType.Accepted)

    const request = await faber.requestProofFromAlice()
    await alice.sendHolderProof(request)
    await faber.updateStateVerifierProofV2(StateType.Accepted)
    await alice.updateStateHolderProofV2(StateType.Accepted)
  })
})
