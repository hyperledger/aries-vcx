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

describe('test establishing and exchanging messages via nonmediated connections', () => {
  it('Establish nonmediated connection via public endpoint, exchange messages', async () => {
    let faberServer
    try {
      const path = '/msg'
      const faberPort = 5400
      const faberEndpoint = `http://127.0.0.1:${faberPort}${path}`

      let faberEncryptedMsg
      const faberApp = express()
      faberApp.use(bodyParser.raw({ type: '*/*' }))
      faberApp.post(path, (req, res) => {
        faberEncryptedMsg = req.body
        res.status(200).send()
      })
      faberServer = faberApp.listen(faberPort)

      const alicePort = 5401
      const aliceEndpoint = `http://127.0.0.1:${alicePort}${path}`

      let aliceEncryptedMsg
      const aliceApp = express()
      aliceApp.use(bodyParser.raw({ type: '*/*' }))
      aliceApp.post(path, (req, res) => {
        aliceEncryptedMsg = req.body
        res.status(200).send()
      })
      aliceServer = aliceApp.listen(alicePort)

      const { alice, faber } = await createAliceAndFaber({ aliceEndpoint, faberEndpoint })
      const invite = await faber.createPublicInvite()
      const pwInfo = await faber.publishService(faberEndpoint)
      const service = await faber.readServiceFromLedger()
      expect(service.recipientKeys[0]).toBe(pwInfo.pw_vk)

      await alice.createNonmediatedConnectionFromInvite(invite)
      const { message: request } = await faber.unpackMsg(faberEncryptedMsg)

      await faber.createNonmediatedConnectionFromRequest(request, pwInfo)
      const { message: response } = await alice.unpackMsg(aliceEncryptedMsg)

      await alice.nonmediatedConnectionProcessResponse(response)
      const { message: ack } = await faber.unpackMsg(faberEncryptedMsg)

      await faber.nonmediatedConnectionProcessAck(ack)
    } catch (err) {
      console.error(`err = ${err.message} stack = ${err.stack}`)
      await sleep(2000)
      throw Error(err)
    } finally {
      if (faberServer) {
        faberServer.close()
      }
      if (aliceServer) {
        aliceServer.close()
      }
    }
  })
})
