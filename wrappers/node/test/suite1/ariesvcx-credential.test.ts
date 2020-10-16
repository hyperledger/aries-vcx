import '../module-resolver-helper'

import { assert } from 'chai'
import { validatePaymentTxn } from 'helpers/asserts'
import {
  createConnectionInviterRequested,
  credentialCreateWithMsgId,
  credentialCreateWithOffer,
  dataCredentialCreateWithMsgId,
  dataCredentialCreateWithOffer
} from 'helpers/entities'
import { initVcxTestMode, shouldThrow } from 'helpers/utils'
import {
  Credential,
  CredentialPaymentManager,
  StateType,
  VCXCode,
  VCXMock,
  VCXMockMessage
} from 'src'
import { PROTOCOL_TYPE_ARIES_STRICT } from '../helpers/test-constants'

describe('Credential:', () => {
  before(() => initVcxTestMode(PROTOCOL_TYPE_ARIES_STRICT))

  describe('create:', () => {
    it('success', async () => {
      await credentialCreateWithOffer()
    })

    it('throws: missing sourceId', async () => {
      const { sourceId, ...data } = await dataCredentialCreateWithOffer()
      const error = await shouldThrow(() => Credential.create(data as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_OPTION)
    })

    it('throws: missing offer', async () => {
      const { offer, ...data } = await dataCredentialCreateWithOffer()
      const error = await shouldThrow(() => Credential.create(data as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_OPTION)
    })

    // Enable when we start utilizing connection prop
    it.skip('throws: missing connection', async () => {
      const { connection, ...data } = await dataCredentialCreateWithOffer()
      const error = await shouldThrow(() => Credential.create({ connection: {} as any, ...data }))
      assert.equal(error.vcxCode, VCXCode.INVALID_OPTION)
    })

    it('throws: invalid offer', async () => {
      const { offer, ...data } = await dataCredentialCreateWithOffer()
      const error = await shouldThrow(() => Credential.create({ offer: 'invalid', ...data }))
      assert.equal(error.vcxCode, VCXCode.INVALID_JSON)
    })
  })

  describe('serialize:', () => {
    it('success credential', async () => {
      const credential = await credentialCreateWithOffer()
      const serialized = await credential.serialize()
      assert.ok(serialized)
      assert.property(serialized, 'version')
      assert.property(serialized, 'data')
    })

    it('throws: not initialized', async () => {
      const credential = new Credential(null as any)
      const error = await shouldThrow(() => credential.serialize())
      assert.equal(error.vcxCode, VCXCode.INVALID_CREDENTIAL_HANDLE)
    })

  })

  describe('deserialize:', () => {
    it('success', async () => {
      const credential1 = await credentialCreateWithOffer()
      const data1 = await credential1.serialize()
      const credential2 = await Credential.deserialize(data1)
      const data2 = await credential2.serialize()
      assert.deepEqual(data1, data2)
    })

    it('throws: incorrect data', async () => {
      const error = await shouldThrow(async () => Credential.deserialize({
        data: { source_id: 'Invalid' } } as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_JSON)
    })
  })

  describe('updateState:', () => {
    it(`returns ${StateType.None}: not initialized`, async () => {
      const credential = new Credential(null as any)
      const state1 = await credential.updateState()
      const state2 = await credential.getState()
      assert.equal(state1, state2)
      assert.equal(state2, StateType.None)
    })

    it(`returns status requestReceived`, async () => {
      const connection = await createConnectionInviterRequested()
      const data = await dataCredentialCreateWithOffer()
      const credential = await Credential.create(data)
      assert.equal(await credential.getState(), StateType.RequestReceived)
      credential.sendRequest({ connection, payment: 0 })
      assert.equal(await credential.getState(), StateType.OfferSent)
    })
  })

  describe('sendRequest:', () => {
    it('success: with offer', async () => {
      const data = await dataCredentialCreateWithOffer()
      const credential = await credentialCreateWithOffer(data)
      await credential.sendRequest({ connection: data.connection, payment: 0 })
      assert.equal(await credential.getState(), StateType.OfferSent)
    })

    it('success: with message id', async () => {
      const data = await dataCredentialCreateWithMsgId()
      const credential = await credentialCreateWithMsgId(data)
      await credential.sendRequest({ connection: data.connection, payment: 0 })
      assert.equal(await credential.getState(), StateType.OfferSent)
    })

    // todo : restore for aries
    it.skip('success: get request message', async () => {
      const data = await dataCredentialCreateWithOffer()
      const credential = await credentialCreateWithOffer(data)
      const pwDid = await data.connection.getPwDid()
      const msg = await credential.getRequestMessage({ myPwDid: pwDid, payment: 0 })
      assert(msg.length > 0)
    })

    // todo : restore for aries
    it.skip('success: issued', async () => {
      const data = await dataCredentialCreateWithOffer()
      const credential = await credentialCreateWithOffer(data)
      await credential.sendRequest({ connection: data.connection, payment: 0 })
      assert.equal(await credential.getState(), StateType.OfferSent)
      VCXMock.setVcxMock(VCXMockMessage.CredentialResponse)
      VCXMock.setVcxMock(VCXMockMessage.UpdateIssuerCredential)
      await credential.updateState()
      assert.equal(await credential.getState(), StateType.Accepted)
    })
  })

  describe('getOffers:', () => {
    it('success', async () => {
      const connection = await createConnectionInviterRequested()
      const offers = await Credential.getOffers(connection)
      assert.ok(offers)
      assert.ok(offers.length)
      const offer = offers[0]
      await credentialCreateWithOffer({
        connection,
        offer: JSON.stringify(offer),
        sourceId: 'credentialGetOffersTestSourceId'
      })
    })
  })

  describe('getAttributes:', () => {
    it('success', async () => {
      const connection = await createConnectionInviterRequested()
      const offers = await Credential.getOffers(connection)
      assert.ok(offers)
      assert.ok(offers.length)
      const offer = offers[0]
      const credential = await credentialCreateWithOffer({
        connection,
        offer: JSON.stringify(offer),
        sourceId: 'credentialGetAttributesTestSourceId'
      })
      const attrs = JSON.parse(await credential.getAttributes(connection))
      const expectedAttrs = JSON.parse('{"last_name":"clark","sex":"female","degree":"maths","date":"05-2018","age":"25","name":"alice"}')
      assert.deepEqual(attrs, expectedAttrs)
    })
  })

  describe('getPaymentInfo:', () => {
    it.skip('success', async () => {
      const credential = await credentialCreateWithOffer()
      const paymentInfo = await credential.getPaymentInfo()
      assert.ok(paymentInfo)
    })
  })

  // todo: ?restore for aries?
  describe('paymentManager:', () => {
    it.skip('exists', async () => {
      const credential = await credentialCreateWithOffer()
      assert.instanceOf(credential.paymentManager, CredentialPaymentManager)
      assert.equal(credential.paymentManager.handle, credential.handle)
    })

    describe('getPaymentTxn:', () => {
      it.skip('success', async () => {
        const data = await dataCredentialCreateWithOffer()
        const credential = await credentialCreateWithOffer(data)
        await credential.sendRequest({ connection: data.connection, payment: 0 })
        assert.equal(await credential.getState(), StateType.OfferSent)
        VCXMock.setVcxMock(VCXMockMessage.CredentialResponse)
        VCXMock.setVcxMock(VCXMockMessage.UpdateIssuerCredential)
        await credential.updateState()
        assert.equal(await credential.getState(), StateType.Accepted)
        const paymentTxn = await credential.paymentManager.getPaymentTxn()
        validatePaymentTxn(paymentTxn)
      })
    })
  })

  describe('createWithMsgId', () => {
    // TODO: to enable this test, credential offer must be mocked in aries code of get_credential_offer_msg
    it.skip('createWithMsgIdsuccess', async () => {
      await credentialCreateWithMsgId()
    })

    it('throws: missing sourceId', async () => {
      const { connection, msgId } = await dataCredentialCreateWithMsgId()
      const error = await shouldThrow(() => Credential.createWithMsgId({ connection, msgId } as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_OPTION)
    })

    it('throws: missing offer', async () => {
      const { connection, sourceId } = await dataCredentialCreateWithMsgId()
      const error = await shouldThrow(() => Credential.createWithMsgId({ connection, sourceId } as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_OPTION)
    })

    it('throws: missing connection', async () => {
      const { connection, ...data } = await dataCredentialCreateWithMsgId()
      const error = await shouldThrow(() => Credential.createWithMsgId(data as any))
      assert.equal(error.vcxCode, VCXCode.UNKNOWN_ERROR)
    })

    it('throws: missing connection handle', async () => {
      const { connection, ...data } = await dataCredentialCreateWithMsgId()
      const error = await shouldThrow(() => Credential.createWithMsgId({ connection: {} as any, ...data }))
      assert.equal(error.vcxCode, VCXCode.INVALID_OBJ_HANDLE)
    })
  })

})
