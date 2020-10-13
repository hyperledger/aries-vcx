module.exports.filterOffersBySchema = function filterOffersBySchema (offers, schemaIdRegex) {
  return offers.filter(offer => {
    const base64data = offer['offers~attach'][0].data.base64
    const data = JSON.parse(Buffer.from(base64data, 'base64').toString('utf-8'))
    const matches = data.schema_id.match(schemaIdRegex)
    return matches && matches.length > 0
  })
}

module.exports.filterOffersByAttr = function filterOffersByAttr (offers, attrRegex) {
  return offers.filter(offer => offer.credential_preview.attributes.find(attr => attr.name.match(attrRegex.name) && attr.value.match(attrRegex.value)))
}
