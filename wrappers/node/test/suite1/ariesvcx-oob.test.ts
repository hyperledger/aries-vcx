import '../module-resolver-helper';

import { initVcxTestMode } from 'helpers/utils'
import { GoalCode, OutOfBandSender } from 'src'
import { assert } from 'chai';

const credentialOffer = {
  "@id": "57b3f85d-7673-4e6f-bb09-cc27cf2653c0",
  "@type": "https://didcomm.org/issue-credential/1.0/offer-credential",
  "credential_preview": {
    "@type": "https://didcomm.org/issue-credential/1.0/credential-preview",
    "attributes": [
      {
        "name": "age",
        "value": "25"
      },
    ]
  },
  "offers~attach": [
    {
      "@id": "libindy-cred-offer-0",
      "data": {
        "base64": "eyJzY2hlzU0NzA3NTkwOTc1MTUyNTk4MTgwNyJ9"
      },
      "mime-type": "application/json"
    }
  ]
}

describe('Connection:', () => {
  before(() => initVcxTestMode())

  describe('create:', () => {
    it('success', async () => {
      const oobSender = await OutOfBandSender.create({label: "foo", goalCode: GoalCode.P2PMessaging, goal: "bar"})
      await oobSender.appendServiceDid("VsKV7grR1BUE29mG2Fm2kX")
      const service = {
        "id": "did:example:123456789abcdefghi;indy",
        "priority": 0,
        "recipientKeys": ["abcde"],
        "routingKeys": ["12345"],
        "serviceEndpoint": "http://example.org/agent",
        "type": "IndyAgent"
      }
      await oobSender.appendService(JSON.stringify(service))
      await oobSender.appendMessage(JSON.stringify(credentialOffer))
      const msg = JSON.parse(await oobSender.toMessage())
      assert.equal(msg["@type"], "https://didcomm.org/out-of-band/1.0/invitation")
      assert.equal(msg["goal"], "bar")
      assert.equal(msg["label"], "foo")
    })
  })
})
