/* eslint-env jest */
require('jest')
const bodyParser = require('body-parser')
const sleep = require('sleep-promise')
const express = require('express')
const { ConnectionStateType } = require('@hyperledger/node-vcx-wrapper')
const { createAliceAndFaber } = require('./utils/utils')
const { initRustapi } = require('../src/index')

beforeAll(async () => {
  jest.setTimeout(1000 * 60 * 4)
  await initRustapi(process.env.VCX_LOG_LEVEL || 'vcx=error')
})

describe('test connecting via unmediated endpoint', () => {
  it('Establish connection via public nonmediated endpoint, exchange messages', async () => {
    let server
    try {
      const port = 5421
      const path = '/msg'
      const endpoint = `http://127.0.0.1:${port}${path}`

      const { alice, faber } = await createAliceAndFaber()
      const invite = await faber.createPublicInvite()
      const pwVk = await faber.publishService(endpoint)

      let msg
      const app = express()
      app.use(bodyParser.raw({ type: '*/*' }))
      app.post(path, (req, res) => {
        msg = req.body
        res.status(200).send()
      })
      server = app.listen(port)

      await alice.acceptInvite(invite)
      const { message } = await faber.unpackMsg(msg)
      await faber.createConnectionFromReceivedRequest(pwVk, message)
      await alice.updateConnection(ConnectionStateType.Finished)
      await faber.updateConnection(ConnectionStateType.Finished)
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
