const { CredentialDef } = require('@hyperledger/node-vcx-wrapper')

module.exports.createServiceLedgerCredDef = function createServiceLedgerCredDef ({ logger, saveCredDef, loadCredDef, listCredDefIds }) {
  async function createCredentialDefinitionV2 (issuerDid, schemaId, credDefId, supportRevocation, tag = 'tag1') {
    const data = {
      issuerDid,
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
      const serCredDef = credDef.serialize()
      logger.info(`Credential definition ${id}: ${JSON.stringify(serCredDef)}`)
    }
  }

  async function getCredDefId (credDefId) {
    const credDef = await loadCredDef(credDefId)
    logger.info(`Getting credDefId for credential definition ${credDefId}`)
    return credDef.getCredDefId()
  }

  return {
    createCredentialDefinitionV2,
    listIds,
    printInfo,
    getCredDefId
  }
}
