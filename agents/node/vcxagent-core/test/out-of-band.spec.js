/* eslint-env jest */
require('jest')

const { initRustapi } = require('../src/index')
const { createPairedAliceAndFaberViaOobMsg } = require('./utils/utils')

beforeAll(async () => {
  jest.setTimeout(1000 * 60 * 4)
  await initRustapi(process.env.VCX_LOG_LEVEL || 'vcx=error')
})

describe('test out of band communication', () => {
  it('Faber establishes connection with Alice via OOB message and they exchange messages', async () => {
    const { alice, faber } = await createPairedAliceAndFaberViaOobMsg()
  })
})
