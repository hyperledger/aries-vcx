/* eslint-env jest */
require('jest')
const { createPairedAliceAndFaber } = require('./utils/utils')
const { initRustapi } = require('../src/index')
const sleep = require('sleep-promise')

beforeAll(async () => {
  jest.setTimeout(1000 * 60 * 4)
  await initRustapi(process.env.VCX_LOG_LEVEL || 'vcx=error')
})

describe('test messaging', () => {
  it('Alice should send message and Faber download it', async () => {
    const { alice, faber } = await createPairedAliceAndFaber()
    await alice.sendMessage("HelloFaber")
    let msgs = await faber.downloadReceivedMessages()
    expect(msgs.length).toBe(1)
    expect(msgs[0].uid).toBeDefined()
    expect(msgs[0].statusCode).toBe("MS-103")
    const payload = JSON.parse(msgs[0].decryptedPayload)
    expect(payload["@id"]).toBeDefined()
    expect(payload["@type"]).toBeDefined()
    expect(payload["content"]).toBe("HelloFaber")
  })
})
