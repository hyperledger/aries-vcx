import * as ffi from '@hyperledger/vcx-napi-rs';
import { ISerializedData } from './common';
import { VcxBase } from './vcx-base';
import { VCXInternalError } from '../errors';

/**
 * @interface Interface that represents the parameters for `Schema.create` function.
 * @description
 */
export interface ISchemaCreateData {
  sourceId: string;
  data: ISchemaAttrs;
}

/**
 * @interface Interface that represents the parameters for `Schema.prepareForEndorser` function.
 * @description
 */
export interface ISchemaPrepareForEndorserData {
  sourceId: string;
  data: ISchemaAttrs;
  // DID of the Endorser that will submit the transaction.
  endorser: string;
}

/**
 * @interface
 * @description
 * name: name of schema
 * version: version of the scheme
 * attrNames: a list of named attribtes inteded to be added to the schema
 * (the number of attributes should be less or equal than 125)
 */
export interface ISchemaAttrs {
  name: string;
  version: string;
  attrNames: string[];
}

export interface ISchemaSerializedData {
  source_id: string;
  handle: string;
  name: string;
  version: string;
  data: string[];
  schema_id: string;
}

export interface ISchemaParams {
  schemaId: string;
  name: string;
  schemaAttrs: ISchemaAttrs;
}

export enum SchemaState {
  Built = 0,
  Published = 1,
}

export class Schema extends VcxBase<ISchemaSerializedData> {
  get schemaAttrs(): ISchemaAttrs {
    return this._schemaAttrs;
  }

  get schemaId(): string {
    return this._schemaId;
  }

  get name(): string {
    return this._name;
  }

  public static async create({ data, sourceId }: ISchemaCreateData, issuerDid: string): Promise<Schema> {
    try {
      const schema = new Schema({ name: data.name, schemaId: '', schemaAttrs: data });
      const handle = await ffi.schemaCreate(
        issuerDid,
        sourceId,
        schema._name,
        data.version,
        JSON.stringify(data.attrNames),
      );
      schema._setHandle(handle);
      schema._schemaId = ffi.schemaGetSchemaId(handle);
      return schema;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public static deserialize(schema: ISerializedData<ISchemaSerializedData>): Schema {
    const {
      data: { name, schema_id, version, data },
    } = schema;
    const jsConstructorParams = {
      name,
      schemaAttrs: { name, version, attrNames: data },
      schemaId: schema_id,
    };
    return super._deserialize(Schema, schema as any, jsConstructorParams);
  }

  protected _serializeFn = ffi.schemaSerialize;
  protected _deserializeFn = ffi.schemaDeserialize;
  protected _releaseFn = ffi.schemaRelease;
  protected _name: string;
  protected _schemaId: string;
  protected _schemaAttrs: ISchemaAttrs;

  constructor({ name, schemaId, schemaAttrs }: ISchemaParams) {
    super();
    this._name = name;
    this._schemaId = schemaId;
    this._schemaAttrs = schemaAttrs;
  }

  public getState(): SchemaState {
    try {
      return ffi.schemaGetState(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  protected getSchemaId(): string {
    try {
      return ffi.schemaGetSchemaId(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  protected _setHandle(handle: number): void {
    super._setHandle(handle);
  }
}
