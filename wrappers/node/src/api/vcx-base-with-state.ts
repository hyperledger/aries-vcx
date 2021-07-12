import * as ffi from 'ffi-napi';
import { VCXInternalError } from '../errors';
import { createFFICallbackPromise, ICbRef } from '../utils/ffi-helpers';
import { Connection } from './connection';
import { VCXBase } from './vcx-base';

export abstract class VCXBaseWithState<SerializedData, StateType> extends VCXBase<SerializedData> {
  protected abstract _updateStFnV2: (
    commandHandle: number,
    handle: number,
    connHandle: number,
    cb: ICbRef,
  ) => StateType | number;
  protected abstract _getStFn: (commandHandle: number, handle: number, cb: ICbRef) => StateType | number;

  public async updateStateV2(connection: Connection): Promise<StateType | number> {
    try {
      const commandHandle = 0;
      const state = await createFFICallbackPromise<StateType | number>(
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
            (handle: number, err: number, _state: StateType | number) => {
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
   * @returns {Promise<StateType | number>}
   */
  public async getState(): Promise<StateType | number> {
    try {
      const commandHandle = 0;
      const stateRes = await createFFICallbackPromise<StateType | number>(
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
            (handle: number, err: number, state: StateType | number) => {
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
