import * as ffi from 'ffi-napi';
import { VCXInternalError } from '../errors';
import { rustAPI } from '../rustlib';
import { createFFICallbackPromise } from '../utils/ffi-helpers';
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
    const commandHandle = 0;
    try {
      await oob._create((cb) =>
        rustAPI().vcx_out_of_band_sender_create(commandHandle, JSON.stringify(config), cb),
      );
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

  public async appendMessage(message: string): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const commandHandle = 0;
          const rc = rustAPI().vcx_out_of_band_sender_append_message(
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
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async appendServiceDid(did: string): Promise<void> {
      try {
          await createFFICallbackPromise<void>(
              (resolve, reject, cb) => {
                  const commandHandle = 0;
                  const rc = rustAPI().vcx_out_of_band_sender_append_service_did(
                      commandHandle,
                      this.handle,
                      did,
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
      } catch (err: any) {
          throw new VCXInternalError(err);
      }
  }

  public async appendService(service: string): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const commandHandle = 0;
          const rc = rustAPI().vcx_out_of_band_sender_append_service(
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
    } catch (err: any) {
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
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async getThreadId(): Promise<string> {
    try {
      const thid = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const commandHandle = 0;
          const rc = rustAPI().vcx_out_of_band_sender_get_thread_id(
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

  protected _serializeFn = rustAPI().vcx_out_of_band_sender_serialize;
  protected _deserializeFn = rustAPI().vcx_out_of_band_sender_deserialize;
  protected _releaseFn = rustAPI().vcx_out_of_band_sender_release;
}
