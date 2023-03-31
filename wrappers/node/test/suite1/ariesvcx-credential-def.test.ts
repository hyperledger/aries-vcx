import '../module-resolver-helper';

import { assert } from 'chai';
import { credentialDefCreate } from 'helpers/entities';
import { initVcxTestMode, shouldThrow } from 'helpers/utils';
import { CredentialDef, VCXCode } from 'src';

describe('CredentialDef:', () => {
  before(() => initVcxTestMode());

  describe('create:', () => {
    it('success', async () => {
      await credentialDefCreate();
    });
  });

  describe('serialize:', () => {
    it('success', async () => {
      const credentialDef = await credentialDefCreate();
      const serialized = await credentialDef.serialize();
      assert.ok(serialized);
      assert.property(serialized, 'version');
      assert.property(serialized, 'data');
      const { data, version } = serialized;
      assert.ok(data);
      assert.ok(version);
    });

    it('throws: not initialized', async () => {
      const credentialDef = new CredentialDef({} as any);
      const error = await shouldThrow(() => credentialDef.serialize());
      assert.equal(error.napiCode, 'NumberExpected');
    });
  });

  describe('deserialize:', () => {
    it('success', async () => {
      const credentialDef1 = await credentialDefCreate();
      const data1 = await credentialDef1.serialize();
      const credentialDef2 = await CredentialDef.deserialize(data1);
      const data2 = await credentialDef2.serialize();
      assert.deepEqual(data1, data2);
    });

    it('throws: incorrect data', async () => {
      const error = await shouldThrow(async () =>
        CredentialDef.deserialize({ data: { source_id: 'Invalid' } } as any),
      );
      assert.equal(error.napiCode, 'GenericFailure');
      assert.equal(error.vcxCode, VCXCode.CREATE_CREDENTIAL_DEF_ERR);
    });
  });

  describe('getCredDefId:', () => {
    it('success', async () => {
      const credentialDef = await credentialDefCreate();
      assert.equal(await credentialDef.getCredDefId(), '2hoqvcwupRTUNkXn6ArYzs:3:CL:2471');
    });

    it('throws: invalid handle', async () => {
      const credentialDef = await credentialDefCreate();
      credentialDef.releaseRustData()
      const error = await shouldThrow(() => credentialDef.getCredDefId());
      assert.equal(error.napiCode, 'GenericFailure');
      assert.equal(error.vcxCode, VCXCode.INVALID_OBJ_HANDLE);
    });
  });
});
