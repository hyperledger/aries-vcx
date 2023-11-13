/* eslint-env jest */
require('jest')
const { createPairedAliceAndFaberViaPublicInvite } = require('./utils/utils')
const { initRustLogger } = require('../src')

jest.setTimeout(1000 * 60 * 4)

beforeAll(async () => {
  initRustLogger(process.env.RUST_LOG || 'vcx=error')
})

describe('test public invite', () => {
  it('Establish connection via public invite, exchange messages', async () => {
    const { alice, faber } = await createPairedAliceAndFaberViaPublicInvite()

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
})
