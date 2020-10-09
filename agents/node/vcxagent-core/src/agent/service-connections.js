const { getMessagesForPwDid } = require('../utils/messages')
const {
  Connection,
  StateType
} = require('@absaoss/node-vcx-wrapper')
const { pollFunction } = require('../common')

module.exports.createServiceConnections = function createServiceConnections (logger, storeConnection, loadConnection, listConnectionIds) {
  async function _createConnection (connectionId) {
    logger.info(`InviterConnectionSM creating connection ${connectionId}`)
    const connection = await Connection.create({ id: connectionId })
    logger.debug(`InviterConnectionSM after created connection:\n${JSON.stringify(await connection.serialize())}`)
    await connection.connect('{}')
    await connection.updateState()
    return connection
  }

  async function inviterConnectionCreate (connectionId, cbInvitation) {
    const connection = await _createConnection(connectionId)
    logger.debug(`InviterConnectionSM after invitation was generated:\n${JSON.stringify(await connection.serialize())}`)
    const invite = await connection.inviteDetails()
    if (cbInvitation) {
      cbInvitation(invite)
    }
    const connSerialized = await connection.serialize()
    await storeConnection(connectionId, connSerialized)
    logger.info(`InviterConnectionSM has established connection ${connectionId}`)
    return { invite, connection }
  }

  async function inviterConnectionCreateAndAccept (conenctionId, cbInvitation) {
    const { invite, connection } = await inviterConnectionCreate(conenctionId, cbInvitation)
    await _progressConnectionToAcceptedState(connection, 20, 2000)

    const connSerialized = await connection.serialize()
    await storeConnection(conenctionId, connSerialized)
    logger.debug(`InviterConnectionSM after connection was accepted:\n${JSON.stringify(connSerialized)}`)
    return { invite, connection }
  }

  // Invitee creates new connection from invite, sends connection request
  async function inviteeConnectionAcceptFromInvitation (connectionId, invite) {
    logger.info(`InviteeConnectionSM creating connection ${connectionId} from connection invitation.`)
    const connection = await Connection.createWithInvite({ id: connectionId, invite })
    logger.debug(`InviteeConnectionSM after created from invitation:\n${JSON.stringify(await connection.serialize())}`)
    await connection.connect({ data: '{}' })
    logger.debug('InviteeConnectionSM created connection agent')
    await connection.updateState()

    const connSerialized = await connection.serialize()
    await storeConnection(connectionId, connSerialized)
    logger.debug(`InviteeConnectionSM after sending connection request:\n${JSON.stringify(connSerialized)}`)
    return connection
  }

  async function inviteeConnectionAcceptFromInvitationAndProgress (connectionId, invite) {
    const connection = await inviteeConnectionAcceptFromInvitation(connectionId, invite)
    await _progressConnectionToAcceptedState(connection, 20, 2000)
    logger.info(`InviteeConnectionSM has established connection ${connectionId}`)
    const connSerialized = await connection.serialize()
    await storeConnection(connectionId, connSerialized)
    return connection
  }

  async function _progressConnectionToAcceptedState (connection, attemptsThreshold, timeoutMs) {
    async function progressToAcceptedState () {
      if (await connection.updateState() !== StateType.Accepted) {
        return { result: undefined, isFinished: false }
      } else {
        return { result: null, isFinished: true }
      }
    }

    const [error] = await pollFunction(progressToAcceptedState, 'Progress connection', logger, attemptsThreshold, timeoutMs)
    if (error) {
      throw Error(`Couldn't progress connection to Accepted state. ${error}`)
    }
  }

  async function connectionUpdate (connectionId) {
    const connSerializedBefore = await loadConnection(connectionId)
    const connection = await Connection.deserialize(connSerializedBefore)

    const state = await connection.updateState()

    const connSerialized = await connection.serialize()
    await storeConnection(connectionId, connSerialized)

    return state
  }

  async function connectionAutoupdate (connectionId, updateAttemptsThreshold = 10, timeoutMs = 2000) {
    const connSerializedBefore = await loadConnection(connectionId)
    const connection = await Connection.deserialize(connSerializedBefore)
    await _progressConnectionToAcceptedState(connection, updateAttemptsThreshold, timeoutMs)

    logger.info('Success! Connection was progressed to Accepted state.')
    const connSerialized = await connection.serialize()
    await storeConnection(connectionId, connSerialized)
    return connection
  }

  async function signData (connectionId, dataBase64) {
    const connSerializedBefore = await loadConnection(connectionId)
    const connection = await Connection.deserialize(connSerializedBefore)
    var challengeBuffer = Buffer.from(dataBase64, 'base64')
    let signatureBuffer
    try {
      signatureBuffer = await connection.signData(challengeBuffer)
    } catch (err) {
      throw Error(`Error occurred while connection was signing data '${dataBase64}'. Err Message = ${err.message} Stack = ${err.stack}`)
    }
    if (!signatureBuffer) {
      throw Error(`Error occurred while connection was signing data '${dataBase64}' The resulting signature was empty.`)
    }
    return signatureBuffer.toString('base64')
  }

  async function verifySignature (connectionId, dataBase64, signatureBase64) {
    const serConnection = await loadConnection(connectionId)
    const connection = await Connection.deserialize(serConnection)
    const data = Buffer.from(dataBase64, 'base64')
    const signature = Buffer.from(signatureBase64, 'base64')
    return connection.verifySignature({ data, signature })
  }

  async function getConnectionPwDid (connectionId) {
    const serConnection = await loadConnection(connectionId)
    return serConnection.data.pw_did
  }

  async function sendMessage (connectionId, payload) {
    const serConnection = await loadConnection(connectionId)
    const connection = await Connection.deserialize(serConnection)
    await connection.sendMessage({ msg: payload, msg_title: 'msg_title', msg_type: 'msg_type', ref_msg_id: 'ref_msg_id' })
  }

  async function getMessages (connectionId, filterStatuses = [], filterUids = []) {
    const serConnection = await loadConnection(connectionId)
    const pwDid = serConnection.data.pw_did
    return getMessagesForPwDid(pwDid, [], filterStatuses, filterUids)
  }

  async function getState (connectionId) {
    const connSerialized = await loadConnection(connectionId)
    const connection = await Connection.deserialize(connSerialized)
    return await connection.getState()
  }

  async function listIds () {
    return listConnectionIds()
  }

  async function printInfo (connectionIds) {
    for (const id of connectionIds) {
      const state = await getState(id)
      logger.info(`Connection ${id} state=${state}`)
    }
  }

  return {
    // inviter
    inviterConnectionCreate,
    inviterConnectionCreateAndAccept,

    // invitee
    inviteeConnectionAcceptFromInvitation,
    inviteeConnectionAcceptFromInvitationAndProgress,

    // universal
    connectionAutoupdate,
    connectionUpdate,
    getConnectionPwDid,

    signData,
    verifySignature,
    sendMessage,
    getMessages,

    getState,
    listIds,
    printInfo
  }
}
