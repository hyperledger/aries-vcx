const { OutOfBandSender, OutOfBandReceiver, Connection } = require('@hyperledger/node-vcx-wrapper')

module.exports.createServiceOutOfBand = function createServiceOutOfBand ({ logger, saveConnection, loadConnection }) {
  function _createOobSender (message, label) {
    const oob = OutOfBandSender.create({ label })
    if (message) {
      const msgParsed = JSON.parse(message)
      const msgType = msgParsed['@type']
      if (!msgType) {
        throw Error(`Message appended to OOB message must have @type. Invalid message: ${msgParsed}`)
      }
      oob.appendMessage(message)
    }
    return oob
  }

  async function createOobMessageWithDid (message, label, publicDid) {
    const oobSender = _createOobSender(message, label)
    oobSender.appendServiceDid(publicDid)
    return oobSender.toMessage()
  }

  async function createConnectionFromOobMsg (connectionId, oobMsg) {
    const connection = await Connection.createWithInvite({ id: 'foo', invite: oobMsg })
    await connection.connect()
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
    const oobReceiver = OutOfBandReceiver.createWithMessage(oobMsg)
    if (connections && await oobReceiver.connectionExists(connections)) {
      return true
    }
    return false
  }

  return {
    createOobMessageWithDid,
    createConnectionFromOobMsg,
    connectionExists,
    reuseConnectionFromOobMsg
  }
}
