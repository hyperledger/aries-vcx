import '../module-resolver-helper';

import { assert } from 'chai';
import {
  createConnectionInviterInvited,
  createConnectionInviterRequested,
  dataProofCreateLegacy, dataProofCreate,
  proofCreate,
} from 'helpers/entities';
import { initVcx, shouldThrow } from 'helpers/utils';
import { Proof, VerifierStateType, VCXCode } from 'src';

describe('Proof:', () => {
  before(() => initVcx());

  describe('create:', () => {
    it('lgeacy success', async () => {
      await proofCreate(dataProofCreateLegacy());
    });

    it('success', async () => {
      await proofCreate(dataProofCreate());
    });

    it('throws: missing sourceId', async () => {
      const { sourceId, ...data } = dataProofCreate();
      const error = await shouldThrow(() => Proof.create(data as any));
      assert.equal(error.napiCode, 'StringExpected');
    });

    it('throws: missing attrs', async () => {
      const { attrs, ...data } = dataProofCreate();
      const error = await shouldThrow(() => Proof.create({ ...data } as any));
      assert.equal(error.napiCode, 'StringExpected');
    });

    it('throws: missing name', async () => {
      const { name, ...data } = dataProofCreate();
      const error = await shouldThrow(() => Proof.create(data as any));
      assert.equal(error.napiCode, 'StringExpected');
    });
  });

  describe('serialize:', () => {
    it('success', async () => {
      const proof = await proofCreate(dataProofCreateLegacy());
      const { data } = await proof.serialize();
      assert.ok(data);
    });
  });

  describe('deserialize:', () => {
    it('success', async () => {
      const proof1 = await proofCreate(dataProofCreate());
      const data1 = await proof1.serialize();
      const proof2 = await Proof.deserialize(data1 as any);
      const data2 = await proof2.serialize();
      assert.deepEqual(data1, data2);
    });

    it('throws: incorrect data', async () => {
      const error = await shouldThrow(() =>
        Proof.deserialize({ source_id: 'xyz', foo: 'bar' } as any),
      );
      assert.equal(error.napiCode, 'GenericFailure');
      assert.equal(error.vcxCode, VCXCode.INVALID_JSON);
    });

    it('throws: incomplete data', async () => {
      const error = await shouldThrow(async () =>
        Proof.deserialize({
          name: 'Invalid',
          requested_attrs: 'Invalid',
          source_id: 'Invalid',
        } as any),
      );
      assert.equal(error.napiCode, 'GenericFailure');
      assert.equal(error.vcxCode, VCXCode.INVALID_JSON);
    });
  });

  describe('updateState:', () => {
    it(`throws error when not initialized`, async () => {
      let caught_error;
      const proof = new Proof();
      const connection = await createConnectionInviterRequested();
      try {
        await proof.updateStateV2(connection);
      } catch (err) {
        caught_error = err;
      }
      assert.isNotNull(caught_error);
    });

    it('build presentation request and mark as sent', async () => {
      const proof = await proofCreate(dataProofCreateLegacy());
      assert.equal(await proof.getState(), VerifierStateType.PresentationRequestSet);
      await proof.markPresentationRequestMsgSent();
      assert.equal(await proof.getState(), VerifierStateType.PresentationRequestSent);
    });
  });

  describe('requestProof:', () => {
    it('success', async () => {
      const connection = await createConnectionInviterRequested();
      const proof = await proofCreate(dataProofCreateLegacy());
      await proof.requestProof(connection);
      assert.equal(await proof.getState(), VerifierStateType.PresentationRequestSent);
    });

    it('successfully get request message', async () => {
      const proof = await proofCreate(dataProofCreateLegacy());
      const msg = await proof.getProofRequestMessage();
      assert(msg);
    });

    it('throws: not initialized', async () => {
      const connection = await createConnectionInviterRequested();
      const proof = new Proof();
      const error = await shouldThrow(() => proof.requestProof(connection));
      assert.equal(error.napiCode, 'NumberExpected');
    });

    it('throws: connection not initialized', async () => {
      const connection = await createConnectionInviterInvited();
      const proof = await proofCreate(dataProofCreateLegacy());
      const error = await shouldThrow(() => proof.requestProof(connection));
      assert.equal(error.napiCode, 'GenericFailure');
      assert.equal(error.vcxCode, VCXCode.NOT_READY);
    });
  });
});
