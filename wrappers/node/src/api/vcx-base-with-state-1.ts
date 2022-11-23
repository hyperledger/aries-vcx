import { VCXInternalError } from '../errors';
import { Connection } from './mediated-connection';
import { VCXBase1 } from './vcx-base-1';

export abstract class VCXBaseWithState1<SerializedData, StateType> extends VCXBase1<SerializedData> {
  protected abstract _updateStFnV2: (
    handle: number,
    connHandle: number,
  ) => StateType;
  protected abstract _getStFn: (handle: number) => StateType;

  public async updateStateV2(connection: Connection): Promise<StateType> {
    try {
      return await this._updateStFnV2(this.handle, connection.handle);
    } catch (err: any) {
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
      return await this._getStFn(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }
}
