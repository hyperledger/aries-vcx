import * as ffi from 'ffi-napi';
import { VCXInternalError } from '../errors';
import { rustAPI } from '../rustlib';
import { createFFICallbackPromise } from '../utils/ffi-helpers';
import { IOOBSerializedData } from './out-of-band-sender';
import { Connection } from './connection';
import { VCXBase } from './vcx-base';
import { ISerializedData } from './common';

export class OutOfBandReceiver extends VCXBase<IOOBSerializedData> {
  public static async createWithMessage(msg: string): Promise<OutOfBandReceiver> {
    const oob = new OutOfBandReceiver("");
    const commandHandle = 0;
    try {
      await oob._create((cb) =>
        rustAPI().vcx_out_of_band_receiver_create(commandHandle, msg, cb),
      );
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

  public async extractMessage(): Promise<string> {
    try {
      const msg = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const commandHandle = 0;
          const rc = rustAPI().vcx_out_of_band_receiver_extract_message(
            commandHandle,
            this.handle,
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
            (handle: number, err: number, msg: string) => {
              if (err) {
                reject(err);
                return;
              }
              resolve(msg);
            },
          ),
      );
      return msg
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async connectionExists(connections: [Connection]): Promise<void | Connection> {
    try {
      const connHandles = connections.map((conn) => conn.handle);
      const res = await createFFICallbackPromise<void | Connection>(
        (resolve, reject, cb) => {
          const commandHandle = 0;
          const rc = rustAPI().vcx_out_of_band_receiver_connection_exists(
            commandHandle,
            this.handle,
            JSON.stringify(connHandles),
            cb,
          );
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          ffi.Callback(
            'void',
            ['uint32', 'uint32', 'uint32', 'bool'],
            (handle: number, err: number, conn_handle: number, found_one: boolean) => {
              if (err) {
                reject(err);
                return;
              }
              if (!found_one) {
                resolve();
              } else {
                const conn = connections.find((conn) => conn.handle === conn_handle);
                if (conn) {
                  resolve(conn);
                  return;
                }
                reject(Error('Unexpected state: should have found connection'));
              }
            },
          ),
      );
      return res
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async buildConnection(): Promise<Connection> {
    try {
      const connection = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const commandHandle = 0;
          const rc = rustAPI().vcx_out_of_band_receiver_build_connection(
            commandHandle,
            this.handle,
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
            (handle: number, err: number, connection: string) => {
              if (err) {
                reject(err);
                return;
              }
              resolve(connection);
            },
          ),
      );
      return await Connection.deserialize(JSON.parse(connection));
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async getThreadId(): Promise<string> {
    try {
      const thid = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const commandHandle = 0;
          const rc = rustAPI().vcx_out_of_band_receiver_get_thread_id(
            commandHandle,
            this.handle,
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
            (handle: number, err: number, thid: string) => {
              if (err) {
                reject(err);
                return;
              }
              resolve(thid);
            },
          ),
      );
      return thid;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  protected _serializeFn = rustAPI().vcx_out_of_band_receiver_serialize;
  protected _deserializeFn = rustAPI().vcx_out_of_band_receiver_deserialize;
  protected _releaseFn = rustAPI().vcx_out_of_band_receiver_release;
}
