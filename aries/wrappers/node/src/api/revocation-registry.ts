import * as ffi from '@hyperledger/vcx-napi-rs';
import { ISerializedData } from './common';
import { VcxBase } from './vcx-base';
import { VCXInternalError } from '../errors';

export interface IRevocationRegistryData {
  source_id: string;
  cred_def_id: string;
  issuer_did: string;
  rev_reg_id: string;
  rev_reg_def: string;
  rev_reg_entry: string;
  tails_dir: string;
  max_creds: number;
  tag: number;
  rev_reg_def_state: string;
  rev_reg_delta_state: string;
}

export interface IRevocationRegistryConfig {
  issuerDid: string;
  credDefId: string;
  tag: number;
  tailsDir: string;
  maxCreds: number;
}

export class RevocationRegistry extends VcxBase<IRevocationRegistryData> {
  public static async create(config: IRevocationRegistryConfig): Promise<RevocationRegistry> {
    try {
      const revReg = new RevocationRegistry();
      const _config = {
        issuer_did: config.issuerDid,
        cred_def_id: config.credDefId,
        tag: config.tag,
        tails_dir: config.tailsDir,
        max_creds: config.maxCreds,
      };
      const handle = await ffi.revocationRegistryCreate(JSON.stringify(_config));
      revReg._setHandle(handle);
      return revReg;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public static deserialize(
    data: ISerializedData<IRevocationRegistryData>,
  ): RevocationRegistry {
    const newObj = { ...data, source_id: 'foo' };
    return super._deserialize(RevocationRegistry, newObj);
  }

  protected _serializeFn = ffi.revocationRegistrySerialize;
  protected _deserializeFn = ffi.revocationRegistryDeserialize;
  protected _releaseFn = ffi.revocationRegistryRelease;

  public async publish(tailsUrl: string): Promise<void> {
    try {
      await ffi.revocationRegistryPublish(this.handle, tailsUrl);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async publishRevocations(submittderDid: string): Promise<void> {
    try {
      await ffi.revocationRegistryPublishRevocations(this.handle, submittderDid);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getRevRegId(): string {
    try {
      return ffi.revocationRegistryGetRevRegId(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getTailsHash(): string {
    try {
      return ffi.revocationRegistryGetTailsHash(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }
}
