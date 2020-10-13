const { createFaber } = require('./faber')
const { createAlice } = require('./alice')
const { StateType } = require('@absaoss/node-vcx-wrapper')

module.exports.createPairedAliceAndFaber = async function createPairedAliceAndFaber () {
  const alice = await createAlice()
  const faber = await createFaber()
  const invite = await faber.createInvite()
  await alice.acceptInvite(invite)
  await faber.updateConnection(StateType.RequestReceived)
  await alice.updateConnection(StateType.Accepted)
  await faber.updateConnection(StateType.Accepted)
  return { alice, faber }
}
