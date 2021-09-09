const { OutOfBand } = require('@hyperledger/node-vcx-wrapper')

module.exports.createServiceOutOfBand = function createServiceOutOfBand ({ logger, saveConnection, loadConnection }) {
  async function createOobMsg (agent, label) {
    const oob = await OutOfBand.create({ label })
    const service = await agent.getService()
    await oob.appendService(service)
    return oob.serialize()
  }

  async function createConnectionFromOobMsg (connectionId, oobMsg) {
    const oob = await OutOfBand.createWithMessage(oobMsg)
    const connection = await oob.buildConnection()
    await saveConnection(connectionId, connection)
  }

  return {
    createOobMsg,
    createConnectionFromOobMsg
  }
}
