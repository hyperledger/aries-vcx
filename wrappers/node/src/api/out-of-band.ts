import * as ffi from 'ffi-napi';
import { VCXInternalError } from '../errors';
import { rustAPI } from '../rustlib';
import { createFFICallbackPromise } from '../utils/ffi-helpers';
import { ISerializedData } from './common';
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
  DidCommV1 = 1,
}

export class OutOfBand extends VCXBase<IOOBSerializedData> {
  public static async create(sourceId: string, config: IOOBCreateData): Promise<OutOfBand> {
    const oob = new OutOfBand(sourceId);
    const commandHandle = 0;
    try {
      await oob._create((cb) =>
        rustAPI().vcx_out_of_band_create(commandHandle, sourceId, JSON.stringify(config), cb),
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

  protected _serializeFn = rustAPI().vcx_out_of_band_serialize;
  protected _deserializeFn = rustAPI().vcx_out_of_band_deserialize;
  protected _releaseFn = rustAPI().vcx_out_of_band_release;
}
