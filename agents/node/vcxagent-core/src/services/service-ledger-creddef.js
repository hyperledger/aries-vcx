const { CredentialDef } = require('@hyperledger/node-vcx-wrapper')

module.exports.createServiceLedgerCredDef = function createServiceLedgerCredDef ({ logger, saveCredDef, loadCredDef, listCredDefIds }) {
  async function createCredentialDefinition (schemaId, credDefId, revocationDetails, tailsUrl) {
    const data = {
      name: credDefId,
      revocationDetails,
      schemaId,
      sourceId: credDefId,
      tailsUrl
    }
    logger.info(`Create a new credential definition on the ledger from input: ${JSON.stringify(data)}`)

    const credDef = await CredentialDef.create(data)
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

  return {
    createCredentialDefinition,
    listIds,
    printInfo,
    getTailsFile,
    getTailsHash
  }
}
