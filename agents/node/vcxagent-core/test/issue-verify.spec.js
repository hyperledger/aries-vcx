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
    const issuerDid = faber.getFaberDid()
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
    expect(presentationRequestAttachment.requested_attributes).toStrictEqual({
      attribute_0: {
        names: [
          'name',
          'last_name',
          'sex'
        ],
        restrictions: [
          {
            issuer_did: 'V4SGRU86Z58d6TV7PBUe6f'
          }
        ]
      },
      attribute_1: {
        name: 'date',
        restrictions: {
          issuer_did: 'V4SGRU86Z58d6TV7PBUe6f'
        }
      },
      attribute_2: {
        name: 'degree',
        restrictions: {
          'attr::degree::value': 'maths'
        }
      },
      attribute_3: {
        name: 'nickname',
        self_attest_allowed: true
      }
    })
    expect(presentationAttachment.requested_proof).toStrictEqual({
      revealed_attrs: {
        attribute_1: {
          sub_proof_index: 0,
          raw: '05-2018',
          encoded: '101085817956371643310471822530712840836446570298192279302750234554843339322886'
        },
        attribute_2: {
          sub_proof_index: 0,
          raw: 'maths',
          encoded: '78137204873448776862705240258723141940757006710839733585634143215803847410018'
        }
      },
      revealed_attr_groups: {
        attribute_0: {
          sub_proof_index: 0,
          values: {
            sex: {
              raw: 'female',
              encoded: '71957174156108022857985543806816820198680233386048843176560473245156249119752'
            },
            name: {
              raw: 'alice',
              encoded: '19831138297880367962895005496563562590284654704047651305948751287370224856720'
            },
            last_name: {
              raw: 'clark',
              encoded: '51192516729287562420368242940555165528396706187345387515033121164720912081028'
            }
          }
        }
      },
      self_attested_attrs: {
        attribute_3: 'Smith'
      },
      unrevealed_attrs: {},
      predicates: {
        predicate_0: {
          sub_proof_index: 0
        }
      }
    })
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
      expect(presentationAttachment.requested_proof).toStrictEqual({
        revealed_attrs: {},
        self_attested_attrs: {
          attribute_0: 'Smith'
        },
        unrevealed_attrs: {},
        predicates: {}
      })
      expect(presentationRequestAttachment.requested_attributes).toStrictEqual({
        attribute_0: {
          name: 'nickname',
          self_attest_allowed: true
        }
      })
      await sleep(500)
    } catch (err) {
      await sleep(500)
      console.error(`err = ${err.message} stack = ${err.stack}`)
      throw Error(err)
    }
  })
})
