import * as ffi from 'ffi-napi';
import { VCXInternalError } from '../errors';
import { rustAPI } from '../rustlib';
import { createFFICallbackPromise } from '../utils/ffi-helpers';
import { ISerializedData } from './common';
import { Connection, IConnectionData } from './connection';
import { VCXBase } from './vcx-base';

export interface IOOBSerializedData {
  id: string;
  label?: string;
  goal_code?: string;
  goal?: string;
  accept?: string;
  handshake_protocols?: string;
  requests_attach: string;
}

export interface IOOBCreateData {
  label?: string;
  goalCode?: GoalCode;
  goal?: string;
}

export enum GoalCode {
  IssueVC = 'issue-vc',
  RequestProof = 'request-proof',
  CreateAccount = 'create-account',
  P2PMessaging = 'p2p-messaging',
}

export enum HandshakeProtocol {
  ConnectionV1 = 0,
  DidExchangeV1 = 1,
}

export class OutOfBand extends VCXBase<IOOBSerializedData> {
  public static async create(config: IOOBCreateData): Promise<OutOfBand> {
    const oob = new OutOfBand("");
    const commandHandle = 0;
    try {
      await oob._create((cb) =>
        rustAPI().vcx_out_of_band_create(commandHandle, JSON.stringify(config), cb),
      );
      return oob;
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  public static async createWithMessage(msg: string): Promise<OutOfBand> {
    const oob = new OutOfBand("");
    const commandHandle = 0;
    try {
      await oob._create((cb) =>
        rustAPI().vcx_out_of_band_create_from_message(commandHandle, msg, cb),
      );
      return oob;
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  public async appendMessage(message: string): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const commandHandle = 0;
          const rc = rustAPI().vcx_out_of_band_append_message(
            commandHandle,
            this.handle,
            message,
            cb,
          );
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
                return;
              }
              resolve();
            },
          ),
      );
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  public async appendService(service: string): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const commandHandle = 0;
          const rc = rustAPI().vcx_out_of_band_append_service(
            commandHandle,
            this.handle,
            service,
            cb,
          );
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
                return;
              }
              resolve();
            },
          ),
      );
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  public async extractMessage(): Promise<string> {
    try {
      const msg = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const commandHandle = 0;
          const rc = rustAPI().vcx_out_of_band_extract_message(
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
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  public async toMessage(): Promise<string> {
    try {
      const msg = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const commandHandle = 0;
          const rc = rustAPI().vcx_out_of_band_to_message(
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
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  public async connectionExists(connections: [Connection]): Promise<void | Connection> {
    try {
      await createFFICallbackPromise<void | Connection>(
        (resolve, reject, cb) => {
          const commandHandle = 0;
          const rc = rustAPI().vcx_out_of_band_connection_exists(
            commandHandle,
            this.handle,
            JSON.stringify(connections),
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
                connections.forEach((conn) => {
                  if (conn.handle === conn_handle) {
                    resolve(conn);
                    return;
                  }
                });
                reject(Error('Unexpected state: should have found connection'));
              }
            },
          ),
      );
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  public async buildConnection(): Promise<Connection> {
    try {
      const connection = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const commandHandle = 0;
          const rc = rustAPI().vcx_out_of_band_build_connection(
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
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  // public static async deserialize(
  //   oobData: ISerializedData<IOOBSerializedData>,
  // ): Promise<OutOfBand> {
  //   return await super._deserialize<OutOfBand>(OutOfBand, oobData);
  // }

  protected _serializeFn = rustAPI().vcx_out_of_band_serialize;
  protected _deserializeFn = rustAPI().vcx_out_of_band_deserialize;
  protected _releaseFn = rustAPI().vcx_out_of_band_release;
}
