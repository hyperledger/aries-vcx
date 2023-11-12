import '../module-resolver-helper';

import { assert } from 'chai';
import { initVcx, shouldThrow } from 'helpers/utils';
import {
  getLedgerAuthorAgreement,
  getVersion,
  provisionCloudAgent,
  setActiveTxnAuthorAgreementMeta,
  VCXCode,
} from 'src';

describe('utils:', () => {
  before(() => initVcx());

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

  describe('setActiveTxnAuthorAgreementMeta:', () => {
    it('success', async () => {
      setActiveTxnAuthorAgreementMeta(
        'indy agreement',
        '1.0.0',
        'acceptance type 1',
      );
    });
  });
  // 
  // describe('getLedgerAuthorAgreement:', () => {
  //   it('success', async () => {
  //     const agreement = await getLedgerAuthorAgreement();
  //     assert.equal(
  //       agreement,
  //       '{"text":"Default indy agreement", "version":"1.0.0", "aml": {"acceptance mechanism label1": "description"}}',
  //     );
  //   });
  // });
});
