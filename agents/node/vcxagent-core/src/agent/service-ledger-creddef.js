const { CredentialDef } = require('@absaoss/node-vcx-wrapper')

module.exports.createServiceLedgerCredDef = function createServiceLedgerCredDef (logger, storeSchema, loadSchema, storeCredDef, loadCredDef, listCredDefIds) {
  async function createCredentialDefinition (schemaId, credDefId) {
    logger.info('Create a new credential definition on the ledger')
    const data = {
      name: credDefId,
      paymentHandle: 0,
      revocationDetails: {
        supportRevocation: true,
        tailsFile: '/tmp/tails',
        maxCreds: 5
      },
      schemaId: schemaId,
      sourceId: 'testCredentialDefSourceId123'
    }
    const credDef = await CredentialDef.create(data)
    await storeCredDef(credDefId, credDef)
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
