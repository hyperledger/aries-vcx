function getRandomInt (min, max) {
  min = Math.ceil(min)
  max = Math.floor(max)
  return Math.floor(Math.random() * (max - min)) + min
}

function getFaberSchemaData () {
  const version = `${getRandomInt(1, 101)}.${getRandomInt(1, 101)}.${getRandomInt(1, 101)}`
  return {
    data: {
      attrNames: ['name', 'last_name', 'sex', 'date', 'degree', 'age'],
      name: 'FaberVcx',
      version
    },
    paymentHandle: 0,
    sourceId: `your-identifier-fabervcx-${version}`
  }
}

function getFaberCredDefName () {
  return 'DemoCredential123'
}

function getFaberProofData (issuerDid) {
  const proofAttributes = [
    {
      names: ['name', 'last_name', 'sex'],
      restrictions: [{ issuer_did: issuerDid }]
    },
    {
      name: 'date',
      restrictions: { issuer_did: issuerDid }
    },
    {
      name: 'degree',
      restrictions: { 'attr::degree::value': 'maths' }
    },
    {
      name: 'nickname',
      self_attest_allowed: true
    }
  ]

  const proofPredicates = [
    { name: 'age', p_type: '>=', p_value: 20, restrictions: [{ issuer_did: issuerDid }] }
  ]

  return {
    sourceId: '213',
    attrs: proofAttributes,
    preds: proofPredicates,
    name: 'proofForAlice',
    revocationInterval: { to: Date.now() }
  }
}

function getAliceSchemaAttrs () {
  return {
    name: 'alice',
    last_name: 'clark',
    sex: 'female',
    date: '05-2018',
    degree: 'maths',
    age: '25'
  }
}

module.exports.getFaberSchemaData = getFaberSchemaData
module.exports.getAliceSchemaAttrs = getAliceSchemaAttrs
module.exports.getFaberCredDefName = getFaberCredDefName
module.exports.getFaberProofData = getFaberProofData
