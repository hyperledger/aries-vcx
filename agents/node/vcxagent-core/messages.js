const _ = require('lodash')
const { downloadMessages } = require('@absaoss/node-vcx-wrapper')

module.exports.messagesGetForPwDid = async function messagesGetForPwDid (
  pwDid,
  filterTypes = [],
  filterStatuses = ['MS-102', 'MS-103', 'MS-104', 'MS-105', 'MS-106'],
  filterUids = []
) {
  filterStatuses = filterStatuses || ['MS-102', 'MS-103', 'MS-104', 'MS-105', 'MS-106'] // explicit null or undefined interpreted as "no filter"
  const messages = []
  const donwloadInstructions = {
    pairwiseDids: pwDid
  }
  if (filterStatuses && filterStatuses.length > 0) {
    donwloadInstructions.status = filterStatuses.join(',')
  }
  if (filterUids && filterUids.length > 0) {
    donwloadInstructions.uids = filterUids.join(',')
  }
  const res = JSON.parse(await downloadMessages(donwloadInstructions))
  if (res && res.length > 1) {
    throw Error('Unexpected to get more than 1 items from download messages for single pairwise did')
  }
  if (res && res.length === 0) {
    throw Error('Expected to get at least 1 item in download message response.')
  }
  if (!res[0].msgs) {
    throw Error('message item was expected to have msgs field')
  }
  if (res[0].msgs.length > 0) {
    await messages.push(res[0].msgs)
  }
  const flattened = _.flatten(messages)
  return (filterTypes && filterTypes.length > 0) ? flattened.filter(msg => filterTypes.includes(msg.type)) : flattened
}
