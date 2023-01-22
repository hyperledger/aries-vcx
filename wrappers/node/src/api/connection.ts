import * as ffiNapi from '@hyperledger/vcx-napi-rs';
import { VCXInternalError } from '../errors';
import { ISerializedData, ConnectionStateType } from './common';
import { VcxBaseWithState } from './vcx-base-with-state';
import { IPwInfo } from './utils';

export type INonmediatedConnectionInvite = string;

export type INonmeditatedConnectionData = string;

export interface IEndpointInfo {
  serviceEndpoint: string,
  routingKeys: string[],
}

export class NonmediatedConnection extends VcxBaseWithState<INonmeditatedConnectionData, ConnectionStateType> {
  public static async createInviter(pwInfo: IPwInfo): Promise<NonmediatedConnection> {
    try {
      const connection = new NonmediatedConnection("");
      connection._setHandle(await ffiNapi.connectionCreateInviter(JSON.stringify(pwInfo)));
      return connection;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public static async createInvitee(invite: string): Promise<NonmediatedConnection> {
    try {
      const connection = new NonmediatedConnection("");
      connection._setHandle(await ffiNapi.connectionCreateInvitee(invite));
      return connection;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getThreadId(): string {
    try {
      return ffiNapi.connectionGetThreadId(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getPairwiseInfo(): string {
    try {
      return ffiNapi.connectionGetPairwiseInfo(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getRemoteDid(): string {
    try {
      return ffiNapi.connectionGetRemoteDid(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public processInvite(invite: string): void {
    try {
      ffiNapi.connectionProcessInvite(this.handle, invite);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async processRequest(request: string, endpointInfo: IEndpointInfo): Promise<void> {
    try {
      const { serviceEndpoint, routingKeys } = endpointInfo;
      await ffiNapi.connectionProcessRequest(this.handle, request, serviceEndpoint, routingKeys);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async processResponse(response: string): Promise<void> {
    try {
      await ffiNapi.connectionProcessResponse(this.handle, response);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async processAck(message: string): Promise<void> {
    try {
      await ffiNapi.connectionProcessAck(this.handle, message);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async sendResponse(): Promise<void> {
    try {
      await ffiNapi.connectionSendResponse(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async sendRequest(endpointInfo: IEndpointInfo): Promise<void> {
    try {
      const { serviceEndpoint, routingKeys } = endpointInfo;
      await ffiNapi.connectionSendRequest(this.handle, serviceEndpoint, routingKeys);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async sendAck(): Promise<void> {
    try {
      await ffiNapi.connectionSendAck(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async createInvite(endpointInfo: IEndpointInfo): Promise<void> {
    try {
      const { serviceEndpoint, routingKeys } = endpointInfo;
      await ffiNapi.connectionCreateInvite(this.handle, serviceEndpoint, routingKeys);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getInvitation(): INonmediatedConnectionInvite {
    try {
      return ffiNapi.connectionGetInvitation(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getState(): ConnectionStateType {
    try {
      return ffiNapi.connectionGetState(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public static deserialize(connectionData: ISerializedData<string>): NonmediatedConnection {
    try {
      const connection = new NonmediatedConnection("");
      connection._setHandle(ffiNapi.connectionDeserialize(connectionData.data));
      return connection;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  protected _releaseFn = ffiNapi.connectionRelease;
  protected _updateStFn = ffiNapi.mediatedConnectionUpdateState;
  protected _updateStFnV2 = async (_handle: number, _connHandle: number): Promise<number> => {
    throw new Error('_updateStFnV2 cannot be called for a Connection object');
  };
  protected _getStFn = ffiNapi.connectionGetState;
  protected _serializeFn = (handle: number): string => {
    const data = ffiNapi.connectionSerialize(handle);
    return JSON.stringify({ data, source_id: this.sourceId, version: '1.0' });
  }
  protected _deserializeFn = ffiNapi.connectionDeserialize;
  protected _inviteDetailFn = ffiNapi.connectionGetInvitation;
}
