const { RevocationRegistry } = require('@hyperledger/node-vcx-wrapper')

module.exports.createServiceLedgerRevocationRegistry = function createServiceLedgerRevocationRegistry ({ logger, saveRevReg, loadRevReg, listRevRegIds }) {
  async function createRevocationRegistry (issuerDid, credDefId, tag, tailsDir, maxCreds, tailsUrl = 'dummy.org') {
    const data = {
      issuerDid,
      credDefId,
      tag,
      tailsDir,
      maxCreds
    }
    const revReg = await RevocationRegistry.create(data)
    await revReg.publish(tailsUrl)
    const revRegId = await revReg.getRevRegId()
    await saveRevReg(revRegId, revReg)
    return { revReg, revRegId }
  }

  async function rotateRevocationRegistry (revRegId, maxCreds) {
    logger.info(`Rotating revocation registry ${revRegId}, maxCreds ${maxCreds}`)
    const revReg = await loadRevReg(revRegId)
    let newRevReg
    try {
      newRevReg = await revReg.rotate(maxCreds)
    } catch (err) {
      throw Error(`Error rotating revocation registry ${revRegId}: ${err}`)
    }
    const newRevRegId = await newRevReg.getRevRegId()
    logger.info(`Revocation registry ${revRegId} rotated, new rev reg id ${newRevRegId}`)
    return { newRevReg, newRevRegId }
  }

  return {
    createRevocationRegistry,
    rotateRevocationRegistry
  }
}
