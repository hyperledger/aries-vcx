module.exports.holderSelectCredentialsForProof = async function holderSelectCredentialsForProof (holderProof, logger) {
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
      selectedCreds.attrs[attrName].tails_file = '/tmp/tails'
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
