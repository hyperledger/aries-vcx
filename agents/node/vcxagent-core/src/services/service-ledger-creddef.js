const { CredentialDef } = require('@hyperledger/node-vcx-wrapper')

module.exports.createServiceLedgerCredDef = function createServiceLedgerCredDef ({ logger, saveCredDef, loadCredDef, listCredDefIds }) {
  async function createCredentialDefinition (schemaId, credDefId, revocationDetails, tailsUrl) {
    const data = {
      revocationDetails,
      schemaId,
      sourceId: credDefId
    }
    logger.info(`Create a new credential definition on the ledger from input: ${JSON.stringify(data)}`)

    const credDef = await CredentialDef.createAndStore(data)
    await credDef.publish(tailsUrl)
    await saveCredDef(credDefId, credDef)
    logger.info(`Created credentialDefinition ${credDefId}.`)
    return credDef
  }

  async function createCredentialDefinitionV2 (schemaId, credDefId, supportRevocation, tag = 'tag1') {
    const data = {
      supportRevocation,
      schemaId,
      sourceId: credDefId,
      tag
    }
    logger.info(`Creating a new credential definition on the ledger from input: ${JSON.stringify(data)}`)
    const credDef = await CredentialDef.create(data)
    await credDef.publish()
    await saveCredDef(credDefId, credDef)
    logger.info(`Created credentialDefinition ${credDefId}.`)
    return credDef
  }

  async function listIds () {
    return listCredDefIds()
  }

  async function printInfo (credDefIds) {
    for (const id of credDefIds) {
      const credDef = await loadCredDef(id)
      const serCredDef = await credDef.serialize()
      logger.info(`Credential definition ${id}: ${JSON.stringify(serCredDef)}`)
    }
  }

  async function getTailsFile (credDefId) {
    const credDef = await loadCredDef(credDefId)
    logger.info(`Getting tails file for credential definition ${credDef}`)
    return credDef.getTailsFile()
  }

  async function getTailsHash (credDefId) {
    const credDef = await loadCredDef(credDefId)
    logger.info(`Getting tails hash for credential definition ${credDefId}`)
    return credDef.getTailsHash()
  }

  async function getCredDefId (credDefId) {
    const credDef = await loadCredDef(credDefId)
    logger.info(`Getting credDefId for credential definition ${credDefId}`)
    return credDef.getCredDefId()
  }

  return {
    createCredentialDefinition,
    createCredentialDefinitionV2,
    listIds,
    printInfo,
    getTailsFile,
    getTailsHash,
    getCredDefId
  }
}
