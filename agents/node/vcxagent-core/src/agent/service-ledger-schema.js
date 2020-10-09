const { Schema } = require('@absaoss/node-vcx-wrapper')

module.exports.createServiceLedgerSchema = function createServiceLedgerSchema (logger, storeSchema, loadSchema, listSchemaIds) {
  async function createSchema (schemaData) {
    logger.info(`Creating a new schema on the ledger: ${JSON.stringify(schemaData, null, 2)}`)

    const schema = await Schema.create(schemaData)
    const schemaId = await schema.getSchemaId()
    const serSchema = await schema.serialize()
    await storeSchema(schemaId, serSchema)
    logger.info(`Created schema with id ${schemaId}`)
    return schemaId
  }

  async function listIds () {
    return listSchemaIds()
  }

  async function printInfo (schemaIds) {
    for (const id of schemaIds) {
      const serSchema = await loadSchema(id)
      logger.info(`Schema ${id}: ${JSON.stringify(serSchema)}`)
    }
  }

  return {
    createSchema,

    listIds,
    printInfo
  }
}
