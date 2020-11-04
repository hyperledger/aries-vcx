import '../module-resolver-helper'

import { assert } from 'chai'
import { initVcxTestMode, shouldThrow } from 'helpers/utils'
import {
  downloadMessages,
  endorseTransaction,
  getLedgerAuthorAgreement,
  getLedgerFees,
  getVersion,
  provisionAgent,
  setActiveTxnAuthorAgreementMeta,
  updateInstitutionConfigs,
  updateMessages,
  VCXCode
} from 'src'
import { errorMessage } from '../../src/utils/error-message'

describe('utils:', () => {
  before(() => initVcxTestMode())

  // tslint:disable-next-line max-line-length
  const updateInstitutionConfigsData = {
    logoUrl: 'https://google.com',
    name: 'New Name'
  }
  const downloadMessagesData = {
    pairwiseDids: 'asdf',
    status: 'MS-104',
    uids: 'asdf'
  }
  const updateMessagesData = {
    msgJson: '[{"pairwiseDID":"QSrw8hebcvQxiwBETmAaRs","uids":["mgrmngq"]}]'
  }

  describe('provisionAgent:', () => {
    it('success', async () => {
      const provisionConfig = JSON.stringify({
        agency_did: 'Ab8TvZa3Q19VNkQVzAWVL7',
        agency_url: 'https://vcx.agency.example.org',
        agency_verkey: '5LXaR43B1aQyeh94VBP8LG1Sgvjk7aNfqiksBCSjwqbf',
        wallet_key: '123',
        wallet_name: 'test_provision_agent'
      })
      const res = await provisionAgent(provisionConfig)
      assert.ok(res)
    })

    it('throws: invalid input', async () => {
      const error = await shouldThrow(() => provisionAgent(''))
      assert.equal(error.vcxCode, VCXCode.INVALID_OPTION)
    })
  })

  describe('getVersion:', () => {
    it('success', async () => {
      const version = getVersion()
      assert.ok(version)
    })
  })

  describe('updateInstitutionConfigs:', () => {
    it('success', async () => {
      const res = updateInstitutionConfigs(updateInstitutionConfigsData)
      assert.equal(res, 0)
    })

    it('throws: missing name', async () => {
      const { name, ...data } = updateInstitutionConfigsData
      const error = await shouldThrow(() => updateInstitutionConfigs(data as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_CONFIGURATION)
    })

    it('throws: missing logoUrl', async () => {
      const { logoUrl, ...data } = updateInstitutionConfigsData
      const error = await shouldThrow(() => updateInstitutionConfigs(data as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_CONFIGURATION)
    })
  })

  describe('getLedgerFees:', () => {
    it('success', async () => {
      const fees = await getLedgerFees()
      assert.ok(fees)
    })
  })

  describe('downloadMessages:', () => {
    it.skip('success', async () => {
      const messages = await downloadMessages(downloadMessagesData)
      assert.ok(messages)
    })
  })

  describe('updateMessages:', () => {
    it.skip('success', async () => {
      await updateMessages(updateMessagesData)
    })
  })

  describe('VCXCode:', () => {
    it('should have a one-to-one mapping for each code', async () => {
      let max: number = 0
      for (const ec in VCXCode) {
        if (Number(VCXCode[ec]) > max) {
          max = Number(VCXCode[ec])
        }
      }
      assert.equal(errorMessage(max + 1), errorMessage(1001))
    })
  })

  describe('setActiveTxnAuthorAgreementMeta:', () => {
    it('success', async () => {
      setActiveTxnAuthorAgreementMeta('indy agreement', '1.0.0', undefined, 'acceptance type 1', 123456789)
    })
  })

  describe('getLedgerAuthorAgreement:', () => {
    it('success', async () => {
      const agreement = await getLedgerAuthorAgreement()
      assert.equal(agreement, '{"text":"Default indy agreement", "version":"1.0.0", "aml": {"acceptance mechanism label1": "description"}}')
    })
  })

  describe('endorseTransaction:', () => {
    it('success', async () => {
      const transaction = '{"req_id":1, "identifier": "EbP4aYNeTHL6q385GuVpRV", "signature": "gkVDhwe2", "endorser": "NcYxiDXkpYi6ov5FcYDi1e"}'
      await endorseTransaction(transaction)
    })
  })
})
