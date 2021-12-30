const {
  Agent
} = require('@hyperledger/node-vcx-wrapper')

module.exports.createServiceAgents = function createServiceAgents ({ logger, saveAgent, loadAgent }) {
  async function publicAgentCreate (agentId, institutionDid) {
    logger.info(`Creating public agent with id ${agentId} for institution did ${institutionDid}`)
    const agent = await Agent.create(agentId, institutionDid)
    await saveAgent(agentId, agent)
    logger.info(`Created public agent with id ${agentId} for institution did ${institutionDid}`)
    return agent
  }

  async function getPublicInvite (agentId, label) {
    logger.info(`Public agent with id ${agentId} is creating public invite with label ${label}`)
    const agent = await loadAgent(agentId)
    return agent.generatePublicInvite(label)
  }

  async function downloadConnectionRequests (agentId) {
    logger.info(`Public agent with id ${agentId} is downloading connection requests`)
    const agent = await loadAgent(agentId)
    return agent.downloadConnectionRequests()
  }

  return {
    publicAgentCreate,
    getPublicInvite,
    downloadConnectionRequests
  }
}
