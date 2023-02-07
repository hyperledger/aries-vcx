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

export interface IProofConstructorData {
  attrs: IProofAttr[];
  preds: IProofPredicate[];
  name: string;
}

export interface IProofData {
  source_id: string;
  handle: number;
  requested_attrs: string;
  requested_predicates: string;
  prover_did: string;
  state: number;
  name: string;
  proof_state: ProofState;
  proof: any; // eslint-disable-line @typescript-eslint/no-explicit-any
}

export interface IProofResponses {
  proof?: string;
  proofState: ProofState;
}

export enum ProofFieldType {
  Revealed = 'revealed',
  Unrevealed = 'unrevealed',
  SelfAttested = 'self_attested',
  Predicate = 'predicate',
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

export enum ProofState {
  Undefined = 0,
  Verified = 1,
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

export class Proof extends VcxBaseWithState<IProofData, VerifierStateType> {
  public static async create({ sourceId, ...createDataRest }: IProofCreateData): Promise<Proof> {
    try {
      const proof = new Proof(sourceId);
      const handle = await ffi.proofCreate(
        proof.sourceId,
        JSON.stringify(createDataRest.attrs),
        JSON.stringify(createDataRest.preds || []),
        JSON.stringify(createDataRest.revocationInterval),
        createDataRest.name,
      );
      proof._setHandle(handle);
      return proof;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public static deserialize(proofData: ISerializedData<IProofData>): Proof {
    try {
      return super._deserialize<Proof, IProofConstructorData>(Proof, proofData);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  protected _releaseFn = ffi.proofRelease;
  protected _updateStFnV2 = ffi.v2ProofUpdateState;
  protected _getStFn = ffi.proofGetState;
  protected _serializeFn = ffi.proofSerialize;
  protected _deserializeFn = ffi.proofDeserialize;

  private static getParams(proofData: ISerializedData<IProofData>): IProofConstructorData {
    const {
      data: { requested_attrs, requested_predicates, name },
    } = proofData;
    const attrs = JSON.parse(requested_attrs);
    const preds = JSON.parse(requested_predicates);
    return { attrs, name, preds };
  }

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

  public getProof(): IProofResponses {
    try {
      const proof = ffi.proofGetProofMsg(this.handle);
      const proofState = ffi.proofGetProofState(this.handle);
      return {
        proof,
        proofState,
      };
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }
}
