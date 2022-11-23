import * as ffi from 'node-napi-rs';
import { VCXInternalError } from '../errors';
import { IOOBSerializedData } from './out-of-band-sender';
import { Connection } from './mediated-connection';
import { VCXBase1 } from './vcx-base-1';
import { ISerializedData } from './common';

export class OutOfBandReceiver extends VCXBase1<IOOBSerializedData> {
  public static createWithMessage(msg: string): OutOfBandReceiver {
    const oob = new OutOfBandReceiver("");
    try {
      oob._setHandle(ffi.createOutOfBandMsgFromMsg(msg))
      return oob;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public static deserialize(
    data: ISerializedData<IOOBSerializedData>,
  ): OutOfBandReceiver {
    const newObj = { ...data, source_id: 'foo' };
    return super._deserialize(OutOfBandReceiver, newObj);
  }

  public extractMessage(): string {
    try {
      return ffi.extractA2AMessage(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async connectionExists(connections: [Connection]): Promise<void | Connection> {
    try {
      const connHandles = connections.map((conn) => conn.handle);
      const connHandle = await ffi.connectionExists(this.handle, connHandles);
      return connections.find((conn) => conn.handle === connHandle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async buildConnection(): Promise<Connection> {
    try {
      const connection = await ffi.buildConnection(this.handle);
      return await Connection.deserialize(JSON.parse(connection));
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getThreadId(): string {
    try {
      return ffi.getThreadIdReceiver(this.handle)
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  protected _serializeFn = ffi.toStringReceiver;
  protected _deserializeFn = ffi.fromStringReceiver;
  protected _releaseFn = ffi.releaseReceiver;
}
