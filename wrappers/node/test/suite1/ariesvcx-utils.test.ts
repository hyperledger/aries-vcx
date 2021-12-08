import '../module-resolver-helper';

import { assert } from 'chai';
import { initVcxTestMode, shouldThrow } from 'helpers/utils';
import {
  downloadMessagesV2,
  endorseTransaction,
  getLedgerAuthorAgreement,
  getVersion,
  provisionCloudAgent,
  setActiveTxnAuthorAgreementMeta,
  updateMessages,
  VCXCode,
} from 'src';
import { errorMessage } from '../../src/utils/error-message';

describe('utils:', () => {
  before(() => initVcxTestMode());

  const downloadMessagesData = {
    pairwiseDids: 'asdf',
    status: 'MS-104',
    uids: 'asdf',
  };
  const updateMessagesData = {
    msgJson: '[{"pairwiseDID":"QSrw8hebcvQxiwBETmAaRs","uids":["mgrmngq"]}]',
  };

  describe('provisionAgent:', () => {
    it('success', async () => {
      const provisionConfig = {
        agency_did: 'Ab8TvZa3Q19VNkQVzAWVL7',
        agency_endpoint: 'https://vcx.agency.example.org',
        agency_verkey: '5LXaR43B1aQyeh94VBP8LG1Sgvjk7aNfqiksBCSjwqbf',
        // agent_seed
      };
      const res = await provisionCloudAgent(provisionConfig);
      assert.ok(res);
    });

    it('throws: invalid input', async () => {
      const error = await shouldThrow(() => provisionCloudAgent({}));
      assert.equal(error.vcxCode, VCXCode.INVALID_CONFIGURATION);
    });
  });

  describe('getVersion:', () => {
    it('success', async () => {
      const version = getVersion();
      assert.ok(version);
    });
  });

  describe('VCXCode:', () => {
    it('should have a one-to-one mapping for each code', async () => {
      let max = 0;
      for (const ec in VCXCode) {
        if (Number(VCXCode[ec]) > max) {
          max = Number(VCXCode[ec]);
        }
      }
      assert.equal(errorMessage(max + 1), errorMessage(1001));
    });
  });

  describe('setActiveTxnAuthorAgreementMeta:', () => {
    it('success', async () => {
      setActiveTxnAuthorAgreementMeta(
        'indy agreement',
        '1.0.0',
        undefined,
        'acceptance type 1',
        123456789,
      );
    });
  });

  describe('getLedgerAuthorAgreement:', () => {
    it('success', async () => {
      const agreement = await getLedgerAuthorAgreement();
      assert.equal(
        agreement,
        '{"text":"Default indy agreement", "version":"1.0.0", "aml": {"acceptance mechanism label1": "description"}}',
      );
    });
  });

  describe('endorseTransaction:', () => {
    it('success', async () => {
      const transaction =
        '{"req_id":1, "identifier": "EbP4aYNeTHL6q385GuVpRV", "signature": "gkVDhwe2", "endorser": "NcYxiDXkpYi6ov5FcYDi1e"}';
      await endorseTransaction(transaction);
    });
  });
});
