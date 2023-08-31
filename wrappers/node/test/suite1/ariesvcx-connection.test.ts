import '../module-resolver-helper';

import { assert, expect } from 'chai';
import { initVcxTestMode } from 'helpers/utils';
import {ConnectionStateType } from 'src';
import { NonmediatedConnection } from 'src';

describe('Nonmediated connection:', () => {
  before(() => initVcxTestMode());

  describe('create invitation:', () => {
    it('success', async () => {
      const serviceEndpoint = 'http://localhost:8080/';
      const routingKeys = [ 'routingKey' ];
      const connection = await NonmediatedConnection.createInviter();
      assert.equal(connection.getState(), ConnectionStateType.Initial);

      await connection.createInvite({ serviceEndpoint, routingKeys });
      assert.equal(connection.getState(), ConnectionStateType.Invited);

      const invite = JSON.parse(connection.getInvitation());
      expect(invite.routingKeys).deep.equal(routingKeys);
      assert.equal(invite.serviceEndpoint, serviceEndpoint);
    });
  });

  describe('serialize / deserialize:', () => {
    it('success', async () => {
      const connection = await NonmediatedConnection.createInviter();
      assert.equal(connection.getState(), ConnectionStateType.Initial);

      const serialized = connection.serialize()
      const deserialized = NonmediatedConnection.deserialize(serialized);
      expect(deserialized.serialize()).deep.equal(serialized);
    });
  });
})
