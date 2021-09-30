const { OutOfBandSender, OutOfBandReceiver } = require('@hyperledger/node-vcx-wrapper')

module.exports.createServiceOutOfBand = function createServiceOutOfBand ({ logger, saveConnection, loadConnection }) {
  async function createOobMsg (agent, label) {
    const oob = await OutOfBandSender.create({ label })
    const service = await agent.getService()
    await oob.appendService(service)
    return oob.toMessage()
  }

  async function createConnectionFromOobMsg (connectionId, oobMsg) {
    const oob = await OutOfBandReceiver.createWithMessage(oobMsg)
    const connection = await oob.buildConnection()
    await connection.connect('{}')
    await saveConnection(connectionId, connection)
  }

  return {
    createOobMsg,
    createConnectionFromOobMsg
  }
}
