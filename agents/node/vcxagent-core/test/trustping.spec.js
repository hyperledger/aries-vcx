/* eslint-env jest */
require('jest')
const { createPairedAliceAndFaber } = require('./utils/utils')
const { initRustapi } = require('../src/index')
const sleep = require('sleep-promise')

beforeAll(async () => {
  jest.setTimeout(1000 * 60 * 4)
  await initRustapi(process.env.VCX_LOG_LEVEL || 'vcx=error')
})

describe('send ping, get ping', () => {
  it('Faber should send credential to Alice', async () => {
    try {
      const { alice, faber } = await createPairedAliceAndFaber()
      await alice.sendPing()

      const faberMessages1 = await faber.downloadReceivedMessagesV2()
      expect(faberMessages1.length).toBe(1)
      expect(JSON.parse(faberMessages1[0].decryptedMsg)['@type'].match(/trust_ping\/1.0\/ping/))
      const pingMsgId = JSON.parse(faberMessages1[0].decryptedMsg)['@id']
      await faber.updateConnection(4) // sends ping_response for received ping
      const faberMessages2 = await faber.downloadReceivedMessagesV2()
      expect(faberMessages2.length).toBe(0)

      const aliceMessages1 = await alice.downloadReceivedMessagesV2()
      expect(aliceMessages1.length).toBe(1)
      expect(JSON.parse(aliceMessages1[0].decryptedMsg)['@type'].match(/trust_ping\/1.0\/ping_response/))
      expect(JSON.parse(aliceMessages1[0].decryptedMsg)['~thread'].thid).toBe(pingMsgId)
      await alice.updateConnection(4) // processes received ping_response
      const aliceMessages2 = await alice.downloadReceivedMessagesV2()
      expect(aliceMessages2.length).toBe(0)
    } catch (err) {
      console.error(`err = ${err.message} stack = ${err.stack}`)
      await sleep(2000)
      throw Error(err)
    }
  })
})
