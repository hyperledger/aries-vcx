const {
  PublicAgent
} = require('@hyperledger/node-vcx-wrapper')

module.exports.createServicePublicAgents = function createServicePublicAgents ({ logger, saveAgent, loadAgent }) {
  async function publicAgentCreate (agentId, institutionDid) {
    logger.info(`Creating public agent with id ${agentId} for institution did ${institutionDid}`)
    const agent = await PublicAgent.create(agentId, institutionDid)
    await saveAgent(agentId, agent)
    logger.info(`Created public agent with id ${agentId} for institution did ${institutionDid}`)
    return agent
  }

  async function downloadConnectionRequests (agentId) {
    logger.info(`Public agent with id ${agentId} is downloading connection requests`)
    const agent = await loadAgent(agentId)
    return agent.downloadConnectionRequests()
  }

  return {
    publicAgentCreate,
    downloadConnectionRequests
  }
}
