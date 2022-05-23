import * as ffi from 'ffi-napi';
import { VCXInternalError } from '../errors';
import { rustAPI } from '../rustlib';
import { createFFICallbackPromise } from '../utils/ffi-helpers';
import { ISerializedData, IssuerStateType } from './common';
import { Connection } from './connection';
import { CredentialDef } from './credential-def';
import { VCXBase } from './vcx-base';

export interface IRevocationRegistryData {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  issuer_sm: Record<string, any>;
  source_id: string;
}

export interface IRevocationRegistryConfig {
  issuer_did: string;
  cred_def_id: string;
  tag: number;
  tails_dir: string;
  max_creds: number;
}

export class RevocationRegistry extends VCXBase<IRevocationRegistryData> {
  public static async create(config: IRevocationRegistryConfig): Promise<RevocationRegistry> {
    try {
      const revReg = new RevocationRegistry('');
      const commandHandle = 0;
      await revReg._create((cb) =>
        rustAPI().vcx_revocation_registry_create(commandHandle, JSON.stringify(config), cb),
      );
      return revReg;
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  public static async rotate(maxCreds: number): Promise<RevocationRegistry> {
    try {
      const commandHandle = 0;
      const revReg = new RevocationRegistry('');
      await revReg._create((cb) =>
        rustAPI().vcx_revocation_registry_rotate(commandHandle, maxCreds, cb),
      );
      return revReg;
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  protected _releaseFn = rustAPI().vcx_revocation_registry_release;
  protected _serializeFn = rustAPI().vcx_revocation_registry_serialize;
  protected _deserializeFn = rustAPI().vcx_revocation_registry_deserialize;
}
