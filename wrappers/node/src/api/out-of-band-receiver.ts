import * as ffi from '@hyperledger/vcx-napi-rs';
import { VCXInternalError } from '../errors';
import { IOOBSerializedData } from './out-of-band-sender';
import { Connection } from './mediated-connection';
import { NonmediatedConnection } from './connection';
import { VcxBase } from './vcx-base';
import { ISerializedData } from './common';

export class OutOfBandReceiver extends VcxBase<IOOBSerializedData> {
  public static createWithMessage(msg: string): OutOfBandReceiver {
    const oob = new OutOfBandReceiver();
    try {
      oob._setHandle(ffi.outOfBandReceiverCreate(msg));
      return oob;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public static deserialize(data: ISerializedData<IOOBSerializedData>): OutOfBandReceiver {
    const newObj = { ...data, source_id: 'foo' };
    try {
      return super._deserialize(OutOfBandReceiver, newObj);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public extractMessage(): string {
    try {
      return ffi.outOfBandReceiverExtractMessage(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async connectionExists(connections: [Connection]): Promise<void | Connection> {
    try {
      const connHandles = connections.map((conn) => conn.handle);
      const connHandle = await ffi.outOfBandReceiverConnectionExists(this.handle, connHandles);
      return connections.find((conn) => conn.handle === connHandle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async nonmediatedConnectionExists(connections: [NonmediatedConnection]): Promise<void | NonmediatedConnection> {
    try {
      const connHandles = connections.map((conn) => conn.handle);
      const connHandle = await ffi.outOfBandReceiverNonmediatedConnectionExists(this.handle, connHandles);
      return connections.find((conn) => conn.handle === connHandle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async buildConnection(): Promise<Connection> {
    try {
      const connection = await ffi.outOfBandReceiverBuildConnection(this.handle);
      return Connection.deserialize(JSON.parse(connection));
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getThreadId(): string {
    try {
      return ffi.outOfBandReceiverGetThreadId(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  protected _serializeFn = ffi.outOfBandReceiverSerialize;
  protected _deserializeFn = ffi.outOfBandReceiverDeserialize;
  protected _releaseFn = ffi.outOfBandReceiverRelease;
}
