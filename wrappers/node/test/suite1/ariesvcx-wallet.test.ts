import '../module-resolver-helper';
import { assert } from 'chai';
import { initVcxTestMode, shouldThrow } from 'helpers/utils';
import { shutdownVcx, VCXCode, Wallet } from 'src';

describe('Wallet:', () => {
  before(() => initVcxTestMode());

  describe('import:', () => {
    it('throws: libindy error', async () => {
      let config =
        '{"wallet_name":"name","wallet_key":"","exported_wallet_path":"","backup_key":""}';
      let error = await shouldThrow(async () => Wallet.import(config));
      assert.equal(error.vcxCode, VCXCode.IO_ERROR);
      shutdownVcx(false);

      config = '{"wallet_key":"","exported_wallet_path":"","backup_key":""}';
      error = await shouldThrow(async () => Wallet.import(config));
      assert.equal(error.vcxCode, VCXCode.INVALID_CONFIGURATION);
      shutdownVcx(false);

      config = '{"wallet_name":"","exported_wallet_path":"","backup_key":""}';
      error = await shouldThrow(async () => Wallet.import(config));
      assert.equal(error.vcxCode, VCXCode.INVALID_CONFIGURATION);
      shutdownVcx(false);

      config = '{"wallet_name":"","wallet_key":"","backup_key":""}';
      error = await shouldThrow(async () => Wallet.import(config));
      assert.equal(error.vcxCode, VCXCode.INVALID_CONFIGURATION);
      shutdownVcx(false);

      config = '{"wallet_name":"","wallet_key":"","exported_wallet_path":""}';
      error = await shouldThrow(async () => Wallet.import(config));
      assert.equal(error.vcxCode, VCXCode.INVALID_CONFIGURATION);
    });
  });

  describe('export:', () => {
    it('throws: libindy error', async () => {
      const error = await shouldThrow(async () =>
        Wallet.export('/tmp/foobar.wallet', 'key_for_wallet'),
      );
      assert.equal(error.napiCode, 'GenericFailure');
      assert.equal(error.vcxCode, VCXCode.INVALID_WALLET_HANDLE);
    });
  });
});
