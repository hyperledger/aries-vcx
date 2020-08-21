const { createFileStorage } = require('./storage-file')
const mkdirp = require('mkdirp')

async function createStorageService (agentName) {

  mkdirp.sync('storage-agentProvisions/')
  mkdirp.sync('storage-connections/')
  mkdirp.sync('storage-credentialDefinitions/')
  mkdirp.sync('storage-schemas/')
  const storageAgentProvisions = await createFileStorage(`storage-agentProvisions/${agentName}`)
  const storageConnections = await createFileStorage(`storage-connections/${agentName}`)
  const storageCredentialDefinitons = await createFileStorage(`storage-credentialDefinitions/${agentName}`)
  const storageSchemas = await createFileStorage(`storage-schemas/${agentName}`)

  async function agentProvisionExists () {
    return storageAgentProvisions.hasKey('agent-provision')
  }

  async function saveAgentProvision (provision) {
    await storageAgentProvisions.set('agent-provision', provision)
  }

  async function loadAgentProvision () {
    return storageAgentProvisions.get('agent-provision')
  }

  async function saveConnection (connectionName, connection) {
    await storageConnections.set(`${connectionName}`, connection)
  }

  async function loadConnection (connectionName) {
    return storageConnections.get(`${connectionName}`)
  }

  async function listConnectionNames () {
    return storageConnections.keys()
  }

  async function saveSchema (schemaName, connection) {
    await storageSchemas.set(schemaName, connection)
  }

  async function loadSchema (schemaName) {
    return storageSchemas.get(schemaName)
  }

  async function saveCredentialDefinition (credentialDefinitionName, credDef) {
    await storageCredentialDefinitons.set(credentialDefinitionName, credDef)
  }

  async function loadCredentialDefinition (credentialDefinitionName) {
    return storageCredentialDefinitons.get(credentialDefinitionName)
  }

  return {
    agentProvisionExists,
    saveAgentProvision,
    loadAgentProvision,
    saveConnection,
    loadConnection,
    listConnectionNames,
    saveSchema,
    loadSchema,
    saveCredentialDefinition,
    loadCredentialDefinition
  }
}

module.exports.createStorageService = createStorageService
