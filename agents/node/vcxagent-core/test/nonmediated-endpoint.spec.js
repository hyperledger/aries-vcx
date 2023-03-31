/* eslint-env jest */
require('jest')
const bodyParser = require('body-parser')
const sleep = require('sleep-promise')
const express = require('express')
const { ConnectionStateType } = require('@hyperledger/node-vcx-wrapper')
const { createAliceAndFaber } = require('./utils/utils')
const { initRustLogger } = require('../src')

beforeAll(async () => {
  jest.setTimeout(1000 * 60 * 4)
  initRustLogger(process.env.RUST_LOG || 'vcx=error')
})

describe('test connecting via unmediated endpoint', () => {
  it('Establish connection via public nonmediated endpoint, exchange messages', async () => {
    let server
    try {
      const port = 5422
      const path = '/msg'
      const endpoint = `http://127.0.0.1:${port}${path}`

      const { alice, faber } = await createAliceAndFaber()
      const invite = await faber.createPublicInvite()
      const pwInfo = await faber.publishService(endpoint)
      const service = await faber.readServiceFromLedger()
      expect(service.recipientKeys[0]).toBe(pwInfo.pw_vk)

      let encryptedMsg
      const app = express()
      app.use(bodyParser.raw({ type: '*/*' }))
      app.post(path, (req, res) => {
        encryptedMsg = req.body
        res.status(200).send()
      })
      server = app.listen(port)

      await alice.acceptInvite(invite)
      const { message } = await faber.unpackMsg(encryptedMsg)
      await faber.createConnectionFromReceivedRequestV2(pwInfo, message)
      await alice.updateConnection(ConnectionStateType.Finished)
      await faber.updateConnection(ConnectionStateType.Finished)

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
    } catch (err) {
      console.error(`err = ${err.message} stack = ${err.stack}`)
      await sleep(2000)
      throw Error(err)
    } finally {
      if (server) {
        server.close()
      }
    }
  })
})
