const {
  Agent
} = require('@hyperledger/node-vcx-wrapper')
const sleepPromise = require('sleep-promise')

module.exports.createServiceAgents = function createServiceAgents ({ logger, saveAgent, loadAgent }) {
  async function publicAgentCreate (agentId, institutionDid) {
    logger.info(`Creating public agent with id ${agentId} for institution did ${institutionDid}`)
    const agent = await Agent.create(institutionDid)
    const invite = await agent.generatePublicInvite('abc')
    logger.info(invite)
    await saveAgent(agentId, agent)
    return agent
  }

  async function getPublicInvite (agentId, label) {
    logger.info(`Public agent with id ${agentId} is creating public invite with label ${label}`)
    let agent
    try {
      agent = await loadAgent(agentId)
    } catch (err) {
      await sleepPromise(1000)
      throw err
    }
    logger.info('Serializing deserialized agent')
    logger.info(`Serialized agent: ${await agent.serialize()}`)
    return agent.generatePublicInvite(label)
  }

  return {
    publicAgentCreate,
    getPublicInvite
  }
}
