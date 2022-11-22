import * as ffi from 'node-napi-rs';
import { VCXInternalError } from '../errors';
import { rustAPI } from '../rustlib';
import { VCXBase } from './vcx-base';
import { ISerializedData } from './common';

export interface IOOBSerializedData {
  source_id: string;
  id: string;
  label?: string;
  goal_code?: string;
  goal?: string;
  accept?: string;
  handshake_protocols?: string;
  requests_attach: string;
}

export interface IOOBCreateData {
  source_id: string;
  label?: string;
  goalCode?: GoalCode;
  goal?: string;
  handshake_protocols?: HandshakeProtocol[];
}

export enum GoalCode {
  IssueVC = 'issue-vc',
  RequestProof = 'request-proof',
  CreateAccount = 'create-account',
  P2PMessaging = 'p2p-messaging',
}

export enum HandshakeProtocol {
  ConnectionV1 = "ConnectionV1",
  DidExchangeV1 = "DidExchangeV1",
}

export class OutOfBandSender extends VCXBase<IOOBSerializedData> {
  public static async create(config: IOOBCreateData): Promise<OutOfBandSender> {
    const oob = new OutOfBandSender(config.source_id);
    try {
      oob._setHandle(await ffi.createOutOfBand(JSON.stringify(config)));
      return oob;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public static async deserialize(
    data: ISerializedData<IOOBSerializedData>,
  ): Promise<OutOfBandSender> {
    const newObj = { ...data, source_id: 'foo' };
    return super._deserialize(OutOfBandSender, newObj);
  }

  public appendMessage(message: string): void {
    try {
      ffi.appendMessage(this.handle, message);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public appendServiceDid(did: string): void {
      try {
        ffi.appendServiceDid(this.handle, did);
      } catch (err: any) {
        throw new VCXInternalError(err);
      }
  }

  public appendService(service: string): void {
    try {
        ffi.appendService(this.handle, service);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public toMessage(): string {
    try {
      return ffi.toA2AMessage(this.handle)
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getThreadId(): string {
    try {
      return ffi.getThreadIdSender(this.handle)
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public serialize_(): ISerializedData<IOOBSerializedData> {
    try {
      return JSON.parse(ffi.toStringSender(this.handle))
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  protected _serializeFn = rustAPI().vcx_out_of_band_sender_serialize;
  protected _deserializeFn = rustAPI().vcx_out_of_band_sender_deserialize;
  protected _releaseFn = rustAPI().vcx_out_of_band_sender_release;
}
