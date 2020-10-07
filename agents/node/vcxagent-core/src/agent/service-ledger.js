const {
  Schema,
  CredentialDef,
  getLedgerAuthorAgreement,
  setActiveTxnAuthorAgreementMeta
} = require('@absaoss/node-vcx-wrapper')

module.exports.createServiceLedger = function createServiceLedger (logger, storeSchema, loadSchema, storeCredDef, loadCredDef) {
  async function createSchema (schemaData) {
    logger.info(`Creating a new schema on the ledger: ${JSON.stringify(schemaData, null, 2)}`)

    const schema = await Schema.create(schemaData)
    const schemaId = await schema.getSchemaId()
    const serSchema = await schema.serialize()
    await storeSchema(schemaId, serSchema)
    logger.info(`Created schema with id ${schemaId}`)
    return schemaId
  }

  async function createCredentialDefinition (schemaId, credDefName) {
    logger.info('Create a new credential definition on the ledger')
    const data = {
      name: credDefName,
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
    const credDefSer = await credDef.serialize()
    await storeCredDef(credDefName, credDefSer)
    logger.info(`Created credentialDefinition ${credDefName}.`)
    return credDef
  }

  async function acceptTaa () {
    const taa = await getLedgerAuthorAgreement()
    const taaJson = JSON.parse(taa)
    const utime = Math.floor(new Date() / 1000)
    await setActiveTxnAuthorAgreementMeta(taaJson.text, taaJson.version, null, Object.keys(taaJson.aml)[0], utime)
  }

  return {
    acceptTaa,
    createSchema,
    createCredentialDefinition
  }
}
