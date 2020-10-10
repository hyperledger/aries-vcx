/**
 * Based on credentials in the wallet and proof request, this function builds data structure specifying how to
 * build proof complying with the proof request.
 * @param holderProof - deserialized libvcx Proof object
 * @param logger - logger implementing debug and info functions
 * @param mapRevRegIdToTailsFilePath - function receiving 1 argument - rev_reg_id, mapping it to a path pointing to
 * file containing tails file for the given revocation registry.
 * @returns {Promise<{attrs: {}}>}
 */
module.exports.holderSelectCredentialsForProof = async function holderSelectCredentialsForProof (holderProof, logger, mapRevRegIdToTailsFilePath) {
  const resolvedCreds = await holderProof.getCredentials()
  const selectedCreds = { attrs: {} }
  logger.debug(`Resolved credentials for proof = ${JSON.stringify(resolvedCreds, null, 2)}`)

  for (const attrName of Object.keys(resolvedCreds.attrs)) {
    const attrCredInfo = resolvedCreds.attrs[attrName]
    if (Array.isArray(attrCredInfo) === false) {
      throw Error('Unexpected data, expected attrCredInfo to be an array.')
    }
    if (attrCredInfo.length > 0) {
      selectedCreds.attrs[attrName] = {
        credential: resolvedCreds.attrs[attrName][0]
      }
      const revRegId = resolvedCreds.attrs[attrName][0].cred_info.rev_reg_id
      if (revRegId) {
        selectedCreds.attrs[attrName].tails_file = await mapRevRegIdToTailsFilePath(revRegId)
      }
    } else {
      logger.info(`No credential was resolved for requested attribute key ${attrName}, will have to be supplied via self-attested attributes.`)
    }
  }
  logger.debug(`Selected credentials:\n${JSON.stringify(selectedCreds, null, 2)}`)
  return selectedCreds
}

module.exports.extractProofRequestAttachement = function extractProofRequestAttachement (proofRequest) {
  const attachment = proofRequest['request_presentations~attach'][0].data.base64
  return JSON.parse(Buffer.from(attachment, 'base64').toString('utf8'))
}
