import * as ffi from 'ffi-napi';
import { VCXInternalError } from '../errors';
import { createFFICallbackPromise, ICbRef } from '../utils/ffi-helpers';
import { Connection } from './connection';
import { VCXBase } from './vcx-base';

export abstract class VCXBaseWithState<SerializedData> extends VCXBase<SerializedData> {
  protected abstract _updateStFnV2: (
    commandHandle: number,
    handle: number,
    connHandle: number,
    cb: ICbRef,
  ) => number;
  protected abstract _getStFn: (commandHandle: number, handle: number, cb: ICbRef) => number;

  public async updateStateV2(connection: Connection): Promise<number> {
    try {
      const commandHandle = 0;
      const state = await createFFICallbackPromise<number>(
        (resolve, reject, cb) => {
          const rc = this._updateStFnV2(commandHandle, this.handle, connection.handle, cb);
          if (rc) {
            resolve(0);
          }
        },
        (resolve, reject) =>
          ffi.Callback(
            'void',
            ['uint32', 'uint32', 'uint32'],
            (handle: number, err: number, _state: number) => {
              if (err) {
                reject(err);
              }
              resolve(_state);
            },
          ),
      );
      return state;
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  /**
   * Gets the state of the entity.
   *
   * Example:
   * ```
   * state = await object.getState()
   * ```
   * @returns {Promise<number>}
   */
  public async getState(): Promise<number> {
    try {
      const commandHandle = 0;
      const stateRes = await createFFICallbackPromise<number>(
        (resolve, reject, cb) => {
          const rc = this._getStFn(commandHandle, this.handle, cb);
          if (rc) {
            resolve(0);
          }
        },
        (resolve, reject) =>
          ffi.Callback(
            'void',
            ['uint32', 'uint32', 'uint32'],
            (handle: number, err: number, state: number) => {
              if (err) {
                reject(err);
              }
              resolve(state);
            },
          ),
      );
      return stateRes;
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }
}
