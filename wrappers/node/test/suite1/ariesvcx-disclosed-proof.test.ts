import '../module-resolver-helper'

import { assert } from 'chai'
import {
  createConnectionInviterRequested,
  dataDisclosedProofCreateWithMsgId,
  dataDisclosedProofCreateWithRequest,
  disclosedProofCreateWithMsgId,
  disclosedProofCreateWithRequest
} from 'helpers/entities'
import { initVcxTestMode, shouldThrow } from 'helpers/utils'
import { mapValues } from 'lodash'
import { DisclosedProof, StateType, VCXCode } from 'src'
import { PROTOCOL_TYPE_ARIES_STRICT } from '../helpers/test-constants'

describe('DisclosedProof', () => {
  before(() => initVcxTestMode(PROTOCOL_TYPE_ARIES_STRICT))

  describe('create:', () => {
    it('success1', async () => {
      await disclosedProofCreateWithRequest()
    })

    it('throws: missing sourceId', async () => {
      const { connection, request } = await dataDisclosedProofCreateWithRequest()
      const error = await shouldThrow(() => DisclosedProof.create({ connection, request } as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_OPTION)
    })

    it('throws: missing request', async () => {
      const { connection, sourceId } = await dataDisclosedProofCreateWithRequest()
      const error = await shouldThrow(() => DisclosedProof.create({ connection, sourceId } as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_OPTION)
    })

    // Enable when we start utilizing connection prop
    it.skip('throws: missing connection', async () => {
      const { request, sourceId } = await dataDisclosedProofCreateWithRequest()
      const error = await shouldThrow(() => DisclosedProof.create({ request, sourceId } as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_OPTION)
    })

    it('throws: invalid request', async () => {
      const { connection, sourceId } = await dataDisclosedProofCreateWithRequest()
      const error = await shouldThrow(() => DisclosedProof.create({ connection, request: 'invalid', sourceId }))
      assert.equal(error.vcxCode, VCXCode.INVALID_JSON)
    })
  })

  describe('createWithMsgId:', () => {
    // todo: migrate to aries
    it.skip('success', async () => {
      await disclosedProofCreateWithMsgId()
    })

    it('throws: missing sourceId', async () => {
      const { connection, msgId } = await dataDisclosedProofCreateWithMsgId()
      const error = await shouldThrow(() => DisclosedProof.createWithMsgId({ connection, msgId } as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_OPTION)
    })

    it('throws: missing request', async () => {
      const { connection, sourceId } = await dataDisclosedProofCreateWithMsgId()
      const error = await shouldThrow(() => DisclosedProof.createWithMsgId({ connection, sourceId } as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_OPTION)
    })

    it('throws: missing connection', async () => {
      const { connection, ...data } = await dataDisclosedProofCreateWithMsgId()
      const error = await shouldThrow(() => DisclosedProof.createWithMsgId({ data } as any))
      assert.equal(error.vcxCode, VCXCode.UNKNOWN_ERROR)
    })

    it('throws: missing connection handle', async () => {
      const { connection, ...data } = await dataDisclosedProofCreateWithMsgId()
      const error = await shouldThrow(() => DisclosedProof.createWithMsgId({ connection: {} as any, ...data }))
      assert.equal(error.vcxCode, VCXCode.INVALID_CONNECTION_HANDLE)
    })
  })

  describe('serialize:', () => {
    it('success', async () => {
      const disclosedProof = await disclosedProofCreateWithRequest()
      const serialized = await disclosedProof.serialize()
      assert.ok(serialized)
      assert.property(serialized, 'version')
      assert.property(serialized, 'data')
    })

    it('throws: not initialized', async () => {
      const disclosedProof = new (DisclosedProof as any)()
      const error = await shouldThrow(() => disclosedProof.serialize())
      assert.equal(error.vcxCode, VCXCode.INVALID_DISCLOSED_PROOF_HANDLE)
    })
  })

  describe('deserialize:', () => {
    it('success', async () => {
      const disclosedProof1 = await disclosedProofCreateWithRequest()
      const data1 = await disclosedProof1.serialize()
      const disclosedProof2 = await DisclosedProof.deserialize(data1)
      const data2 = await disclosedProof2.serialize()
      assert.deepEqual(data1, data2)
    })

    it('throws: incorrect data', async () => {
      const error = await shouldThrow(async () => DisclosedProof.deserialize({ data: { source_id: 'Invalid' } } as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_JSON)
    })
  })

  describe('updateState:', () => {
    it(`returns ${StateType.None}: not initialized`, async () => {
      const disclosedProof = new (DisclosedProof as any)()
      const state1 = await disclosedProof.updateState()
      const state2 = await disclosedProof.getState()
      assert.equal(state1, state2)
      assert.equal(state2, StateType.None)
    })

    it(`returns ${StateType.RequestReceived}: created`, async () => {
      const disclosedProof = await disclosedProofCreateWithRequest()
      await disclosedProof.updateState()
      assert.equal(await disclosedProof.getState(), StateType.RequestReceived)
    })
  })

  describe('sendProof:', () => {
    it.skip('success', async () => {
      const data = await dataDisclosedProofCreateWithRequest()
      const disclosedProof = await disclosedProofCreateWithRequest(data)
      await disclosedProof.sendProof(data.connection)
      assert.equal(await disclosedProof.getState(), StateType.Accepted)
    })
  })

  describe('getRequests:', async () => {
    it.skip('success', async () => {
      const connection = await createConnectionInviterRequested()
      const requests = await DisclosedProof.getRequests(connection)
      assert.ok(requests)
      assert.ok(requests.length)
      const request = requests[0]
      const disclosedProof = await disclosedProofCreateWithRequest({
        connection,
        request: JSON.stringify(request),
        sourceId: 'disclosedProofTestSourceId'
      })
      await disclosedProof.updateState()
      assert.equal(await disclosedProof.getState(), StateType.RequestReceived)
    })
  })

  describe('generateProof:', async () => {
    it.skip('success', async () => {
      const data = await dataDisclosedProofCreateWithRequest()
      const disclosedProof = await disclosedProofCreateWithRequest(data)
      const { attrs } = await disclosedProof.getCredentials()
      const valSelfAttested = 'testSelfAttestedVal'
      await disclosedProof.generateProof({
        selectedCreds: {},
        selfAttestedAttrs: mapValues(attrs, () => valSelfAttested)
      })
      await disclosedProof.sendProof(data.connection)
    })
  })

  describe('getAttributes:', async () => {
    it('success', async () => {
      const data = await dataDisclosedProofCreateWithRequest()
      const disclosedProof = await disclosedProofCreateWithRequest(data)
      const attrs = await disclosedProof.getAttributes()
      const expectedAttrs = '{"name":"proofForAlice","non_revoked":{"from":null,"to":1599834712270},"nonce":"1137618739380422483900458","requested_attributes":{"attribute_0":{"names":["name","last_name","sex"],"restrictions":{"$or":[{"issuer_did":"V4SGRU86Z58d6TV7PBUe6f"}]}},"attribute_1":{"name":"date","restrictions":{"issuer_did":"V4SGRU86Z58d6TV7PBUe6f"}},"attribute_2":{"name":"degree","restrictions":{"attr::degree::value":"maths"}},"attribute_3":{"name":"nickname"}},"requested_predicates":{"predicate_0":{"name":"age","p_type":">=","p_value":20,"restrictions":{"$or":[{"issuer_did":"V4SGRU86Z58d6TV7PBUe6f"}]}}},"ver":"1.0","version":"1.0"}'
      assert.equal(attrs, expectedAttrs)
    })
  })

  describe('rejectProof:', async () => {
    it('success', async () => {
      const data = await dataDisclosedProofCreateWithRequest()
      const disclosedProof = await disclosedProofCreateWithRequest(data)
      await disclosedProof.rejectProof(data.connection)
    })
  })

  describe('declinePresentationRequest:', () => {
    it('success', async () => {
      const data = await dataDisclosedProofCreateWithRequest()
      await disclosedProofCreateWithRequest(data)
    })
  })

})
