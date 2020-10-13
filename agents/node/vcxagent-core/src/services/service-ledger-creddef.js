const { CredentialDef } = require('@absaoss/node-vcx-wrapper')

module.exports.createServiceLedgerCredDef = function createServiceLedgerCredDef ({ logger, saveCredDef, loadCredDef, listCredDefIds }) {
  async function createCredentialDefinition (schemaId, credDefId, revocationDetails) {
    const data = {
      name: credDefId,
      paymentHandle: 0,
      revocationDetails,
      schemaId,
      sourceId: credDefId
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

  return {
    createCredentialDefinition,

    listIds,
    printInfo
  }
}
