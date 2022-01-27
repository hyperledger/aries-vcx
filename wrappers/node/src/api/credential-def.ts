import * as ffi from 'ffi-napi';
import { VCXInternalError } from '../errors';
import { rustAPI } from '../rustlib';
import { createFFICallbackPromise } from '../utils/ffi-helpers';
import { ISerializedData } from './common';
import { VCXBase } from './vcx-base';

/**
 * @interface Interface that represents the parameters for `CredentialDef.create` function.
 * @description
 * SourceId: Enterprise's personal identification for the user.
 * name: Name of credential definition
 * schemaId: The schema id given during the creation of the schema
 * revocation: type-specific configuration of credential definition revocation
 *     TODO: Currently supports ISSUANCE BY DEFAULT, support for ISSUANCE ON DEMAND will be added as part of ticket: IS-1074
 *     support_revocation: true|false - Optional, by default its false
 *     tails_file: path to tails file - Optional if support_revocation is false
 *     max_creds: size of tails file - Optional if support_revocation is false
 */
export interface ICredentialDefCreateData {
  sourceId: string;
  schemaId: string;
  revocationDetails: IRevocationDetails;
  tailsUrl?: string;
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

/**
 * @class Class representing a credential Definition
 */
export class CredentialDef extends VCXBase<ICredentialDefData> {
  public static async createAndStore({
    revocationDetails,
    schemaId,
    sourceId,
  }: ICredentialDefCreateData): Promise<CredentialDef> {
    const tailsDir = revocationDetails.tailsDir;
    const credentialDef = new CredentialDef(sourceId, { schemaId, tailsDir });
    const commandHandle = 0;
    const issuerDid = null;
    const revocation = {
      max_creds: revocationDetails.maxCreds,
      support_revocation: revocationDetails.supportRevocation,
      tails_dir: revocationDetails.tailsDir,
    };
    try {
      await credentialDef._create((cb) =>
        rustAPI().vcx_credentialdef_create_and_store(
          commandHandle,
          sourceId,
          schemaId,
          issuerDid,
          'tag1',
          JSON.stringify(revocation),
          cb,
        ),
      );
      return credentialDef;
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  public static async deserialize(
    credentialDef: ISerializedData<ICredentialDefData>,
  ): Promise<CredentialDef> {
    // Todo: update the ICredentialDefObj
    const {
      data: { name },
    } = credentialDef;
    const credentialDefParams = {
      name,
      schemaId: null,
    };
    return super._deserialize(CredentialDef, credentialDef, credentialDefParams);
  }

  protected _releaseFn = rustAPI().vcx_credentialdef_release;
  protected _serializeFn = rustAPI().vcx_credentialdef_serialize;
  protected _deserializeFn = rustAPI().vcx_credentialdef_deserialize;
  private _name: string | undefined;
  private _schemaId: string | undefined;
  private _credDefId: string | undefined;
  private _tailsDir: string | undefined;
  private _credDefTransaction: string | null;
  private _revocRegDefTransaction: string | null;
  private _revocRegEntryTransaction: string | null;

  constructor(sourceId: string, { name, schemaId, credDefId, tailsDir }: ICredentialDefParams) {
    super(sourceId);
    this._name = name;
    this._schemaId = schemaId;
    this._credDefId = credDefId;
    this._tailsDir = tailsDir;
    this._credDefTransaction = null;
    this._revocRegDefTransaction = null;
    this._revocRegEntryTransaction = null;
  }

  public async publish(tailsUrl?: string): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_credentialdef_publish(0, this.handle, tailsUrl || null, cb);
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          ffi.Callback(
            'void',
            ['uint32', 'uint32'],
            (handle: number, err: number) => {
              if (err) {
                reject(err);
              }
              resolve();
            },
          ),
      );
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }


  /**
   * Retrieves the credential definition id associated with the created cred def.
   * Example:
   * ```
   * data = {
   *   name: 'testCredentialDefName',
   *   revocation: false,
   *   schemaId: 'testCredentialDefSchemaId',
   *   sourceId: 'testCredentialDefSourceId'
   * }
   * credentialDef = await CredentialDef.create(data)
   * id = await credentialDef.getCredDefId()
   * ```
   */
  public async getCredDefId(): Promise<string> {
    try {
      const credDefId = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_credentialdef_get_cred_def_id(0, this.handle, cb);
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          ffi.Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (xcommandHandle: number, err: number, credDefIdVal: string) => {
              if (err) {
                reject(err);
                return;
              }
              this._credDefId = credDefIdVal;
              resolve(credDefIdVal);
            },
          ),
      );
      return credDefId;
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  public async getTailsHash(): Promise<string> {
    try {
      const tailsHash = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_credentialdef_get_tails_hash(0, this.handle, cb);
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          ffi.Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (xcommandHandle: number, err: number, _tailsHash: string) => {
              if (err) {
                reject(err);
                return;
              }
              resolve(_tailsHash);
            },
          ),
      );
      return tailsHash;
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  public async getRevRegId(): Promise<string> {
    try {
      const revRegId = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_credentialdef_get_rev_reg_id(0, this.handle, cb);
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

  /**
   *
   * Checks if credential definition is published on the Ledger and updates the state
   *
   * Example:
   * ```
   * await credentialDef.updateState()
   * ```
   * @returns {Promise<void>}
   */
  public async updateState(): Promise<CredentialDefState> {
    try {
      const state = await createFFICallbackPromise<CredentialDefState>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_credentialdef_update_state(0, this.handle, cb);
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          ffi.Callback(
            'void',
            ['uint32', 'uint32', 'uint32'],
            (handle: number, err: number, _state: CredentialDefState) => {
              if (err) {
                reject(err);
              }
              resolve(_state);
            },
          ),
      );
      return state;
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  /**
   * Get the current state of the credential definition object
   *
   * Example:
   * ```
   * state = await credentialdef.getState()
   * ```
   * @returns {Promise<CredentialDefState>}
   */
  public async getState(): Promise<CredentialDefState> {
    try {
      const stateRes = await createFFICallbackPromise<CredentialDefState>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_credentialdef_get_state(0, this.handle, cb);
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          ffi.Callback(
            'void',
            ['uint32', 'uint32', 'uint32'],
            (handle: number, err: number, state: CredentialDefState) => {
              if (err) {
                reject(err);
              }
              resolve(state);
            },
          ),
      );
      return stateRes;
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  public async rotateRevRegDef(
    revocationDetails: IRevocationDetails
  ): Promise<ISerializedData<ICredentialDefCreateData>> {
    const revocation = {
      max_creds: revocationDetails.maxCreds,
      tails_file: revocationDetails.tailsDir,
    };
    try {
      const dataStr = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_credentialdef_rotate_rev_reg_def(
            0,
            this.handle,
            JSON.stringify(revocation),
            cb,
          );
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          ffi.Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (handle: number, err: number, x: string) => {
              if (err) {
                reject(err);
              }
              resolve(x);
            },
          ),
      );
      const data: ISerializedData<ICredentialDefCreateData> = JSON.parse(dataStr);
      return data;
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  public async publishRevocations(): Promise<void> {
    try {
      await createFFICallbackPromise<number>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_credentialdef_publish_revocations(0, this.handle, cb);
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          ffi.Callback(
            'void',
            ['uint32', 'uint32', 'uint32'],
            (handle: number, err: number, state: CredentialDefState) => {
              if (err) {
                reject(err);
              }
              resolve(state);
            },
          ),
      );
    } catch (err) {
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

  get credentialDefTransaction(): string | null {
    return this._credDefTransaction;
  }

  get revocRegDefTransaction(): string | null {
    return this._revocRegDefTransaction;
  }

  get revocRegEntryTransaction(): string | null {
    return this._revocRegEntryTransaction;
  }
}
