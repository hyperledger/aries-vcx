const { getMessagesForPwDid } = require('../utils/messages')
const {
  Connection,
  StateType
} = require('@absaoss/node-vcx-wrapper')
const { pollFunction } = require('../common')

module.exports.createServiceConnections = function createServiceConnections (logger, storeConnection, loadConnection) {
  async function _createConnection (connectionName) {
    logger.info(`InviterConnectionSM creating connection ${connectionName}`)
    const connection = await Connection.create({ id: connectionName })
    logger.debug(`InviterConnectionSM after created connection:\n${JSON.stringify(await connection.serialize())}`)
    await connection.connect('{}')
    await connection.updateState()
    return connection
  }

  async function inviterConnectionCreate (connectionName, cbInvitation) {
    const connection = await _createConnection(connectionName)
    logger.debug(`InviterConnectionSM after invitation was generated:\n${JSON.stringify(await connection.serialize())}`)
    const invite = await connection.inviteDetails()
    if (cbInvitation) {
      cbInvitation(invite)
    }
    const connSerialized = await connection.serialize()
    await storeConnection(connectionName, connSerialized)
    logger.info(`InviterConnectionSM has established connection ${connectionName}`)
    return { invite, connection }
  }

  async function inviterConnectionCreateAndAccept (connectionName, cbInvitation) {
    const { invite, connection } = await inviterConnectionCreate(connectionName, cbInvitation)
    await _progressConnectionToAcceptedState(connection, 20, 2000)

    const connSerialized = await connection.serialize()
    await storeConnection(connectionName, connSerialized)
    logger.debug(`InviterConnectionSM after connection was accepted:\n${JSON.stringify(connSerialized)}`)
    return { invite, connection }
  }

  // Invitee creates new connection from invite, sends connection request
  async function inviteeConnectionAcceptFromInvitation (connectionName, invite) {
    logger.info(`InviteeConnectionSM creating connection ${connectionName} from connection invitation.`)
    const connection = await Connection.createWithInvite({ id: connectionName, invite })
    logger.debug(`InviteeConnectionSM after created from invitation:\n${JSON.stringify(await connection.serialize())}`)
    await connection.connect({ data: '{}' })
    logger.debug('InviteeConnectionSM created connection agent')
    await connection.updateState()

    const connSerialized = await connection.serialize()
    await storeConnection(connectionName, connSerialized)
    logger.debug(`InviteeConnectionSM after sending connection request:\n${JSON.stringify(connSerialized)}`)
    return connection
  }

  async function inviteeConnectionAcceptFromInvitationAndProgress (connectionName, invite) {
    const connection = await inviteeConnectionAcceptFromInvitation(connectionName, invite)
    await _progressConnectionToAcceptedState(connection, 20, 2000)
    logger.info(`InviteeConnectionSM has established connection ${connectionName}`)
    const connSerialized = await connection.serialize()
    await storeConnection(connectionName, connSerialized)
    return connection
  }

  async function _progressConnectionToAcceptedState (connection, attemptsThreshold, timeout) {
    async function progressToAcceptedState () {
      if (await connection.updateState() !== StateType.Accepted) {
        return { result: undefined, isFinished: false }
      } else {
        return { result: null, isFinished: true }
      }
    }

    const [error] = await pollFunction(progressToAcceptedState, 'Progress connection', logger, attemptsThreshold, timeout)
    if (error) {
      throw Error(`Couldn't progress connection to Accepted state. ${error}`)
    }
  }

  async function connectionUpdate (connectionName) {
    const connSerializedBefore = await loadConnection(connectionName)
    const connection = await Connection.deserialize(connSerializedBefore)

    const state = await connection.updateState()

    const connSerialized = await connection.serialize()
    await storeConnection(connectionName, connSerialized)

    return state
  }

  async function connectionAutoupdate (connectionName, updateAttemptsThreshold = 10, timeout = 2000) {
    const connSerializedBefore = await loadConnection(connectionName)
    const connection = await Connection.deserialize(connSerializedBefore)
    await _progressConnectionToAcceptedState(connection, updateAttemptsThreshold, timeout)

    logger.info('Success! Connection was progressed to Accepted state.')
    const connSerialized = await connection.serialize()
    await storeConnection(connectionName, connSerialized)
    return connection
  }

  async function signData (connectionName, dataBase64) {
    const connSerializedBefore = await loadConnection(connectionName)
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

  async function verifySignature (connectionName, dataBase64, signatureBase64) {
    const serConnection = await loadConnection(connectionName)
    const connection = await Connection.deserialize(serConnection)
    const data = Buffer.from(dataBase64, 'base64')
    const signature = Buffer.from(signatureBase64, 'base64')
    const success = await connection.verifySignature({ data, signature })
    return success === 'Success'
  }

  async function getConnectionPwDid (connectionName) {
    const serConnection = await loadConnection(connectionName)
    return serConnection.agent_info.pw_did
  }

  async function sendMessage (connectionName, payload) {
    const serConnection = await loadConnection(connectionName)
    const connection = await Connection.deserialize(serConnection)
    await connection.sendMessage({ msg: payload, msg_title: 'msg_title', msg_type: 'msg_type', ref_msg_id: 'ref_msg_id' })
  }

  async function getMessages (connectionName, filterStatuses = [], filterUids = []) {
    const serConnection = await loadConnection(connectionName)
    const pwDid = serConnection.data.pw_did
    return getMessagesForPwDid(pwDid, [], filterStatuses, filterUids)
  }

  async function connectionGetState (connectionName) {
    const connSerialized = await loadConnection(connectionName)
    const connection = await Connection.deserialize(connSerialized)
    return await connection.getState()
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

    signData,
    verifySignature,
    connectionGetState,
    getConnectionPwDid,
    sendMessage,
    getMessages
  }
}
