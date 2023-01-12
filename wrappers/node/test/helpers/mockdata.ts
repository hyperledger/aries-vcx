export const ARIES_PING = {
  '@type': 'https://didcomm.org/trust_ping/1.0/ping',
  '@id': '518be002-de8e-456e-b3d5-8fe472477a86',
  '~timing': {
    out_time: '2018-12-15 04:29:23Z',
    expires_time: '2018-12-15 05:29:23Z',
    delay_milli: 0,
  },
  comment: 'Hi. Are you listening?',
  response_requested: true,
};

export const ARIES_PING_RESPONSE = {
  '@type': 'https://didcomm.org/trust_ping/1.0/ping_response',
  '@id': 'e002518b-456e-b3d5-de8e-7a86fe472847',
  '~thread': { thid: '518be002-de8e-456e-b3d5-8fe472477a86' },
  '~timing': { in_time: '2018-12-15 04:29:28Z', out_time: '2018-12-15 04:31:00Z' },
  comment: "Hi yourself. I'm here.",
};

export const ARIES_UNKNOWN_TYPE = {
  '@type': 'https://didcomm.org/foo/5.0/bar',
  '@id': 'e002518b-456e-b3d5-de8e-7a86fe472847',
  '~thread': { thid: '518be002-de8e-456e-b3d5-8fe472477a86' },
  '~timing': { in_time: '2018-12-15 04:29:28Z', out_time: '2018-12-15 04:31:00Z' },
};

export const ARIES_CONNECTION_ACK = {
  '@id': '680e90b0-4a01-4dc7-8a1d-e54b43ebcc28',
  '@type': 'https://didcomm.org/notification/1.0/ack',
  status: 'OK',
  '~thread': {
    received_orders: {},
    sender_order: 0,
    thid: 'b5517062-303f-4267-9a29-09bc89497c06',
  },
};

export const ARIES_CONNECTION_REQUEST = {
  '@id': 'b5517062-303f-4267-9a29-09bc89497c06',
  '@type': 'did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/1.0/request',
  connection: {
    DID: '2RjtVytftf9Psbh3E8jqyq',
    DIDDoc: {
      '@context': 'https://w3id.org/did/v1',
      authentication: [
        {
          publicKey: '2RjtVytftf9Psbh3E8jqyq#1',
          type: 'Ed25519SignatureAuthentication2018',
        },
      ],
      id: '2RjtVytftf9Psbh3E8jqyq',
      publicKey: [
        {
          controller: '2RjtVytftf9Psbh3E8jqyq',
          id: '1',
          publicKeyBase58: 'n6ZJrPGhbkLxQBxH11BvQHSKch58sx3MAqDTkUG4GmK',
          type: 'Ed25519VerificationKey2018',
        },
      ],
      service: [
        {
          id: 'did:example:123456789abcdefghi;indy',
          priority: 0,
          recipientKeys: ['2RjtVytftf9Psbh3E8jqyq#1'],
          routingKeys: [
            'AKnC8qR9xsZZEBY7mdV6fzjmmtKxeegrNatpz4jSJhrH',
            'Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR',
          ],
          serviceEndpoint: 'http://localhost:8080/agency/msg',
          type: 'IndyAgent',
        },
      ],
    },
  },
  label: 'alice-157ea14b-4b7c-48a5-b536-d4ed6e027b84',
};
