/* eslint-env jest */
require('jest')
const express = require('express')
const axios = require('axios')
const { buildRevocationDetails, initRustLogger } = require('../src')
const { createPairedAliceAndFaber } = require('./utils/utils')
const { IssuerStateType, HolderStateType, VerifierStateType, ProverStateType } = require('@hyperledger/node-vcx-wrapper')
const uuid = require('uuid')
const sleep = require('sleep-promise')
const fs = require('fs')
const mkdirp = require('mkdirp')
const { proofRequestDataStandard } = require('./utils/data')

const TAILS_DIR_ALICE = '/tmp/alice/tails'
const TAILS_DIR_FABER = '/tmp/faber/tails'

jest.setTimeout(1000 * 60 * 4)

beforeAll(async () => {
  initRustLogger(process.env.RUST_LOG || 'vcx=error')
  mkdirp(TAILS_DIR_ALICE)
  mkdirp(TAILS_DIR_FABER)
})

describe('test tails distribution', () => {
  it('Faber uploads tails and Alice downloads using tails location info from ledger', async () => {
    let server
    try {
      const { alice, faber } = await createPairedAliceAndFaber()

      const port = 5468
      const tailsUrlId = uuid.v4()
      const tailsUrl = `http://127.0.0.1:${port}/${tailsUrlId}`
      await faber.buildLedgerPrimitives(buildRevocationDetails({ supportRevocation: true, tailsDir: TAILS_DIR_FABER, maxCreds: 5, tailsUrl }))
      await faber.sendCredentialOffer()
      await alice.acceptCredentialOffer()
      await faber.updateStateCredential(IssuerStateType.RequestReceived)
      await faber.sendCredential()
      await alice.updateStateCredential(HolderStateType.Finished)

      const faberTailsHash = await faber.getTailsHash()
      const app = express()
      app.use(`/${tailsUrlId}`, express.static(`${TAILS_DIR_FABER}/${faberTailsHash}`))
      server = app.listen(port)

      const aliceTailsLocation = await alice.getTailsLocation()
      const aliceTailsHash = await alice.getTailsHash()
      const aliceTailsFilePath = TAILS_DIR_ALICE + `/${aliceTailsHash}`
      axios.default.get(`${aliceTailsLocation}`, { responseType: 'stream' }).then(res => {
        res.data.pipe(fs.createWriteStream(aliceTailsFilePath))
      })
      const issuerDid = faber.getFaberDid()
      const request = await faber.requestProofFromAlice(proofRequestDataStandard(issuerDid))
      await alice.sendHolderProof(JSON.parse(request), revRegId => TAILS_DIR_ALICE, { attr_nickname: 'Smith' })
      await faber.updateStateVerifierProof(VerifierStateType.Finished)
      await alice.updateStateHolderProof(ProverStateType.Finished)
    } catch (err) {
      console.error(`err = ${err.message} stack = ${err.stack}`)
      if (server) {
        await server.close()
      }
      await sleep(2000)
      throw Error(err)
    }
  })
})
