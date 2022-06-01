import * as ffi from 'ffi-napi';
import { VCXInternalError } from '../errors';
import { rustAPI } from '../rustlib';
import { createFFICallbackPromise } from '../utils/ffi-helpers';
import { ISerializedData, IssuerStateType } from './common';
import { Connection } from './connection';
import { CredentialDef } from './credential-def';
import { VCXBase } from './vcx-base';

export interface IRevocationRegistryData {
  source_id: string;
  cred_def_id: string,
  issuer_did: string,
  rev_reg_id: string,
  rev_reg_def: string,
  rev_reg_entry: string,
  tails_dir: string,
  max_creds: number,
  tag: number,
  rev_reg_def_state: string,
  rev_reg_delta_state: string,
}

export interface IRevocationRegistryConfig {
  issuerDid: string;
  credDefId: string;
  tag: number;
  tailsDir: string;
  maxCreds: number;
}

export class RevocationRegistry extends VCXBase<IRevocationRegistryData> {
  public static async create(config: IRevocationRegistryConfig): Promise<RevocationRegistry> {
    try {
      const revReg = new RevocationRegistry('');
      const commandHandle = 0;
      const _config = {
        issuer_did: config.issuerDid,
        cred_def_id: config.credDefId,
        tag: config.tag,
        tails_dir: config.tailsDir,
        max_creds: config.maxCreds
      }
      await revReg._create((cb) =>
        rustAPI().vcx_revocation_registry_create(commandHandle, JSON.stringify(_config), cb),
      );
      return revReg;
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  public async rotate(maxCreds: number): Promise<RevocationRegistry> {
    try {
      const commandHandle = 0;
      const revReg = new RevocationRegistry('');
      await revReg._create((cb) =>
        rustAPI().vcx_revocation_registry_rotate(commandHandle, this.handle, maxCreds, cb),
      );
      return revReg;
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  public async publish(tailsUrl: string): Promise<void> {
    try {
      const revRegId = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_revocation_registry_publish(0, this.handle, tailsUrl, cb);
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          ffi.Callback(
            'void',
            ['uint32', 'uint32', 'uint32'],
            (xcommandHandle: number, err: number, handle: number) => {
              if (err) {
                reject(err);
                return;
              }
              resolve();
            },
          ),
      );
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  public async getRevRegId(): Promise<string> {
    try {
      const revRegId = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_revocation_registry_get_rev_reg_id(0, this.handle, cb);
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          ffi.Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (xcommandHandle: number, err: number, _revRegId: string) => {
              if (err) {
                reject(err);
                return;
              }
              resolve(_revRegId);
            },
          ),
      );
      return revRegId;
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  public async getTailsHash(): Promise<string> {
    try {
      const tailsHash = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_revocation_registry_get_tails_hash(0, this.handle, cb);
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          ffi.Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (xcommandHandle: number, err: number, tailsHash: string) => {
              if (err) {
                reject(err);
                return;
              }
              resolve(tailsHash);
            },
          ),
      );
      return tailsHash;
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  public static async deserialize(
    data: ISerializedData<IRevocationRegistryData>,
  ): Promise<RevocationRegistry> {
    const newObj = { ...data, source_id: 'foo' };
    return super._deserialize(RevocationRegistry, newObj);
  }

  protected _releaseFn = rustAPI().vcx_revocation_registry_release;
  protected _serializeFn = rustAPI().vcx_revocation_registry_serialize;
  protected _deserializeFn = rustAPI().vcx_revocation_registry_deserialize;
}
