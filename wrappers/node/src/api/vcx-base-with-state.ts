import * as ffi from 'ffi-napi';
import { VCXInternalError } from '../errors';
import { createFFICallbackPromise, ICbRef } from '../utils/ffi-helpers';
import { StateType } from './common';
import { Connection } from './connection';
import { VCXBase } from './vcx-base';

export abstract class VCXBaseWithState<SerializedData> extends VCXBase<SerializedData> {
  protected abstract _updateStFn: (commandHandle: number, handle: number, cb: ICbRef) => number;
  protected abstract _updateStFnV2: (
    commandHandle: number,
    handle: number,
    connHandle: number,
    cb: ICbRef,
  ) => number;
  protected abstract _updateStWithMessageFn: (
    commandHandle: number,
    handle: number,
    message: string,
    cb: ICbRef,
  ) => number;
  protected abstract _getStFn: (commandHandle: number, handle: number, cb: ICbRef) => number;

  /**
   *
   * Communicates with the agent service for polling and setting the state of the entity.
   *
   * Example:
   * ```
   * await object.updateState()
   * ```
   * @returns {Promise<void>}
   */
  public async updateState(): Promise<number> {
    try {
      const commandHandle = 0;
      const state = await createFFICallbackPromise<number>(
        (resolve, reject, cb) => {
          const rc = this._updateStFn(commandHandle, this.handle, cb);
          if (rc) {
            resolve(StateType.None);
          }
        },
        (resolve, reject) =>
          ffi.Callback(
            'void',
            ['uint32', 'uint32', 'uint32'],
            (handle: number, err: any, _state: StateType) => {
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

  public async updateStateV2(connection: Connection): Promise<number> {
    try {
      const commandHandle = 0;
      const state = await createFFICallbackPromise<number>(
        (resolve, reject, cb) => {
          const rc = this._updateStFnV2(commandHandle, this.handle, connection.handle, cb);
          if (rc) {
            resolve(StateType.None);
          }
        },
        (resolve, reject) =>
          ffi.Callback(
            'void',
            ['uint32', 'uint32', 'uint32'],
            (handle: number, err: any, _state: StateType) => {
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
   *
   * Communicates with the agent service for polling and setting the state of the entity.
   *
   * Example:
   * ```
   * await object.updateState()
   * ```
   * @returns {Promise<void>}
   */
  public async updateStateWithMessage(message: string): Promise<number> {
    try {
      const commandHandle = 0;
      const state = await createFFICallbackPromise<number>(
        (resolve, reject, cb) => {
          const rc = this._updateStWithMessageFn(commandHandle, this.handle, message, cb);
          if (rc) {
            resolve(StateType.None);
          }
        },
        (resolve, reject) =>
          ffi.Callback(
            'void',
            ['uint32', 'uint32', 'uint32'],
            (handle: number, err: any, _state: StateType) => {
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
   * @returns {Promise<StateType>}
   */
  public async getState(): Promise<StateType> {
    try {
      const commandHandle = 0;
      const stateRes = await createFFICallbackPromise<StateType>(
        (resolve, reject, cb) => {
          const rc = this._getStFn(commandHandle, this.handle, cb);
          if (rc) {
            resolve(StateType.None);
          }
        },
        (resolve, reject) =>
          ffi.Callback(
            'void',
            ['uint32', 'uint32', 'uint32'],
            (handle: number, err: number, state: StateType) => {
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
