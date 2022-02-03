const { OutOfBandSender, OutOfBandReceiver } = require('@hyperledger/node-vcx-wrapper')

module.exports.createServiceOutOfBand = function createServiceOutOfBand ({ logger, saveConnection, loadConnection }) {

  async function _createOobSender(message, label) {
    logger.info(`createOobMsg >>>`)
    const oob = await OutOfBandSender.create({ label })
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
    return oob
  }

  async function createOobMessageWithService (message, label, service) {
    const oobSender = await _createOobSender(message, label)
    await oobSender.appendService(service)
    return await oobSender.toMessage()
  }

  async function createOobMessageWithDid (message, label, publicDid) {
    const oobSender = await _createOobSender(message, label)
    await oobSender.appendServiceDid(publicDid)
    return await oobSender.toMessage()
  }

  async function createConnectionFromOobMsg (connectionId, oobMsg) {
    const oob = await OutOfBandReceiver.createWithMessage(oobMsg)
    const connection = await oob.buildConnection()
    await connection.connect('{}')
    await saveConnection(connectionId, connection)
  }

  async function reuseConnectionFromOobMsg (connectionId, oobMsg) {
    const oob = await OutOfBandReceiver.createWithMessage(oobMsg)
    const connection1 = await loadConnection(connectionId)
    const connection2 = await oob.connectionExists([connection1])
    if (!connection2) { throw Error('Connection implied in the OOB message was not established yet') }
    await connection1.sendHandshakeReuse(await oob.getThreadId())
  }

  return {
    createOobMessageWithService,
    createOobMessageWithDid,
    createConnectionFromOobMsg,
    reuseConnectionFromOobMsg
  }
}
