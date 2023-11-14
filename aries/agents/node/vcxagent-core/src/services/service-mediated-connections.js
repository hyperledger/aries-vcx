const { NonmediatedConnection } = require('@hyperledger/node-vcx-wrapper')

module.exports.createServiceNonmediatedConnections = function createServiceNonmediatedConnections ({ logger, saveNonmediatedConnection, loadNonmediatedConnection, endpointInfo }) {
  async function inviterConnectionCreatePwInvite (connectionId) {
    logger.info(`inviterConnectionCreatePwInvite >> connectionId=${connectionId}`)
    const connection = await NonmediatedConnection.createInviter()
    logger.debug(`InviterConnectionSM after created connection:\n${JSON.stringify(connection.serialize())}`)
    await connection.createInvite(endpointInfo)
    logger.debug(`InviterConnectionSM after invitation was generated:\n${JSON.stringify(connection.serialize())}`)
    await saveNonmediatedConnection(connectionId, connection)
    const invite = connection.getInvitation()
    logger.debug(`InviterConnectionSM created invitation ${invite}`)
    return invite
  }

  async function inviterConnectionCreateFromRequest (connectionId, request, pwInfo) {
    logger.info(`inviterConnectionCreateFromRequest >> connectionId=${connectionId}, request: ${request}, pwInfo: ${pwInfo}`)
    const connection = await NonmediatedConnection.createInviter(pwInfo)
    logger.debug(`InviterConnectionSM after created connection:\n${JSON.stringify(connection.serialize())}`)
    await connection.processRequest(request, endpointInfo)
    logger.debug(`InviterConnectionSM after processing request:\n${JSON.stringify(connection.serialize())}`)
    await connection.sendResponse()
    logger.debug(`InviterConnectionSM after sending response:\n${JSON.stringify(connection.serialize())}`)
    await saveNonmediatedConnection(connectionId, connection)
  }

  async function inviterConnectionProcessRequest (connectionId, request) {
    logger.info(`inviterConnectionProcessRequest >> connectionId=${connectionId}, request: ${request}`)
    const connection = await loadNonmediatedConnection(connectionId)
    await connection.processRequest(request, endpointInfo)
    logger.info(`InviterConnectionSM after processing request:\n${JSON.stringify(connection.serialize())}`)
    await connection.sendResponse()
    logger.info(`InviterConnectionSM after sending response:\n${JSON.stringify(connection.serialize())}`)
    await saveNonmediatedConnection(connectionId, connection)
  }

  async function inviterConnectionProcessAck (connectionId, ack) {
    logger.info(`inviterConnectionProcessAck >> connectionId=${connectionId}, ack: ${ack}`)
    const connection = await loadNonmediatedConnection(connectionId)
    await connection.processAck(ack)
    logger.debug(`InviterConnectionSM after processing ack: ${JSON.stringify(connection.serialize())}`)
    await saveNonmediatedConnection(connectionId, connection)
  }

  async function inviteeConnectionCreateFromInvite (connectionId, invite) {
    logger.info(`inviteeConnectionCreateFromInvite >> connectionId=${connectionId}, invite: ${invite}`)
    const connection = await NonmediatedConnection.createInvitee(invite)
    logger.debug(`InviteeConnectionSM after created from invitation:\n${JSON.stringify(connection.serialize())}`)
    await connection.processInvite(invite)
    await connection.sendRequest(endpointInfo)
    logger.debug(`InviteeConnectionSM after sending request:\n${JSON.stringify(connection.serialize())}`)
    await saveNonmediatedConnection(connectionId, connection)
  }

  async function inviteeConnectionProcessResponse (connectionId, response) {
    logger.info(`inviteeConnectionProcessResponse >> connectionId=${connectionId}, response: ${response}`)
    const connection = await loadNonmediatedConnection(connectionId)
    await connection.processResponse(response)
    logger.debug(`InviteeConnectionSM after processing response:\n${JSON.stringify(connection.serialize())}`)
    await connection.sendAck()
    logger.debug(`InviteeConnectionSM connection after sending ack:\n${JSON.stringify(connection.serialize())}`)
    await saveNonmediatedConnection(connectionId, connection)
  }

  async function sendMessage (connectionId, content) {
    logger.info(`nonmediatedConnectionSendMessage >> connectionId=${connectionId}, content: ${content}`)
    const connection = await loadNonmediatedConnection(connectionId)
    await connection.sendMessage(content)
  }

  async function getState (connectionId) {
    const connection = await loadNonmediatedConnection(connectionId)
    return connection.getState()
  }

  return {
    inviterConnectionCreatePwInvite,
    inviterConnectionCreateFromRequest,
    inviterConnectionProcessRequest,
    inviterConnectionProcessAck,
    inviteeConnectionCreateFromInvite,
    inviteeConnectionProcessResponse,
    sendMessage,
    getState
  }
}
