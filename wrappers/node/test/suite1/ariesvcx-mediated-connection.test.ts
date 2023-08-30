import '../module-resolver-helper';

import { assert } from 'chai';
import {
  connectionCreateInviterNull,
  createConnectionInviterFinished,
  createConnectionInviterInvited,
  createConnectionInviterRequested,
  dataConnectionCreate,
} from 'helpers/entities';
import { initVcxTestMode, shouldThrow } from 'helpers/utils';
import { Connection, MediatedConnectionStateType, VCXCode } from 'src';
import { ARIES_PING, ARIES_PING_RESPONSE } from '../helpers/mockdata';

describe('Connection:', () => {
  before(() => initVcxTestMode());

  describe('create:', () => {
    it('success', async () => {
      await connectionCreateInviterNull();
    });

    it('success: parallel', async () => {
      const numConnections = 50;
      const data = dataConnectionCreate();
      await Promise.all(
        new Array(numConnections).fill(0).map(() => connectionCreateInviterNull(data)),
      );
    });
  });

  describe('connect:', () => {
    it('success', async () => {
      const connection = await connectionCreateInviterNull();
      await connection.connect();
    });

    it('throws: not initialized', async () => {
      const connection = new (Connection as any)();
      const err = await shouldThrow(async () => connection.connect({ data: '{}' }));
      // NAPI throws error - connection handle is undefined instead of a number
      assert.equal(err.message, `Failed to convert napi value Undefined into rust type \`u32\``);
    });
  });

  describe('sendMessage:', () => {
    it('success: sends message', async () => {
      const connection = await connectionCreateInviterNull();
      await connection.connect();
      const error = await shouldThrow(() =>
        connection.sendMessage({ msg: 'msg', type: 'msg', title: 'title' }),
      );
      assert.equal(error.vcxCode, VCXCode.NOT_READY);
    });
  });

  describe('signData:', () => {
    it('success: signs data', async () => {
      const connection = await connectionCreateInviterNull();
      await connection.connect();
      const signature = await connection.signData(Buffer.from('random string'));
      assert(signature);
    });
  });

  describe('verifySignature', () => {
    it('success: verifies the signature', async () => {
      const connection = await createConnectionInviterRequested();
      const valid = await connection.verifySignature({
        data: Buffer.from('random string'),
        signature: Buffer.from('random string'),
      });
      assert(valid);
    });
  });

  describe('serialize:', () => {
    it('success', async () => {
      const connection = await connectionCreateInviterNull();
      const serialized = connection.serialize();
      assert.ok(serialized);
      assert.property(serialized, 'version');
      assert.property(serialized, 'data');
      assert.property(serialized, 'state');
      assert.property(serialized, 'source_id');
      assert.property(serialized.data, 'pw_did');
      assert.property(serialized.data, 'pw_vk');
      assert.property(serialized.data, 'agent_did');
      assert.property(serialized.data, 'agent_vk');
      const { data, version, source_id } = serialized;
      assert.ok(data);
      assert.ok(version);
      assert.ok(source_id);
    });
  });

  describe('deserialize:', () => {
    it('success', async () => {
      const connection1 = await connectionCreateInviterNull();
      const data1 = connection1.serialize();
      const connection2 = Connection.deserialize(data1);
      const data2 = connection2.serialize();
      assert.deepEqual(data1, data2);
    });

    it('throws: incorrect data', async () => {
      const error = await shouldThrow(() =>
        Connection.deserialize({ data: { source_id: 'Invalid' } } as any),
      );
      assert.equal(error.vcxCode, VCXCode.INVALID_JSON);
    });
  });

  describe('updateState:', () => {
    it('throws error when not initialized', async () => {
      let caught_error;
      const connection = new (Connection as any)();
      try {
        await connection.updateState();
      } catch (err) {
        caught_error = err;
      }
      assert.isNotNull(caught_error);
    });

    it(`returns ${MediatedConnectionStateType.Initial}: not connected`, async () => {
      const connection = await connectionCreateInviterNull({ id: 'alice' });
      await connection.updateState();
      assert.equal(await connection.getState(), MediatedConnectionStateType.Initial);
    });

    it(`returns ${MediatedConnectionStateType.Finished}: mocked accepted`, async () => {
      const connection = await createConnectionInviterFinished();
      assert.equal(await connection.getState(), MediatedConnectionStateType.Finished);
    });
  });

  describe('inviteDetails:', () => {
    it('success', async () => {
      const connection = await createConnectionInviterInvited();
      const details = connection.inviteDetails();
      const parsedInvitation = JSON.parse(details);
      assert.isString(parsedInvitation['@id']);
      assert.equal(parsedInvitation['@type'], 'https://didcomm.org/connections/1.0/invitation');
      assert.isString(parsedInvitation.label);
      assert.isArray(parsedInvitation.recipientKeys);
      assert.equal(parsedInvitation.recipientKeys.length, 1);
      assert.isArray(parsedInvitation.routingKeys);
      assert.equal(parsedInvitation.routingKeys.length, 2);
      assert.equal(parsedInvitation.serviceEndpoint, 'http://127.0.0.1:8080/agency/msg');
    });
  });

  describe('trustping:', () => {
    it('should handle ping message', async () => {
      const connection = await createConnectionInviterFinished();
      await connection.handleMessage(JSON.stringify(ARIES_PING));
    });

    it('should handle ping response message', async () => {
      const connection = await createConnectionInviterFinished();
      await connection.handleMessage(JSON.stringify(ARIES_PING_RESPONSE));
    });

    it('should send ping message', async () => {
      const connection = await createConnectionInviterFinished();
      await connection.sendPing('ping');
    });
  });

  describe('sendDiscoveryFeatures:', () => {
    it('success: send discovery features', async () => {
      const connection = await createConnectionInviterFinished();
      await connection.sendDiscoveryFeatures('*', 'comment');
    });
  });
});
