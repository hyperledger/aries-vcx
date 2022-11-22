import * as ffiNapi from 'node-napi-rs';
import { VCXInternalError } from '../errors';
import { rustAPI } from '../rustlib';
import { IOOBSerializedData } from './out-of-band-sender';
import { Connection } from './mediated-connection';
import { VCXBase } from './vcx-base';
import { ISerializedData } from './common';

export class OutOfBandReceiver extends VCXBase<IOOBSerializedData> {
  public static createWithMessage(msg: string): OutOfBandReceiver {
    const oob = new OutOfBandReceiver("");
    try {
      oob._setHandle(ffiNapi.createOutOfBandMsgFromMsg(msg))
      return oob;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public static async deserialize(
    data: ISerializedData<IOOBSerializedData>,
  ): Promise<OutOfBandReceiver> {
    const newObj = { ...data, source_id: 'foo' };
    return super._deserialize(OutOfBandReceiver, newObj);
  }

  public extractMessage(): string {
    try {
      return ffiNapi.extractA2AMessage(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async connectionExists(connections: [Connection]): Promise<void | Connection> {
    try {
      const connHandles = connections.map((conn) => conn.handle);
      const connHandle = await ffiNapi.connectionExists(this.handle, connHandles);
      return connections.find((conn) => conn.handle === connHandle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async buildConnection(): Promise<Connection> {
    try {
      const connection = await ffiNapi.buildConnection(this.handle);
      return await Connection.deserialize(JSON.parse(connection));
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getThreadId(): string {
    try {
      return ffiNapi.getThreadIdReceiver(this.handle)
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  protected _serializeFn = rustAPI().vcx_out_of_band_receiver_serialize;
  protected _deserializeFn = rustAPI().vcx_out_of_band_receiver_deserialize;
  protected _releaseFn = rustAPI().vcx_out_of_band_receiver_release;
}
