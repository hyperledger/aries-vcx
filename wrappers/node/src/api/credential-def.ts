import * as ffi from '@hyperledger/vcx-napi-rs';
import { ISerializedData } from './common';
import { VcxBase } from './vcx-base';
import { VCXInternalError } from '../errors';

export interface ICredentialDefCreateDataV2 {
  sourceId: string;
  schemaId: string;
  supportRevocation: boolean;
  tag: string;
}

export interface ICredentialDefData {
  source_id: string;
  handle: number;
  name: string;
  credential_def: ICredentialDefDataObj;
}

export interface ICredentialDefDataObj {
  ref: number;
  origin: string;
  signature_type: string;
  data: any; // eslint-disable-line @typescript-eslint/no-explicit-any
}

export interface ICredentialDefParams {
  schemaId?: string;
  name?: string;
  credDefId?: string;
  tailsDir?: string;
}

export interface IRevocationDetails {
  maxCreds?: number;
  supportRevocation?: boolean;
  tailsDir?: string;
}

export enum CredentialDefState {
  Built = 0,
  Published = 1,
}

export class CredentialDef extends VcxBase<ICredentialDefData> {
  public static async create({
    supportRevocation,
    schemaId,
    sourceId,
    tag,
  }: ICredentialDefCreateDataV2): Promise<CredentialDef> {
    try {
      const credentialDef = new CredentialDef(sourceId, { schemaId });
      const handle = await ffi.credentialdefCreateV2(sourceId, schemaId, tag, supportRevocation);
      credentialDef._setHandle(handle);
      return credentialDef;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public static deserialize(
    credentialDef: ISerializedData<ICredentialDefData>,
  ): CredentialDef {
    const {
      data: { name },
    } = credentialDef;
    const credentialDefParams = {
      name,
      schemaId: null,
    };
    return super._deserialize(CredentialDef, credentialDef, credentialDefParams);
  }

  protected _serializeFn = ffi.credentialdefSerialize;
  protected _deserializeFn = ffi.credentialdefDeserialize;
  protected _releaseFn = ffi.credentialdefRelease;
  private _name: string | undefined;
  private _schemaId: string | undefined;
  private _credDefId: string | undefined;
  private _tailsDir: string | undefined;

  constructor(sourceId: string, { name, schemaId, credDefId, tailsDir }: ICredentialDefParams) {
    super(sourceId);
    this._name = name;
    this._schemaId = schemaId;
    this._credDefId = credDefId;
    this._tailsDir = tailsDir;
  }

  public releaseRustData(): void {
    try {
      this._releaseFn(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async publish(): Promise<void> {
    try {
      await ffi.credentialdefPublish(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getCredDefId(): string {
    try {
      return ffi.credentialdefGetCredDefId(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async updateState(): Promise<CredentialDefState> {
    try {
      return await ffi.credentialdefUpdateState(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getState(): CredentialDefState {
    try {
      return ffi.credentialdefGetState(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  get name(): string | undefined {
    return this._name;
  }

  get schemaId(): string | undefined {
    return this._schemaId;
  }

  get credDefId(): string | undefined {
    return this._credDefId;
  }

  get tailsDir(): string | undefined {
    return this._tailsDir;
  }

  protected _setHandle(handle: number): void {
    super._setHandle(handle);
  }
}
