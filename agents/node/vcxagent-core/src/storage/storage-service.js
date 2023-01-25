const { createFileStorage } = require('./storage-file')
const mkdirp = require('mkdirp')
const {
  Connection,
  NonmediatedConnection,
  Credential,
  IssuerCredential,
  CredentialDef,
  RevocationRegistry,
  Schema,
  DisclosedProof,
  Proof,
} = require('@hyperledger/node-vcx-wrapper')

async function createStorageService (agentName) {
  mkdirp.sync('storage-agentProvisions/')
  mkdirp.sync('storage-connections/')
  mkdirp.sync('storage-nonmediated-connections/')
  mkdirp.sync('storage-credentialDefinitions/')
  mkdirp.sync('storage-revocationRegistries/')
  mkdirp.sync('storage-schemas/')

  const storageAgentProvisions = await createFileStorage(`storage-agentProvisions/${agentName}`)
  const storageConnections = await createFileStorage(`storage-connections/${agentName}`)
  const storageNonmediatedConnections = await createFileStorage(`storage-connections/${agentName}`)
  const storageCredIssuer = await createFileStorage(`storage-credsIssuer/${agentName}`)
  const storageCredHolder = await createFileStorage(`storage-credsHolder/${agentName}`)
  const storageProof = await createFileStorage(`storage-proofs/${agentName}`)
  const storageDisclosedProof = await createFileStorage(`storage-dislosedProofs/${agentName}`)
  const storageCredentialDefinitons = await createFileStorage(`storage-credentialDefinitions/${agentName}`)
  const storageRevocationRegistries = await createFileStorage(`storage-revocationRegistries/${agentName}`)
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

  async function saveNonmediatedConnection (name, connection) {
    const serialized = await connection.serialize()
    await storageNonmediatedConnections.set(`${name}`, serialized)
  }

  async function loadNonmediatedConnection (name) {
    const serialized = await storageNonmediatedConnections.get(`${name}`)
    if (!serialized) {
      throw Error(`Nonmediated connection ${name} was not found.`)
    }
    return NonmediatedConnection.deserialize(serialized)
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
      throw Error(`CredentialDefinition ${name} was not found.`)
    }
    return CredentialDef.deserialize(serialized)
  }

  async function saveRevocationRegistry (name, revReg) {
    const serialized = await revReg.serialize()
    await storageRevocationRegistries.set(name, serialized)
  }

  async function loadRevocationRegistry (name) {
    const serialized = await storageRevocationRegistries.get(name)
    if (!serialized) {
      throw Error(`RevocationRegistry ${name} was not found.`)
    }
    return RevocationRegistry.deserialize(serialized)
  }

  async function saveCredIssuer (name, credIssuer) {
    const serialized = await credIssuer.serialize()
    await storageCredIssuer.set(name, serialized)
  }

  async function loadCredIssuer (name) {
    const serialized = await storageCredIssuer.get(name)
    if (!serialized) {
      throw Error(`CredentialIssuer ${name} was not found.`)
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
      throw Error(`CredentialHolder ${name} was not found.`)
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
      throw Error(`DisclosedProof ${name} was not found.`)
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
      throw Error(`Proof ${name} was not found.`)
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

  async function listRevocationRegistryKeys () {
    return storageRevocationRegistries.keys()
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

    saveNonmediatedConnection,
    loadNonmediatedConnection,

    saveSchema,
    loadSchema,

    saveCredentialDefinition,
    loadCredentialDefinition,

    saveRevocationRegistry,
    loadRevocationRegistry,

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
    listRevocationRegistryKeys,
    listCredIssuerKeys,
    listCredHolderKeys,
    listDisclosedProofKeys,
    listProofKeys
  }
}

module.exports.createStorageService = createStorageService
