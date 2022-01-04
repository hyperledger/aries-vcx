const { OutOfBandSender, OutOfBandReceiver } = require('@hyperledger/node-vcx-wrapper')

module.exports.createServiceOutOfBand = function createServiceOutOfBand ({ logger, saveConnection, loadConnection }) {
  async function createOobMsg (agent, label, message) {
    logger.info(`createOobMsg >>>`)
    const oob = await OutOfBandSender.create({ label })
    const service = await agent.getService()
    logger.info(`createOobMsg >>> appending service ${service}`)
    await oob.appendService(service)
    if (message) {
      const msgParsed = JSON.parse(message)
      const msgType = msgParsed["@type"]
      if (!msgType) {
        throw Error(`Message appended to OOB message must have @type. Invalid message: ${msgParsed}`)
      }
      logger.info(`createOobMsg >>> appending message of type ${msgType}`)
      await oob.appendMessage(message)
      logger.info(`createOobMsg >>> append message`)
    }
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
