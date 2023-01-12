import '../module-resolver-helper';

import { assert } from 'chai';
import {
  createConnectionInviterRequested,
  credentialCreateWithOffer,
  dataCredentialCreateWithOffer,
} from 'helpers/entities';
import { initVcxTestMode, shouldThrow } from 'helpers/utils';
import { Credential, HolderStateType, VCXCode } from 'src';

describe('Credential:', () => {
  before(() => initVcxTestMode());

  describe('create:', () => {
    it('success', async () => {
      await credentialCreateWithOffer();
    });

    it('throws: missing sourceId', async () => {
      const { sourceId, ...data } = await dataCredentialCreateWithOffer();
      const error = await shouldThrow(() => Credential.create(data as any));
      assert.equal(error.napiCode, 'StringExpected');
    });

    it('throws: missing offer', async () => {
      const { offer, ...data } = await dataCredentialCreateWithOffer();
      const error = await shouldThrow(() => Credential.create(data as any));
      assert.equal(error.napiCode, 'StringExpected');
    });

    it('throws: invalid offer', async () => {
      const { offer, ...data } = await dataCredentialCreateWithOffer();
      const error = await shouldThrow(() => Credential.create({ offer: 'invalid', ...data }));
      assert.equal(error.vcxCode, VCXCode.INVALID_JSON);
    });
  });

  describe('serialize:', () => {
    it('success credential', async () => {
      const credential = await credentialCreateWithOffer();
      const serialized = await credential.serialize();
      assert.ok(serialized);
      assert.property(serialized, 'version');
      assert.property(serialized, 'data');
    });

    it('throws: not initialized', async () => {
      const credential = new Credential(null as any);
      const error = await shouldThrow(() => credential.serialize());
      assert.equal(error.napiCode, 'NumberExpected');
    });
  });

  describe('deserialize:', () => {
    it('success', async () => {
      const credential1 = await credentialCreateWithOffer();
      const data1 = await credential1.serialize();
      const credential2 = await Credential.deserialize(data1);
      const data2 = await credential2.serialize();
      assert.deepEqual(data1, data2);
    });

    it('throws: incorrect data', async () => {
      const error = await shouldThrow(async () =>
        Credential.deserialize({
          data: { source_id: 'Invalid' },
        } as any),
      );
      assert.equal(error.napiCode, 'GenericFailure');
      assert.equal(error.vcxCode, VCXCode.INVALID_JSON);
    });
  });

  describe('updateState:', () => {
    it(`returns status requestReceived`, async () => {
      const connection = await createConnectionInviterRequested();
      const data = await dataCredentialCreateWithOffer();
      const credential = await Credential.create(data);
      assert.equal(await credential.getState(), HolderStateType.OfferReceived);
      await credential.sendRequest({ connection });
      assert.equal(await credential.getState(), HolderStateType.RequestSent);
    });
  });

  describe('sendRequest:', () => {
    it('success: with offer', async () => {
      const data = await dataCredentialCreateWithOffer();
      const credential = await credentialCreateWithOffer(data);
      await credential.sendRequest({ connection: data.connection });
      assert.equal(await credential.getState(), HolderStateType.RequestSent);
    });
  });

  describe('getOffers:', () => {
    it('success', async () => {
      const connection = await createConnectionInviterRequested();
      const offers = await Credential.getOffers(connection);
      assert.ok(offers);
      assert.ok(offers.length);
      const offer = offers[0];
      await credentialCreateWithOffer({
        connection,
        offer: JSON.stringify(offer),
        sourceId: 'credentialGetOffersTestSourceId',
      });
    });
  });

  describe('getAttributes:', () => {
    it('success', async () => {
      const connection = await createConnectionInviterRequested();
      const offers = await Credential.getOffers(connection);
      assert.ok(offers);
      assert.ok(offers.length);
      const offer = offers[0];
      const credential = await credentialCreateWithOffer({
        connection,
        offer: JSON.stringify(offer),
        sourceId: 'credentialGetAttributesTestSourceId',
      });
      const attrs = JSON.parse(credential.getAttributes());
      const expectedAttrs = {
        last_name: 'clark',
        sex: 'female',
        degree: 'maths',
        date: '05-2018',
        age: '25',
        name: 'alice',
      };
      assert.deepEqual(attrs, expectedAttrs);
    });
  });

  describe('getAttachment:', () => {
    it('success', async () => {
      const connection = await createConnectionInviterRequested();
      const offers = await Credential.getOffers(connection);
      assert.ok(offers);
      assert.ok(offers.length);
      const offer = offers[0];
      const credential = await credentialCreateWithOffer({
        connection,
        offer: JSON.stringify(offer),
        sourceId: 'credentialGetAttributesTestSourceId',
      });
      const attach = JSON.parse(credential.getAttachment());
      assert.deepEqual(attach.schema_id, 'V4SGRU86Z58d6TV7PBUe6f:2:FaberVcx:83.23.62');
    });
  });
});
