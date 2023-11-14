const _ = require('lodash')

async function maybeJoinWithComma (list) {
  let res
  if (list && list.length > 0) {
    res = list.join(',')
  }
  return res
}

async function parseDownloadMessagesResult (msgs) {
  const messages = []
  const res = JSON.parse(msgs)
  if (res && res.length > 1) {
    throw Error(`Expected to receive messages for single connection, but received messages for ${res.length} connection. This is agency bug.`)
  }
  if (res && res.length === 0) {
    throw Error('Expected to receive messages for single connection, but received messages for none.')
  }
  if (!res[0].msgs) {
    throw Error(`Invalid response, field 'msgs' is missing in response. This is agency bug. Received response = ${JSON.stringify(res)}`)
  }
  if (res[0].msgs.length > 0) {
    await messages.push(res[0].msgs)
  }
  return _.flatten(messages)
}

module.exports.getMessagesForConnection = async function getMessagesForConnection (
  connection,
  filterStatuses = ['MS-103', 'MS-106'],
  filterUids = []
) {
  filterStatuses = filterStatuses || ['MS-103', 'MS-106'] // explicit null or undefined interpreted as "no filter"
  const downloadInstructions = {
    status: await maybeJoinWithComma(filterStatuses),
    uids: await maybeJoinWithComma(filterUids)
  }
  return parseDownloadMessagesResult(await connection.downloadMessages(downloadInstructions))
}
