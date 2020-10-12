const { createFileStorage } = require('./storage-file')
const mkdirp = require('mkdirp')
const {
  Connection,
  Credential,
  IssuerCredential,
  CredentialDef,
  Schema,
  DisclosedProof,
  Proof
} = require('@absaoss/node-vcx-wrapper')

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
    const serialized = await connection.serialize()
    await storageConnections.set(`${name}`, serialized)
  }

  async function loadConnection (name) {
    const serialized = await storageConnections.get(`${name}`)
    if (!serialized) {
      throw Error(`Connection ${name} was not found.`)
    }
    return Connection.deserialize(serialized)
  }

  async function saveSchema (name, schema) {
    const serialized = await schema.serialize()
    await storageSchemas.set(name, serialized)
  }

  async function loadSchema (name) {
    const serialized = await storageSchemas.get(name)
    if (!serialized) {
      throw Error(`Schema ${name} was not found.`)
    }
    return Schema.deserialize(serialized)
  }

  async function saveCredentialDefinition (name, credDef) {
    const serialized = await credDef.serialize()
    await storageCredentialDefinitons.set(name, serialized)
  }

  async function loadCredentialDefinition (name) {
    const serialized = await storageCredentialDefinitons.get(name)
    if (!serialized) {
      throw Error(`Connection ${name} was not found.`)
    }
    return CredentialDef.deserialize(serialized)
  }

  async function saveCredIssuer (name, credIssuer) {
    const serialized = await credIssuer.serialize()
    await storageCredIssuer.set(name, serialized)
  }

  async function loadCredIssuer (name) {
    const serialized = await storageCredIssuer.get(name)
    if (!serialized) {
      throw Error(`Connection ${name} was not found.`)
    }
    return IssuerCredential.deserialize(serialized)
  }

  async function saveCredHolder (name, credHolder) {
    const serialized = await credHolder.serialize()
    await storageCredHolder.set(name, serialized)
  }

  async function loadCredHolder (name) {
    const serialized = await storageCredHolder.get(name)
    if (!serialized) {
      throw Error(`Connection ${name} was not found.`)
    }
    return Credential.deserialize(serialized)
  }

  async function saveDisclosedProof (name, disclosedProof) {
    const serialized = await disclosedProof.serialize()
    await storageDisclosedProof.set(name, serialized)
  }

  async function loadDisclosedProof (name) {
    const serialized = await storageDisclosedProof.get(name)
    if (!serialized) {
      throw Error(`Connection ${name} was not found.`)
    }
    return DisclosedProof.deserialize(serialized)
  }

  async function saveProof (name, proof) {
    const serialized = await proof.serialize()
    await storageProof.set(name, serialized)
  }

  async function loadProof (name) {
    const serialized = await storageProof.get(name)
    if (!serialized) {
      throw Error(`Connection ${name} was not found.`)
    }
    return Proof.deserialize(serialized)
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
