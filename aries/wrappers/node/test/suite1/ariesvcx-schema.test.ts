import '../module-resolver-helper';

import { assert } from 'chai';
import { dataSchemaCreate, schemaCreate } from 'helpers/entities';
import { initVcx, shouldThrow } from 'helpers/utils';
import { Schema, SchemaState, VCXCode } from 'src';

describe('Schema:', () => {
  before(() => initVcx());

  describe('create:', () => {
    it('success', async () => {
      const schema = await schemaCreate();
      assert.equal(await schema.getState(), SchemaState.Published);
    });

    it('throws: missing sourceId', async () => {
      const { sourceId, ...data } = dataSchemaCreate();
      const error = await shouldThrow(() => Schema.create(data as any));
      assert.equal(error.napiCode, 'StringExpected');
    });

    it('throws: incomplete data', async () => {
      const { data, ...rest } = dataSchemaCreate();
      const error = await shouldThrow(() => Schema.create({ data: {} as any, ...rest }));
      assert.equal(error.napiCode, 'StringExpected');
    });

    it('throws: missing data.name', async () => {
      const {
        data: { name, ...dataRest },
        ...rest
      } = dataSchemaCreate();
      const error = await shouldThrow(() => Schema.create({ data: dataRest, ...rest } as any));
      assert.equal(error.napiCode, 'StringExpected');
    });

    it('throws: missing data.version', async () => {
      const {
        data: { version, ...dataRest },
        ...rest
      } = dataSchemaCreate();
      const error = await shouldThrow(() => Schema.create({ data: dataRest, ...rest } as any));
      assert.equal(error.napiCode, 'StringExpected');
    });

    it('throws: missing data.attrNames', async () => {
      const {
        data: { attrNames, ...dataRest },
        ...rest
      } = dataSchemaCreate();
      const error = await shouldThrow(() => Schema.create({ data: dataRest, ...rest } as any));
      assert.equal(error.napiCode, 'StringExpected');
    });

    it('throws: invalid data', async () => {
      const { data, ...rest } = dataSchemaCreate();
      const error = await shouldThrow(() =>
        Schema.create({
          data: {
            attrNames: 'foobar',
            name: 'Schema',
            version: '1.0.0',
          } as any,
          ...rest,
        }),
      );
      assert.equal(error.napiCode, 'GenericFailure');
      assert.equal(error.vcxCode, VCXCode.SERIALIZATION_ERROR);
    });
  });

  describe('serialize:', () => {
    it('success', async () => {
      const schema = await schemaCreate();
      const serialized = await schema.serialize();
      assert.ok(serialized);
      assert.property(serialized, 'version');
      assert.property(serialized, 'data');
      const { data, version } = serialized;
      assert.ok(data);
      assert.ok(version);
    });

    it('throws: not initialized', async () => {
      const schema = new Schema({} as any);
      const error = await shouldThrow(() => schema.serialize());
      assert.equal(error.napiCode, 'NumberExpected');
    });
  });

  describe('deserialize:', () => {
    it('success', async () => {
      const schema1 = await schemaCreate();
      const data1 = await schema1.serialize();
      const schema2 = await Schema.deserialize(data1);
      const data2 = await schema2.serialize();
      assert.deepEqual(data1, data2);
    });

    it('throws: incorrect data', async () => {
      const error = await shouldThrow(async () =>
        Schema.deserialize({ data: { source_id: 'Invalid' } } as any),
      );
      assert.equal(error.vcxCode, VCXCode.INVALID_JSON);
    });
  });
});
