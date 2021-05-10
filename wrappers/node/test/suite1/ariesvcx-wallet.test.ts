import '../module-resolver-helper';

import { assert } from 'chai';
import { validateUTXO } from 'helpers/asserts';
import { initVcxTestMode, shouldThrow } from 'helpers/utils';
import { shutdownVcx, VCXCode, Wallet } from 'src';

const WALLET_RECORD = {
  id: 'RecordId',
  tags_json: {},
  type_: 'TestType',
  value: 'RecordValue',
};

const OPTIONS = {
  retrieveTags: false,
  retrieveType: true,
  retrieveValue: true,
};
const QUERY_JSON = { tagName1: 'str1' };

const UPDATE_WALLET_RECORD = {
  id: 'RecordId',
  type_: 'TestType',
  value: 'RecordValueNew',
};

const UPDATE_WALLET_TAGS = {
  id: 'RecordId',
  tags_json: {},
  type_: 'TestType',
  value: '',
};

const PAYMENT_ADDRESS_SEED = {
  seed: '0000000000000000000WHATEVER00000',
};

describe('Wallet:', () => {
  before(() => initVcxTestMode());


  describe('createPaymentAddress:', () => {
    it('success', async () => {
      const address = await Wallet.createPaymentAddress(PAYMENT_ADDRESS_SEED);
      assert.ok(address);
    });
  });

  describe('validatePaymentAddress:', () => {
    it('success', async () => {
      await Wallet.validatePaymentAddress('sov:1:1234');
    });
  });

  describe('signWithAddress:', () => {
    it('success', async () => {
      const msg = Buffer.from('random string');
      const sig = await Wallet.signWithAddress('pay:sov:1234', msg);
      assert.ok(sig);
    });
  });

  describe('verifyWithAddress:', () => {
    it('success', async () => {
      const msg = Buffer.from('random string');
      const sig = Buffer.from('random string');
      const valid = await Wallet.verifyWithAddress('pay:sov:1234', msg, sig);
      assert.ok(valid);
    });
  });

  describe('records:', () => {
    it('success', async () => {
      await Wallet.addRecord(WALLET_RECORD);
      await Wallet.getRecord({
        type: WALLET_RECORD.type_,
        id: WALLET_RECORD.id,
        optionsJson: OPTIONS,
      });
      await Wallet.updateRecordValue(UPDATE_WALLET_RECORD);
      await Wallet.updateRecordTags(UPDATE_WALLET_TAGS);
      await Wallet.addRecordTags(UPDATE_WALLET_TAGS);
      await Wallet.deleteRecordTags(WALLET_RECORD, { tagList: ['one', 'two'] });
      await Wallet.deleteRecord({ type: WALLET_RECORD.type_, id: WALLET_RECORD.id });
      const searchHandle = await Wallet.openSearch({
        optionsJson: '{}',
        queryJson: JSON.stringify(QUERY_JSON),
        type: WALLET_RECORD.type_,
      });
      assert(searchHandle === 1);
      JSON.parse(await Wallet.searchNextRecords(searchHandle, { count: 1 }));
      await Wallet.closeSearch(searchHandle);
    });
  });

  describe('import:', () => {
    it('throws: libindy error', async () => {
      let config =
        '{"wallet_name":"name","wallet_key":"","exported_wallet_path":"","backup_key":""}';
      let error = await shouldThrow(async () => Wallet.import(config));
      assert.equal(error.vcxCode, VCXCode.IO_ERROR);
      shutdownVcx(false);

      config = '{"wallet_key":"","exported_wallet_path":"","backup_key":""}';
      error = await shouldThrow(async () => Wallet.import(config));
      assert.equal(error.vcxCode, VCXCode.INVALID_JSON);
      shutdownVcx(false);

      config = '{"wallet_name":"","exported_wallet_path":"","backup_key":""}';
      error = await shouldThrow(async () => Wallet.import(config));
      assert.equal(error.vcxCode, VCXCode.INVALID_JSON);
      shutdownVcx(false);

      config = '{"wallet_name":"","wallet_key":"","backup_key":""}';
      error = await shouldThrow(async () => Wallet.import(config));
      assert.equal(error.vcxCode, VCXCode.INVALID_JSON);
      shutdownVcx(false);

      config = '{"wallet_name":"","wallet_key":"","exported_wallet_path":""}';
      error = await shouldThrow(async () => Wallet.import(config));
      assert.equal(error.vcxCode, VCXCode.INVALID_JSON);
    });
  });

  describe('export:', () => {
    it('throws: libindy error', async () => {
      const error = await shouldThrow(async () =>
        Wallet.export('/tmp/foobar.wallet', 'key_for_wallet'),
      );
      assert.equal(error.vcxCode, VCXCode.INVALID_WALLET_HANDLE);
    });
  });
});
