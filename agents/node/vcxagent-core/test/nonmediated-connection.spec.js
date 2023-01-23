/* eslint-env jest */
require('jest')
const bodyParser = require('body-parser')
const sleep = require('sleep-promise')
const express = require('express')
const { createAliceAndFaber } = require('./utils/utils')
const { initRustLogger } = require('../src')

beforeAll(async () => {
  jest.setTimeout(1000 * 60 * 4)
  initRustLogger(process.env.RUST_LOG || 'vcx=error')
})

describe('test establishing and exchanging messages via nonmediated connections', () => {
  it('Establish nonmediated connection via public endpoint using public invite, exchange messages', async () => {
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

      await alice.nonmediatedConnectionSendMessage('Hello Faber')
      const { message: msgFaber } = await faber.unpackMsg(faberEncryptedMsg)
      expect(JSON.parse(msgFaber).content).toBe('Hello Faber')

      await faber.nonmediatedConnectionSendMessage('Hello Alice')
      const { message: msgAlice } = await alice.unpackMsg(aliceEncryptedMsg)
      expect(JSON.parse(msgAlice).content).toBe('Hello Alice')

    } catch (err) {
      console.error(`err = ${err.message} stack = ${err.stack}`)
      throw Error(err)
    } finally {
      if (faberServer) {
        faberServer.close()
      }
      if (aliceServer) {
        aliceServer.close()
      }
      await sleep(2000)
    }
  })

  it('Establish nonmediated connection via public endpoint using pairwise invite, exchange messages', async () => {
    let faberServer
    try {
      const path = '/msg'
      const faberPort = 5402
      const faberEndpoint = `http://127.0.0.1:${faberPort}${path}`

      let faberEncryptedMsg
      const faberApp = express()
      faberApp.use(bodyParser.raw({ type: '*/*' }))
      faberApp.post(path, (req, res) => {
        faberEncryptedMsg = req.body
        res.status(200).send()
      })
      faberServer = faberApp.listen(faberPort)

      const alicePort = 5403
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

      const invite = await faber.createNonmediatedConnectionWithInvite()
      await alice.createNonmediatedConnectionFromInvite(invite)

      const { message: request } = await faber.unpackMsg(faberEncryptedMsg)
      await faber.nonmediatedConnectionProcessRequest(request)

      const { message: response } = await alice.unpackMsg(aliceEncryptedMsg)
      await alice.nonmediatedConnectionProcessResponse(response)

      const { message: ack } = await faber.unpackMsg(faberEncryptedMsg)
      await faber.nonmediatedConnectionProcessAck(ack)

      await alice.nonmediatedConnectionSendMessage('Hello Faber')
      const { message: msgFaber } = await faber.unpackMsg(faberEncryptedMsg)
      expect(JSON.parse(msgFaber).content).toBe('Hello Faber')

      await faber.nonmediatedConnectionSendMessage('Hello Alice')
      const { message: msgAlice } = await alice.unpackMsg(aliceEncryptedMsg)
      expect(JSON.parse(msgAlice).content).toBe('Hello Alice')

    } catch (err) {
      console.error(`err = ${err.message} stack = ${err.stack}`)
      throw Error(err)
    } finally {
      if (faberServer) {
        faberServer.close()
      }
      if (aliceServer) {
        aliceServer.close()
      }
      await sleep(2000)
    }
  })

  it('Establish nonmediated connection via public endpoint using OOB invite, exchange messages', async () => {
    let faberServer
    try {
      const path = '/msg'
      const faberPort = 5404
      const faberEndpoint = `http://127.0.0.1:${faberPort}${path}`

      let faberEncryptedMsg
      const faberApp = express()
      faberApp.use(bodyParser.raw({ type: '*/*' }))
      faberApp.post(path, (req, res) => {
        faberEncryptedMsg = req.body
        res.status(200).send()
      })
      faberServer = faberApp.listen(faberPort)

      const alicePort = 5405
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

      const pwInfo = await faber.publishService(faberEndpoint)
      const msg = await faber.createOobMessageWithDid()
      await alice.createNonmediatedConnectionUsingOobMessage(msg)

      const { message: request } = await faber.unpackMsg(faberEncryptedMsg)
      await faber.createNonmediatedConnectionFromRequest(request, pwInfo)

      const { message: response } = await alice.unpackMsg(aliceEncryptedMsg)
      await alice.nonmediatedConnectionProcessResponse(response)

      const { message: ack } = await faber.unpackMsg(faberEncryptedMsg)
      await faber.nonmediatedConnectionProcessAck(ack)

      await alice.nonmediatedConnectionSendMessage('Hello Faber')
      const { message: msgFaber } = await faber.unpackMsg(faberEncryptedMsg)
      console.log(`msgFaber = ${msgFaber}`)
      expect(JSON.parse(msgFaber).content).toBe('Hello Faber')

      await faber.nonmediatedConnectionSendMessage('Hello Alice')
      const { message: msgAlice } = await alice.unpackMsg(aliceEncryptedMsg)
      expect(JSON.parse(msgAlice).content).toBe('Hello Alice')

    } catch (err) {
      console.error(`err = ${err.message} stack = ${err.stack}`)
      throw Error(err)
    } finally {
      if (faberServer) {
        faberServer.close()
      }
      if (aliceServer) {
        aliceServer.close()
      }
      await sleep(2000)
    }
  })
})
