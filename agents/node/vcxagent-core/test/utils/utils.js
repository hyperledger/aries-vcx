const bodyParser = require('body-parser')
const sleep = require('sleep-promise')
const express = require('express')
const { createFaber } = require('./faber')
const { createAlice } = require('./alice')
const { ConnectionStateType } = require('@hyperledger/node-vcx-wrapper')
const assert = require('assert')

module.exports.createAliceAndFaber = async function createAliceAndFaber ({ aliceEndpoint, faberEndpoint } = {}) {
  const alice = await createAlice(aliceEndpoint)
  const faber = await createFaber(faberEndpoint)
  return { alice, faber }
}

module.exports.createPairedAliceAndFaber = async function createPairedAliceAndFaber () {
  const alice = await createAlice()
  const faber = await createFaber()
  const invite = await faber.createInvite()
  await alice.acceptInvite(invite)
  await faber.updateConnection(ConnectionStateType.Responded)
  await alice.updateConnection(ConnectionStateType.Finished)
  await faber.updateConnection(ConnectionStateType.Finished)
  return { alice, faber }
}

async function executeFunctionWithServer (f1, f2) {
  let server
  try {
    const port = 5419
    const path = '/msg'
    const endpoint = `http://127.0.0.1:${port}${path}`

    let encryptedMsg
    const app = express()
    app.use(bodyParser.raw({ type: '*/*' }))
    app.post(path, (req, res) => {
      encryptedMsg = req.body
      res.status(200).send()
    })
    server = app.listen(port)

    const { alice, faber, pwInfo } = await f1(endpoint)
    await sleep(150)
    assert(encryptedMsg, "It seems that no message has yet arrived on faber's endpoint, try to increase timeout")
    const { message } = await faber.unpackMsg(encryptedMsg)
    return await f2(alice, faber, pwInfo, message)
  } catch (err) {
    console.error(`err = ${err.message} stack = ${err.stack}`)
    await sleep(2000)
    throw Error(err)
  } finally {
    if (server) {
      server.close()
      await sleep(3000)
    }
  }
}

module.exports.createPairedAliceAndFaberViaPublicInvite = async function createPairedAliceAndFaberViaPublicInvite () {
  const f1 = async (endpoint) => {
    const alice = await createAlice()
    const faber = await createFaber()
    const pwInfo = await faber.publishService(endpoint)
    const invite = await faber.createPublicInvite()
    await alice.acceptInvite(invite)
    return { alice, faber, pwInfo }
  }
  const f2 = async (alice, faber, pwInfo, message) => {
    await faber.createConnectionFromReceivedRequestV2(pwInfo, message)
    await alice.updateConnection(ConnectionStateType.Finished)
    await faber.updateConnection(ConnectionStateType.Finished)
    return { alice, faber }
  }
  return await executeFunctionWithServer(f1, f2)
}

module.exports.createPairedAliceAndFaberViaOobMsg = async function createPairedAliceAndFaberViaOobMsg () {
  const f1 = async (endpoint) => {
    const alice = await createAlice()
    const faber = await createFaber()
    const pwInfo = await faber.publishService(endpoint)
    const msg = await faber.createOobMessageWithDid()
    await alice.createConnectionUsingOobMessage(msg)
    await alice.updateConnection(ConnectionStateType.Requested)
    return { alice, faber, pwInfo }
  }
  const f2 = async (alice, faber, pwInfo, message) => {
    await faber.createConnectionFromReceivedRequestV2(pwInfo, message)
    await alice.updateConnection(ConnectionStateType.Finished)
    await faber.updateConnection(ConnectionStateType.Finished)
    return { alice, faber }
  }
  return await executeFunctionWithServer(f1, f2)
}

module.exports.connectViaOobMessage = async function connectViaOobMessage (alice, faber, msg) {
  const f1 = async (endpoint) => {
    const pwInfo = await faber.publishService(endpoint)
    await alice.createConnectionUsingOobMessage(msg)
    await alice.updateConnection(ConnectionStateType.Requested)
    return { alice, faber, pwInfo }
  }
  const f2 = async (alice, faber, pwInfo, message) => {
    await faber.createConnectionFromReceivedRequestV2(pwInfo, message)
    await alice.updateConnection(ConnectionStateType.Finished)
    await faber.updateConnection(ConnectionStateType.Finished)
    return { alice, faber }
  }
  return await executeFunctionWithServer(f1, f2)
}
