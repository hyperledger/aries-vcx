/* eslint-env jest */
require('jest')
const { createPairedAliceAndFaber } = require('./utils/alice')
const { initRustapi } = require('../src/index')

beforeAll(async () => {
  jest.setTimeout(1000 * 60 * 4)
  await initRustapi(process.env.VCX_LOG_LEVEL || 'vcx=error')
})

describe('test update state', () => {
  it('Faber should fail to update state of the their credential via V1 API', async () => {
    const { alice, faber } = await createPairedAliceAndFaber()

    const signature = await alice.signData('foobar')
    await faber.verifySignature('foobar', signature)
  })
})
