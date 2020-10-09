const { createFileStorage } = require('./storage-file')
const mkdirp = require('mkdirp')

async function createStorageService (agentName) {
  mkdirp.sync('storage-agentProvisions/')
  mkdirp.sync('storage-connections/')
  mkdirp.sync('storage-credentialDefinitions/')
  mkdirp.sync('storage-schemas/')

  const storageAgentProvisions = await createFileStorage(`storage-agentProvisions/${agentName}`)
  const storageConnections = await createFileStorage(`storage-connections/${agentName}`)
  const storageCredIssuer = await createFileStorage(`storage-credsIssuer/${agentName}`)
  const storageCredHolder = await createFileStorage(`storage-credsHolder/${agentName}`)
  const storageProof = await createFileStorage(`storage-proofs/${agentName}`)
  const storageDisclosedProof = await createFileStorage(`storage-dislosedProofs/${agentName}`)
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

  async function saveConnection (name, connection) {
    await storageConnections.set(`${name}`, connection)
  }

  async function loadConnection (name) {
    const res = await storageConnections.get(`${name}`)
    if (!res) {
      throw Error(`Connection ${name} was not found.`)
    }
    return res
  }

  async function saveSchema (name, connection) {
    await storageSchemas.set(name, connection)
  }

  async function loadSchema (name) {
    const res = await storageSchemas.get(name)
    if (!res) {
      throw Error(`Schema ${name} was not found.`)
    }
    return res
  }

  async function saveCredentialDefinition (name, credDef) {
    await storageCredentialDefinitons.set(name, credDef)
  }

  async function loadCredentialDefinition (name) {
    const res = await storageCredentialDefinitons.get(name)
    if (!res) {
      throw Error(`Connection ${name} was not found.`)
    }
    return res
  }

  async function saveCredIssuer (name, credIssuer) {
    await storageCredIssuer.set(name, credIssuer)
  }

  async function loadCredIssuer (name) {
    const res = await storageCredIssuer.get(name)
    if (!res) {
      throw Error(`Connection ${name} was not found.`)
    }
    return res
  }

  async function saveCredHolder (name, credHolder) {
    await storageCredHolder.set(name, credHolder)
  }

  async function loadCredHolder (name) {
    const res = await storageCredHolder.get(name)
    if (!res) {
      throw Error(`Connection ${name} was not found.`)
    }
    return res
  }

  async function saveDisclosedProof (name, proof) {
    await storageDisclosedProof.set(name, proof)
  }

  async function loadDisclosedProof (name) {
    const res = await storageDisclosedProof.get(name)
    if (!res) {
      throw Error(`Connection ${name} was not found.`)
    }
    return res
  }

  async function saveProof (name, proof) {
    await storageProof.set(name, proof)
  }

  async function loadProof (name) {
    const res = await storageProof.get(name)
    if (!res) {
      throw Error(`Connection ${name} was not found.`)
    }
    return res
  }

  async function listConnectionKeys () {
    return storageConnections.keys()
  }

  async function listSchemaKeys () {
    return storageSchemas.keys()
  }

  async function listCredentialDefinitionKeys () {
    return storageCredentialDefinitons.keys()
  }

  async function listCredIssuerKeys () {
    return storageCredIssuer.keys()
  }

  async function listCredHolderKeys () {
    return storageCredHolder.keys()
  }

  async function listDisclosedProofKeys () {
    return storageDisclosedProof.keys()
  }

  async function listProofKeys () {
    return storageProof.keys()
  }

  return {
    agentProvisionExists,
    saveAgentProvision,
    loadAgentProvision,

    saveConnection,
    loadConnection,

    saveSchema,
    loadSchema,

    saveCredentialDefinition,
    loadCredentialDefinition,

    saveCredIssuer,
    loadCredIssuer,

    saveCredHolder,
    loadCredHolder,

    saveDisclosedProof,
    loadDisclosedProof,

    saveProof,
    loadProof,

    listConnectionKeys,
    listSchemaKeys,
    listCredentialDefinitionKeys,
    listCredIssuerKeys,
    listCredHolderKeys,
    listDisclosedProofKeys,
    listProofKeys
  }
}

module.exports.createStorageService = createStorageService
