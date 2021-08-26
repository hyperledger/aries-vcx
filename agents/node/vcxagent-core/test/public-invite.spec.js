/* eslint-env jest */
require('jest')
const { createPairedAliceAndFaberViaPublicInvite } = require('./utils/utils')
const { initRustapi } = require('../src/index')

beforeAll(async () => {
  jest.setTimeout(1000 * 60 * 4)
  await initRustapi(process.env.VCX_LOG_LEVEL || 'vcx=error')
})

describe('test public invite', () => {
  it('Establish connection via public invite, exchange messages', async () => {
      const { alice, faber } = await createPairedAliceAndFaberViaPublicInvite()
    }
  )
})
