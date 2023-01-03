const bodyParser = require('body-parser')
const sleep = require('sleep-promise')
const express = require('express')
const { createFaber } = require('./faber')
const { createAlice } = require('./alice')
const { ConnectionStateType } = require('@hyperledger/node-vcx-wrapper')

module.exports.createAliceAndFaber = async function createAliceAndFaber () {
  const alice = await createAlice()
  const faber = await createFaber()
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

module.exports.createPairedAliceAndFaberViaPublicInvite = async function createPairedAliceAndFaberViaPublicInvite () {
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

    const alice = await createAlice()
    const faber = await createFaber()
    const pwInfo = await faber.publishService(endpoint)
    const invite = await faber.createPublicInvite()
    await alice.acceptInvite(invite)
    const { message } = await faber.unpackMsg(encryptedMsg)
    await faber.createConnectionFromReceivedRequestV2(pwInfo, message)
    await alice.updateConnection(ConnectionStateType.Finished)
    await faber.updateConnection(ConnectionStateType.Finished)
    return { alice, faber }
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

module.exports.createPairedAliceAndFaberViaOobMsg = async function createPairedAliceAndFaberViaOobMsg () {
  let server
  try {
    const port = 5420
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

    const alice = await createAlice()
    const faber = await createFaber()
    const pwInfo = await faber.publishService(endpoint)
    const msg = await faber.createOobMessageWithDid()
    await alice.createConnectionUsingOobMessage(msg)
    await alice.updateConnection(ConnectionStateType.Requested)
    const { message } = await faber.unpackMsg(encryptedMsg)
    await faber.createConnectionFromReceivedRequestV2(pwInfo, message)
    await alice.updateConnection(ConnectionStateType.Finished)
    await faber.updateConnection(ConnectionStateType.Finished)
    return { alice, faber }
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

module.exports.connectViaOobMessage = async function connectViaOobMessage (alice, faber, msg) {
  let server
  try {
    const port = 5421
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

    const pwInfo = await faber.publishService(endpoint)
    await alice.createConnectionUsingOobMessage(msg)
    await alice.updateConnection(ConnectionStateType.Requested)
    const { message } = await faber.unpackMsg(encryptedMsg)
    await faber.createConnectionFromReceivedRequestV2(pwInfo, message)
    await alice.updateConnection(ConnectionStateType.Finished)
    await faber.updateConnection(ConnectionStateType.Finished)
    return { alice, faber }
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
