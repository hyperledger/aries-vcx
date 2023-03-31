function getFaberCredDefName () {
  return 'DemoCredential123'
}

function getFaberProofDataWithNonRevocation (issuerDid) {
  const proofData = proofRequestDataStandard(issuerDid)
  proofData.revocationInterval = { to: Date.now() }
  return proofData
}

function proofRequestDataStandard (issuerDid) {
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
    name: 'proof-test-standard',
    revocationInterval: { to: null, from: null }
  }
}

function proofRequestDataSelfAttest () {
  const proofAttributes = [
    {
      name: 'nickname',
      self_attest_allowed: true
    }
  ]

  return {
    sourceId: '213',
    attrs: proofAttributes,
    name: 'proof-test-self-attesting',
    revocationInterval: { to: null, from: null }
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

module.exports = {
  getAliceSchemaAttrs,
  getFaberCredDefName,
  proofRequestDataStandard,
  getFaberProofDataWithNonRevocation,
  proofRequestDataSelfAttest
}
