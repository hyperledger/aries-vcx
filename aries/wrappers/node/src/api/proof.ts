import * as ffi from '@hyperledger/vcx-napi-rs';
import { ISerializedData, VerifierStateType } from './common';
import { Connection } from './mediated-connection';
import { NonmediatedConnection } from './connection';
import { VcxBaseWithState } from './vcx-base-with-state';
import { VCXInternalError } from '../errors';

export interface IProofCreateData {
  sourceId: string;
  attrs: IProofAttr[];
  preds: IProofPredicate[];
  name: string;
  revocationInterval: IRevocationInterval;
}

export interface IProofCreateDataV2 {
  sourceId: string;
  attrs: Record<string, any>;
  preds: Record<string, IProofPredicate>;
  name: string;
  revocationInterval: IRevocationInterval;
}

export interface IProofConstructorData {
  attrs: IProofAttr[];
  preds: IProofPredicate[];
  name: string;
}

export enum PredicateTypes {
  GE = 'GE',
  LE = 'LE',
  EQ = 'EQ',
}

export interface IProofAttr {
  restrictions?: IFilter[] | IFilter;
  // Requested attribute name
  name?: string;
  // Requested attribute names. Can be used to specify several attributes that have to match a single credential.
  // NOTE: should either be "name" or "names", not both and not none of them.
  names?: string[];
}

export interface IFilter {
  schema_id?: string;
  schema_issuer_did?: string;
  schema_name?: string;
  schema_version?: string;
  issuer_did?: string;
  cred_def_id?: string;
}

export enum ProofVerificationStatus {
  Unavailable = 0,
  Valid = 1,
  Invalid = 2,
}

export interface IProofPredicate {
  name: string;
  p_type: string;
  p_value: number;
  restrictions?: IFilter[];
}

export interface IRevocationInterval {
  from?: number;
  to?: number;
}

export class Proof extends VcxBaseWithState<Record<string, unknown>, VerifierStateType> {
  public static async create({
    sourceId,
    ...createDataRest
  }: IProofCreateData | IProofCreateDataV2): Promise<Proof> {
    try {
      const proof = new Proof();
      const handle = await ffi.proofCreate(
        sourceId,
        JSON.stringify(createDataRest.attrs),
        JSON.stringify(createDataRest.preds || {}),
        JSON.stringify(createDataRest.revocationInterval),
        createDataRest.name,
      );
      proof._setHandle(handle);
      return proof;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public static deserialize(serializedData: Record<string, unknown>): Proof {
    try {
      return super._deserialize<Proof, IProofConstructorData>(Proof, serializedData);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  protected _releaseFn = ffi.proofRelease;
  protected _updateStFnV2 = ffi.v2ProofUpdateState;
  protected _getStFn = ffi.proofGetState;
  protected _serializeFn = ffi.proofSerialize;
  protected _deserializeFn = ffi.proofDeserialize;

  public async updateStateWithMessage(connection: Connection, message: string): Promise<number> {
    try {
      return await ffi.v2ProofUpdateStateWithMessage(
        this.handle,
        message,
        connection.handle,
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async updateStateWithMessageNonmediated(connection: NonmediatedConnection, message: string): Promise<number> {
    try {
      return await ffi.proofUpdateStateWithMessageNonmediated(
        this.handle,
        connection.handle,
        message,
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async requestProof(connection: Connection): Promise<void> {
    try {
      return await ffi.proofSendRequest(this.handle, connection.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async requestProofNonmediated(connection: NonmediatedConnection): Promise<void> {
    try {
      return await ffi.proofSendRequestNonmediated(this.handle, connection.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getProofRequestMessage(): string {
    try {
      return ffi.proofGetRequestMsg(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public markPresentationRequestMsgSent(): void {
    try {
      return ffi.markPresentationRequestMsgSent(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getThreadId(): string {
    try {
      return ffi.proofGetThreadId(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getPresentationMsg(): string {
    try {
      return ffi.proofGetPresentationMsg(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getVerificationStatus(): ProofVerificationStatus {
    try {
      return ffi.proofGetVerificationStatus(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getPresentationAttachment(): string {
    try {
      return ffi.proofGetPresentationAttachment(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getPresentationRequestAttachment(): string {
    try {
      return ffi.proofGetPresentationRequestAttachment(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }
}
