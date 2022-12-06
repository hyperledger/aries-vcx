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
  const alice = await createAlice()
  const faber = await createFaber()
  const invite = await faber.createPublicInvite()
  await alice.acceptInvite(invite)
  await faber.createConnectionFromReceivedRequest()
  await alice.updateConnection(ConnectionStateType.Finished)
  await faber.updateConnection(ConnectionStateType.Finished)
  return { alice, faber }
}

module.exports.createPairedAliceAndFaberViaOobMsg = async function createPairedAliceAndFaberViaOobMsg (usePublicDid) {
  const alice = await createAlice()
  // const faber = await createFaber()
  const msg =   "{\"@type\": \"https:\/\/didcomm.org\/out-of-band\/1.0\/invitation\", \"requests~attach\": [],\"@id\": \"dd9b40f3-acf0-44e1-ad04-82e2cd38dc95\", \"services\": [\"did:sov:Qk6KsN9EQs9mJJarFAdkCV\"], \"label\": \"cliente\", \"handshake_protocols\": [\"https:\/\/didcomm.org\/didexchange\/1.0\", \"https:\/\/didcomm.org\/connections\/1.0\"]}";


  // (usePublicDid) ? await faber.createOobMessageWithDid() : await faber.createOobMessageWithService()
  await alice.createConnectionUsingOobMessage(msg)
  await alice.updateConnection(ConnectionStateType.Requested)
  // await faber.createConnectionFromReceivedRequest()
  await alice.updateConnection(ConnectionStateType.Finished)
  // await faber.updateConnection(ConnectionStateType.Finished)
  return { alice }
}

module.exports.connectViaOobMessage = async function connectViaOobMessage (alice, faber, msg) {
  await alice.createConnectionUsingOobMessage(msg)
  await alice.updateConnection(ConnectionStateType.Requested)
  await faber.createConnectionFromReceivedRequest()
  await alice.updateConnection(ConnectionStateType.Finished)
  await faber.updateConnection(ConnectionStateType.Finished)
  return { alice, faber }
}
