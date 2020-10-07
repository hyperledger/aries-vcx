/* eslint-env jest */
require('jest')
const { createPairedAliceAndFaber } = require('./utils/utils')
const { initRustapi } = require('../src/index')

beforeAll(async () => {
  jest.setTimeout(1000 * 60 * 4)
  await initRustapi(process.env.VCX_LOG_LEVEL || 'vcx=error')
})

describe('test signatures', () => {
  it('Alice should sign data and Faber verify it', async () => {
    const { alice, faber } = await createPairedAliceAndFaber()

    const dataBase64 = Buffer.from('foobar').toString('base64')
    const signatureBase64 = await alice.signData(dataBase64)
    const isValid = await faber.verifySignature(dataBase64, signatureBase64)
    expect(isValid).toBeTruthy()
  })

  it('Faber should evaluate signature as invalid if was created by someone else', async () => {
    const { faber } = await createPairedAliceAndFaber()

    const dataBase64 = Buffer.from('foobar').toString('base64')
    // following is signature of "foobar" by some random key
    const signatureBase64 = 'aL2gZL2YfAieArCv5hrGznnwTEinnp9UU+X16axgtFIkX29M40v4n89iH35AtqApgfjvn6Okq6B8Q2IcKn+3DQ=='
    const isValid = await faber.verifySignature(dataBase64, signatureBase64)
    expect(isValid).toBeFalsy()
  })
})
