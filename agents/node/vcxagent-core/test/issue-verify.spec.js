/* eslint-env jest */
require('jest')
const { createPairedAliceAndFaber } = require('./utils/utils')
const { IssuerStateType, HolderStateType, ProverStateType, VerifierStateType, ProofState } = require('@hyperledger/node-vcx-wrapper')
const sleep = require('sleep-promise')
const { initRustLogger } = require('../src')
const { proofRequestDataStandard, proofRequestDataSelfAttest } = require('./utils/data')
const path = require('path')

beforeAll(async () => {
  jest.setTimeout(1000 * 60 * 4)
  initRustLogger(process.env.RUST_LOG || 'vcx=error')
})

afterAll(async () => {
  await sleep(500)
})

describe('test update state', () => {
  it('Faber should issue credential, verify proof', async () => {
    const { alice, faber } = await createPairedAliceAndFaber()
    const issuerDid = faber.getDi
    const tailsDir = path.join(__dirname, '/tmp/faber/tails')
    await faber.buildLedgerPrimitives({ tailsDir, maxCreds: 5 })
    await faber.rotateRevReg(tailsDir, 5)
    await faber.sendCredentialOffer()
    await alice.acceptCredentialOffer()

    await faber.updateStateCredential(IssuerStateType.RequestReceived)
    await faber.sendCredential()
    await alice.updateStateCredential(HolderStateType.Finished)
    await faber.receiveCredentialAck()

    const request = await faber.requestProofFromAlice(proofRequestDataStandard(issuerDid))
    await alice.sendHolderProof(JSON.parse(request), revRegId => tailsDir, { attribute_3: 'Smith' })
    await faber.updateStateVerifierProof(VerifierStateType.Finished)
    await alice.updateStateHolderProof(ProverStateType.Finished)
    const { presentationVerificationState, presentationAttachment, presentationRequestAttachment } = await faber.getPresentationInfo()
    expect(presentationVerificationState).toBe(ProofState.Verified)
    expect(presentationRequestAttachment.requested_attributes.attribute_0.names).toStrictEqual(['name', 'last_name', 'sex'])
    expect(presentationRequestAttachment.requested_attributes.attribute_1.name).toBe('date')
    expect(presentationRequestAttachment.requested_attributes.attribute_2.name).toBe('degree')
    expect(presentationRequestAttachment.requested_attributes.attribute_2.restrictions['attr::degree::value']).toBe('maths')
    expect(presentationRequestAttachment.requested_attributes.attribute_3.name).toBe('nickname')
    expect(presentationRequestAttachment.requested_attributes.attribute_3.self_attest_allowed).toBe(true)
    expect(presentationAttachment.requested_proof.revealed_attrs).toBeDefined()
    expect(presentationAttachment.requested_proof.revealed_attrs.attribute_1.raw).toBe('05-2018')
    expect(presentationAttachment.requested_proof.revealed_attrs.attribute_2.raw).toBe('maths')
    expect(presentationAttachment.requested_proof.revealed_attr_groups.attribute_0.values.name.raw).toBe('alice')
    expect(presentationAttachment.requested_proof.revealed_attr_groups.attribute_0.values.sex.raw).toBe('female')
    expect(presentationAttachment.requested_proof.revealed_attr_groups.attribute_0.values.last_name.raw).toBe('clark')
    expect(presentationAttachment.requested_proof.predicates.predicate_0).toBeDefined()
    expect(presentationAttachment.requested_proof.self_attested_attrs.attribute_3).toBe('Smith')
  })

  it('Faber should verify proof with self attestation', async () => {
    try {
      const { alice, faber } = await createPairedAliceAndFaber()
      const request = await faber.requestProofFromAlice(proofRequestDataSelfAttest())
      await alice.sendHolderProofSelfAttested(JSON.parse(request), { attribute_0: 'Smith' })
      await faber.updateStateVerifierProof(VerifierStateType.Finished)
      await alice.updateStateHolderProof(ProverStateType.Finished)
      const { presentationVerificationState, presentationAttachment, presentationRequestAttachment } = await faber.getPresentationInfo()
      expect(presentationVerificationState).toBe(ProofState.Verified)
      expect(presentationAttachment.requested_proof.self_attested_attrs.attribute_0).toBe('Smith')
      expect(presentationRequestAttachment.requested_attributes.attribute_0.name).toBe('nickname')
      expect(presentationRequestAttachment.requested_attributes.attribute_0.self_attest_allowed).toBe(true)
      await sleep(500)
    } catch (err) {
      await sleep(500)
      console.error(`err = ${err.message} stack = ${err.stack}`)
      throw Error(err)
    }
  })
})
