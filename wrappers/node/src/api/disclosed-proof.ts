import { ISerializedData, ProverStateType } from './common';
import { Connection } from './mediated-connection';
import { VcxBaseWithState } from './vcx-base-with-state';
import * as ffi from '@hyperledger/vcx-napi-rs';
import { VCXInternalError } from '../errors';

export interface IDisclosedProofData {
  source_id: string;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type IDisclosedProofRequest = Record<string, any>;

export interface IDisclosedProofCreateData {
  connection: Connection;
  sourceId: string;
  request: string;
}

export interface IDisclosedProofCreateWithMsgIdData {
  connection: Connection;
  msgId: string;
  sourceId: string;
}

export interface IRetrievedCreds {
  attrs: {
    [index: string]: ICredData[];
  };
  predicates: any; // eslint-disable-line @typescript-eslint/no-explicit-any
}

export interface ICredData {
  cred_info: {
    [index: string]: any; // eslint-disable-line @typescript-eslint/no-explicit-any
  };
  interval: any; // eslint-disable-line @typescript-eslint/no-explicit-any
}

export interface IGenerateProofData {
  selectedCreds: {
    [index: string]: ICredData;
  };
  selfAttestedAttrs: {
    [index: string]: string;
  };
}

export interface IDeclinePresentationRequestData {
  connection: Connection;
  reason?: string;
  proposal?: any; // eslint-disable-line @typescript-eslint/no-explicit-any
}

export class DisclosedProof extends VcxBaseWithState<IDisclosedProofData, ProverStateType> {
  public static create({ sourceId, request }: IDisclosedProofCreateData): DisclosedProof {
    try {
      const disclosedProof = new DisclosedProof();
      disclosedProof._setHandle(ffi.disclosedProofCreateWithRequest(sourceId, request));
      return disclosedProof;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public static deserialize(data: ISerializedData<IDisclosedProofData>): DisclosedProof {
    try {
      return super._deserialize(DisclosedProof, data as any);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  protected _releaseFn = ffi.disclosedProofRelease;
  protected _updateStFnV2 = ffi.v2DisclosedProofUpdateState;
  protected _getStFn = ffi.disclosedProofGetState;
  protected _serializeFn = ffi.disclosedProofSerialize;
  protected _deserializeFn = ffi.disclosedProofDeserialize;

  public static async getRequests(connection: Connection): Promise<IDisclosedProofRequest[]> {
    try {
      const string_msgs = await ffi.disclosedProofGetRequests(connection.handle);
      return JSON.parse(string_msgs);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async getCredentials(): Promise<IRetrievedCreds> {
    try {
      const credentials = await ffi.disclosedProofRetrieveCredentials(this.handle);
      return JSON.parse(credentials);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getProofRequestAttachment(): string {
    try {
      return ffi.disclosedProofGetProofRequestAttachment(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getThreadId(): string {
    try {
      return ffi.disclosedProofGetThreadId(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async sendProof(connection: Connection): Promise<void> {
    try {
      return await ffi.disclosedProofSendProof(this.handle, connection.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async rejectProof(connection: Connection): Promise<void> {
    try {
      return await ffi.disclosedProofRejectProof(this.handle, connection.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getProofMessage(): string {
    try {
      return ffi.disclosedProofGetProofMsg(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async generateProof({
    selectedCreds,
    selfAttestedAttrs,
  }: IGenerateProofData): Promise<void> {
    try {
      return await ffi.disclosedProofGenerateProof(
        this.handle,
        JSON.stringify(selectedCreds),
        JSON.stringify(selfAttestedAttrs),
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async declinePresentationRequest({
    connection,
    reason,
    proposal,
  }: IDeclinePresentationRequestData): Promise<void> {
    try {
      return await ffi.disclosedProofDeclinePresentationRequest(
        this.handle,
        connection.handle,
        reason,
        JSON.stringify(proposal),
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }
}
