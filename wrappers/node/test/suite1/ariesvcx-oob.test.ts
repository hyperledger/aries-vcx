import '../module-resolver-helper';

import { initVcxTestMode } from 'helpers/utils';
import { GoalCode, OutOfBandSender, OutOfBandReceiver, HandshakeProtocol } from 'src';
import { assert } from 'chai';

const credentialOffer = {
  '@id': '57b3f85d-7673-4e6f-bb09-cc27cf2653c0',
  '@type': 'https://didcomm.org/issue-credential/1.0/offer-credential',
  credential_preview: {
    '@type': 'https://didcomm.org/issue-credential/1.0/credential-preview',
    attributes: [
      {
        name: 'age',
        value: '25',
      },
    ],
  },
  'offers~attach': [
    {
      '@id': 'libindy-cred-offer-0',
      data: {
        base64: 'eyJzY2hlzU0NzA3NTkwOTc1MTUyNTk4MTgwNyJ9',
      },
      'mime-type': 'application/json',
    },
  ],
};

describe('Out of Band:', () => {
  before(() => initVcxTestMode());

  describe('create:', () => {
    it('success', async () => {
      const oobSender = await OutOfBandSender.create({
        source_id: 'abcd',
        label: 'foo',
        goalCode: GoalCode.P2PMessaging,
        goal: 'bar',
        handshake_protocols: [HandshakeProtocol.ConnectionV1],
      });
      oobSender.appendServiceDid('VsKV7grR1BUE29mG2Fm2kX');
      const service = {
        id: 'did:example:123456789abcdefghi;indy',
        priority: 0,
        recipientKeys: ['abcde'],
        routingKeys: ['12345'],
        serviceEndpoint: 'http://example.org/agent',
        type: 'IndyAgent',
      };
      oobSender.appendService(JSON.stringify(service));
      oobSender.appendMessage(JSON.stringify(credentialOffer));
      const msg = JSON.parse(oobSender.toMessage());
      assert.equal(msg['@type'], 'https://didcomm.org/out-of-band/1.1/invitation');
      assert.equal(msg['goal'], 'bar');
      assert.equal(msg['label'], 'foo');
      assert.equal(msg['handshake_protocols'][0], 'https://didcomm.org/connections/1.0');
    });
  });

  describe('sender serde:', () => {
    it('success', async () => {
      const oobSender = await OutOfBandSender.create({ source_id: 'abcd' });
      oobSender.appendServiceDid('VsKV7grR1BUE29mG2Fm2kX');
      const serialized = oobSender.serialize();
      OutOfBandSender.deserialize(serialized);
    });
  });

  describe('receiver serde:', () => {
    it('success', async () => {
      const oobSender = await OutOfBandSender.create({ source_id: 'abcd' });
      const oobReceiver = OutOfBandReceiver.createWithMessage(oobSender.toMessage());
      const serialized = oobReceiver.serialize();
      OutOfBandReceiver.deserialize(serialized);
    });
  });
});
