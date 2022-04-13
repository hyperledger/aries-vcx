const { OutOfBandSender, OutOfBandReceiver } = require('@hyperledger/node-vcx-wrapper')

module.exports.createServiceOutOfBand = function createServiceOutOfBand ({ logger, saveConnection, loadConnection }) {
  async function _createOobSender (message, label) {
    logger.info('createOobMsg >>>')
    const oob = await OutOfBandSender.create({ label })
    if (message) {
      const msgParsed = JSON.parse(message)
      const msgType = msgParsed['@type']
      if (!msgType) {
        throw Error(`Message appended to OOB message must have @type. Invalid message: ${msgParsed}`)
      }
      logger.info(`createOobMsg >>> appending message of type ${msgType}`)
      await oob.appendMessage(message)
      logger.info('createOobMsg >>> append message')
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
    const connection = await loadConnection(connectionId)
    await connection.sendHandshakeReuse(oobMsg)
  }

  async function connectionExists (connectionIds, oobMsg) {
    const connections = await connectionIds.reduce(async (filtered, cid) => {
      let connection
      try {
        connection = await loadConnection(cid)
        filtered.push(connection)
        return filtered
      } catch {}
    }, [])
    const oobReceiver = await OutOfBandReceiver.createWithMessage(oobMsg)
    if (connections && await oobReceiver.connectionExists(connections)) {
      return true
    }
    return false
  }

  return {
    createOobMessageWithService,
    createOobMessageWithDid,
    createConnectionFromOobMsg,
    connectionExists,
    reuseConnectionFromOobMsg
  }
}
