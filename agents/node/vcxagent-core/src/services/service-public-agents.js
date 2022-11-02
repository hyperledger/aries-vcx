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

  async function getPwVk (agentId) {
    logger.info(`Public agent with id ${agentId} is getting pw vk`)
    const agent = await loadAgent(agentId)
    return JSON.parse(await agent.serialize()).pairwise_info.my_pw_vk
  }

  return {
    publicAgentCreate,
    downloadConnectionRequests,
    getPwVk,
  }
}
