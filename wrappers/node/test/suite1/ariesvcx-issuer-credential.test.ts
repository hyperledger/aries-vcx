import '../module-resolver-helper';

import { assert } from 'chai';
import {
  createConnectionInviterRequested, credentialDefCreate,
  dataIssuerCredentialCreate,
  issuerCredentialCreate,
} from 'helpers/entities'
import { initVcxTestMode, shouldThrow } from 'helpers/utils';
import { Connection, IssuerCredential, IssuerStateType, VCXCode } from 'src';

describe('IssuerCredential:', () => {
  before(() => initVcxTestMode());

  describe('create:', () => {
    it('success', async () => {
      await issuerCredentialCreate();
    });

    it('throws: missing sourceId', async () => {
      const data = await dataIssuerCredentialCreate();
      const error = await shouldThrow(() => IssuerCredential.create(''));
      assert.equal(error.vcxCode, VCXCode.INVALID_OPTION);
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
      const issuerCredential = new IssuerCredential('');
      const error = await shouldThrow(() => issuerCredential.serialize());
      assert.equal(error.vcxCode, VCXCode.INVALID_OBJ_HANDLE);
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
      assert.equal(error.vcxCode, VCXCode.UNKNOWN_ERROR);
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
      assert.equal(error.vcxCode, VCXCode.INVALID_JSON);
    });
  });

  describe('updateState:', () => {
    it(`returns state offer sent`, async () => {
      const [issuerCredential, data] = await issuerCredentialCreate();
      await issuerCredential.sendOffer(data);
      assert.equal(await issuerCredential.getState(), IssuerStateType.OfferSent);
    });
  });

  describe('sendOffer:', () => {
    it('success', async () => {
      const [issuerCredential, data] = await issuerCredentialCreate();
      await issuerCredential.sendOffer(data);
      assert.equal(await issuerCredential.getState(), IssuerStateType.OfferSent);
    });

    it('success sendOfferV2', async () => {
      const connection = await createConnectionInviterRequested();
      const credDef = await credentialDefCreate();
      const issuerCredential = await IssuerCredential.create('testCredentialSourceId');
      const attr = {
        key1: 'value1',
        key2: 'value2',
        key3: 'value3',
      }
      await issuerCredential.buildCredentialOfferMsg({credDef, attr, comment: "hello"})
      await issuerCredential.sendOfferV2(connection);
      assert.equal(await issuerCredential.getState(), IssuerStateType.OfferSent);
    });

    it('build offer and mark as sent', async () => {
      const issuerCredential = await IssuerCredential.create('testCredentialSourceId');
      const credDef = await credentialDefCreate();
      const attr = {
          key1: 'value1',
          key2: 'value2',
          key3: 'value3',
      }
      await issuerCredential.buildCredentialOfferMsg({ credDef, attr, comment: "c1" });
      assert.equal(await issuerCredential.getState(), IssuerStateType.OfferSet);
      const offer = JSON.parse(await issuerCredential.getCredentialOfferMsg())
      // @ts-ignore
      assert.isDefined(offer['@id']);
      assert.equal(offer.comment, 'c1');
      assert.isDefined(offer.credential_preview);
      assert.equal(offer.credential_preview['@type'], 'https://didcomm.org/issue-credential/1.0/credential-preview');

      await issuerCredential.markCredentialOfferMsgSent();
      assert.equal(await issuerCredential.getState(), IssuerStateType.OfferSent);
    });

    it('throws: not initialized', async () => {
      const [_issuerCredential, data] = await issuerCredentialCreate();
      const issuerCredential = new IssuerCredential('');
      const error = await shouldThrow(() => issuerCredential.sendOffer(data));
      assert.equal(error.vcxCode, VCXCode.INVALID_OBJ_HANDLE);
    });

    it('throws: connection not initialized', async () => {
      const connection = new (Connection as any)();
      const [issuerCredential, data] = await issuerCredentialCreate();
      data.connection = connection;
      const error = await shouldThrow(() => issuerCredential.sendOffer(data));
      assert.equal(error.vcxCode, VCXCode.INVALID_OBJ_HANDLE);
    });

    // "vcx_issuer_get_credential_offer_msg" not implemented for Aries
    it.skip('can generate the offer message', async () => {
      const [issuerCredential, data] = await issuerCredentialCreate();
      const message = await issuerCredential.getCredentialOfferMsg();
      assert(message.length > 0);
    });

    it('throws: missing attr', async () => {
      const [issuerCredential, _data] = await issuerCredentialCreate();
      const { attr, ...data } = _data;
      const error = await shouldThrow(() => issuerCredential.sendOffer(data as any));
      assert.equal(error.vcxCode, VCXCode.INVALID_OPTION);
    });

    it('throws: invalid credDefHandle', async () => {
      const [issuerCredential, _data] = await issuerCredentialCreate();
      const { credDef, ...data } = _data;
      const error = await shouldThrow(() => issuerCredential.sendOffer(data as any));
      assert.equal(error.vcxCode, VCXCode.UNKNOWN_ERROR);
    });
  });

  describe('sendCredential:', () => {
    it('throws: not initialized', async () => {
      const connection = await createConnectionInviterRequested();
      const issuerCredential = new IssuerCredential('');
      const error = await shouldThrow(() => issuerCredential.sendCredential(connection));
      assert.equal(error.vcxCode, VCXCode.INVALID_OBJ_HANDLE);
    });

    // todo: recorder this test/behaviour in 4.0, issuerCredential is not throwing, only prints warning
    it.skip('throws: no offer', async () => {
      const connection = await createConnectionInviterRequested();
      const issuerCredential = await issuerCredentialCreate();
      const error = await shouldThrow(() => issuerCredential[0].sendCredential(connection));
      assert.equal(error.vcxCode, VCXCode.NOT_READY);
    });

    // todo: recorder this test/behaviour in 4.0, issuerCredential is not throwing, only prints warning
    it.skip('throws: no request', async () => {
      const [issuerCredential, data] = await issuerCredentialCreate();
      await issuerCredential.sendOffer(data);
      const error = await shouldThrow(() => issuerCredential.sendCredential(data.connection));
      assert.equal(error.vcxCode, VCXCode.NOT_READY);
    });
  });

  // describe('revoke:', () => {
  //   it('throws: invalid revocation details', async () => {
  //     const issuerCredential = await issuerCredentialCreate()
  //     const error = await shouldThrow(() => issuerCredential.revokeCredential())
  //     assert.equal(error.vcxCode, VCXCode.INVALID_REVOCATION_DETAILS)
  //   })
  //
  //   it('success', async () => {
  //     const issuerCredential1 = await issuerCredentialCreate()
  //     const data = await issuerCredential1[0].serialize()
  //     data.data.cred_rev_id = '123'
  //     data.data.rev_reg_id = '456'
  //     data.data.tails_file = 'file'
  //     const issuerCredential2 = await IssuerCredential.deserialize(data)
  //     await issuerCredential2.revokeCredential()
  //   })
  // })
});
