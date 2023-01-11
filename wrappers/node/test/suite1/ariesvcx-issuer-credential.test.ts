import '../module-resolver-helper';

import { assert } from 'chai';
import {
  createConnectionInviterRequested,
  credentialDefCreate,
  dataCredentialCreateWithOffer,
  issuerCredentialCreate,
} from 'helpers/entities';
import { initVcxTestMode, shouldThrow } from 'helpers/utils';
import { Connection, Credential, IssuerCredential, IssuerStateType, VCXCode } from 'src';

describe('IssuerCredential:', () => {
  before(() => initVcxTestMode());

  describe('create:', () => {
    it('success', async () => {
      await issuerCredentialCreate();
    });
  });

  describe('serialize:', () => {
    it('success', async () => {
      const issuerCredential = await issuerCredentialCreate();
      const serialized = await issuerCredential[0].serialize();
      assert.ok(serialized);
      assert.property(serialized, 'version');
      assert.property(serialized, 'data');
      const { data, version } = serialized;
      assert.ok(data);
      assert.ok(version);
    });

    it('throws: not initialized', async () => {
      const issuerCredential = new IssuerCredential('foo');
      const error = await shouldThrow(() => issuerCredential.serialize());
      assert.equal(error.napiCode, 'NumberExpected');
    });
  });

  describe('deserialize:', () => {
    it('success', async () => {
      const [issuerCredential1, data] = await issuerCredentialCreate();
      const data1 = await issuerCredential1.serialize();
      const issuerCredential2 = await IssuerCredential.deserialize(data1);
      const data2 = await issuerCredential2.serialize();
      assert.deepEqual(data1, data2);
    });

    it('throws: incorrect data', async () => {
      const error = await shouldThrow(async () =>
        IssuerCredential.deserialize({ source_id: 'Invalid' } as any),
      );
      assert.equal(error.napiCode, 'GenericFailure');
      assert.equal(error.vcxCode, VCXCode.INVALID_JSON);
    });

    it('throws: incomplete data', async () => {
      const error = await shouldThrow(async () =>
        IssuerCredential.deserialize({
          version: '2.0',
          data: {
            issuer_sm: {
              state: {
                SomeUnknown: {},
              },
              source_id: 'alice_degree',
            },
          },
        } as any),
      );
      assert.equal(error.napiCode, 'GenericFailure');
      assert.equal(error.vcxCode, VCXCode.INVALID_JSON);
    });
  });

  describe('updateState:', () => {
    it(`returns state offer sent`, async () => {
      const [issuerCredential, data] = await issuerCredentialCreate();
      await issuerCredential.buildCredentialOfferMsgV2(data);
      assert.equal(await issuerCredential.getState(), IssuerStateType.OfferSet);
      const connection = await createConnectionInviterRequested();
      await issuerCredential.sendOfferV2(connection);
      assert.equal(await issuerCredential.getState(), IssuerStateType.OfferSent);
    });

    it('build offer and mark as sent', async () => {
      const [issuerCredential, data] = await issuerCredentialCreate();
      await issuerCredential.buildCredentialOfferMsgV2(data);
      const offer = JSON.parse(await issuerCredential.getCredentialOfferMsg());
      assert.equal(await issuerCredential.getState(), IssuerStateType.OfferSet);
      assert.isDefined(offer['@id']);
      assert.equal(offer.comment, 'foo');
      assert.isDefined(offer.credential_preview);
      assert.equal(
        offer.credential_preview['@type'],
        'https://didcomm.org/issue-credential/1.0/credential-preview',
      );

      await issuerCredential.markCredentialOfferMsgSent();
      assert.equal(await issuerCredential.getState(), IssuerStateType.OfferSent);
    });

    it('throws: not initialized', async () => {
      const [, data] = await issuerCredentialCreate();
      const issuerCredential = new IssuerCredential('');
      const error = await shouldThrow(() => issuerCredential.buildCredentialOfferMsgV2(data));
      assert.equal(error.napiCode, 'NumberExpected');
    });

    it('throws: connection not initialized', async () => {
      const [issuerCredential, data] = await issuerCredentialCreate();
      await issuerCredential.buildCredentialOfferMsgV2(data);
      assert.equal(await issuerCredential.getState(), IssuerStateType.OfferSet);
      const connection = { handle: 123 } as Connection;
      const error = await shouldThrow(() => issuerCredential.sendOfferV2(connection));
      assert.equal(error.napiCode, 'GenericFailure');
      assert.equal(error.vcxCode, VCXCode.INVALID_OBJ_HANDLE);
    });

    // todo: _build_credential_preview should throw if attr is not JSON Array or Object
    it.skip('throws: missing attr', async () => {
      const [issuerCredential, buildOfferData] = await issuerCredentialCreate();
      buildOfferData.attr = '{{{' as any;
      const error = await shouldThrow(() =>
        issuerCredential.buildCredentialOfferMsgV2(buildOfferData),
      );
      assert.equal(error.napiCode, 'GenericFailure');
      assert.equal(error.vcxCode, VCXCode.INVALID_JSON);
    });

    it('throws: invalid credDefHandle', async () => {
      const [issuerCredential, buildOfferData] = await issuerCredentialCreate();
      buildOfferData.credDef = {} as any
      const error = await shouldThrow(() =>
        issuerCredential.buildCredentialOfferMsgV2(buildOfferData),
      );
      assert.equal(error.napiCode, 'NumberExpected');
    });
  });
});
