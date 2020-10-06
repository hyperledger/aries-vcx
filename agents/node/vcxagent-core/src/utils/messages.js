const { downloadMessages } = require('@absaoss/node-vcx-wrapper')
const _ = require('lodash')

module.exports.getMessagesForPwDid = async function getMessagesForPwDid (
  pwDid,
  filterStatuses = ['MS-102', 'MS-103', 'MS-104', 'MS-105', 'MS-106'],
  filterUids = []
) {
  filterStatuses = filterStatuses || ['MS-102', 'MS-103', 'MS-104', 'MS-105', 'MS-106'] // explicit null or undefined interpreted as "no filter"
  const messages = []
  const downloadInstructions = {
    pairwiseDids: pwDid
  }
  if (filterStatuses && filterStatuses.length > 0) {
    downloadInstructions.status = filterStatuses.join(',')
  }
  if (filterUids && filterUids.length > 0) {
    downloadInstructions.uids = filterUids.join(',')
  }
  const res = JSON.parse(await downloadMessages(downloadInstructions))
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
