const { OutOfBandSender } = require('@hyperledger/node-vcx-wrapper')

module.exports.createServiceOutOfBand = function createServiceOutOfBand ({ loadConnection }) {
  async function createOobMessageWithDid (message, label, publicDid) {
    const oobSender = OutOfBandSender.create({ label })
    if (message) {
      const msgParsed = JSON.parse(message)
      const msgType = msgParsed['@type']
      if (!msgType) {
        throw Error(`Message appended to OOB message must have @type. Invalid message: ${msgParsed}`)
      }
      oobSender.appendMessage(message)
    }
    oobSender.appendServiceDid(publicDid)
    return oobSender.toMessage()
  }

  async function reuseConnectionFromOobMsg (connectionId, oobMsg) {
    const connection = await loadConnection(connectionId)
    await connection.sendHandshakeReuse(oobMsg)
  }

  return {
    createOobMessageWithDid,
    reuseConnectionFromOobMsg
  }
}
