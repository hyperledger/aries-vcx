import * as ffi from 'ffi-napi';
import { VCXInternalError } from '../errors';
import { createFFICallbackPromise, ICbRef } from '../utils/ffi-helpers';
import { ISerializedData } from './common';

export type IVCXBaseCreateFn = (cb: ICbRef) => number;

export abstract class VCXBase1<SerializedData> {
  private _handleRef!: number;

  protected static async _deserialize<T extends VCXBase1<unknown>, P = unknown>(
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    VCXClass: new (sourceId: string, args?: any) => T,
    objData: ISerializedData<{ source_id: string }>,
    constructorParams?: P,
  ): Promise<T> {
    try {
      const obj = new VCXClass(objData.source_id || objData.data.source_id, constructorParams);
      await obj._initFromData(objData);
      return obj;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  protected abstract _serializeFn: (handle: number) => string;
  protected abstract _deserializeFn: (data: string) => number;
  protected _sourceId: string;

  constructor(sourceId: string) {
    this._sourceId = sourceId;
  }

  /**
   *
   * Data returned can be used to recreate an entity by passing it to the deserialize function.
   *
   * Same json object structure that is passed to the deserialize function.
   *
   * Example:
   *
   * ```
   *  data = await object.serialize()
   * ```
   */
  public async serialize(): Promise<ISerializedData<SerializedData>> {
    try {
      return JSON.parse(this._serializeFn(this.handle));
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  /** The source Id assigned by the user for this object */
  get sourceId(): string {
    return this._sourceId;
  }

  protected async _create(createFn: IVCXBaseCreateFn): Promise<void> {
    const handleRes = await createFFICallbackPromise<number>(
      (resolve, reject, cb) => {
        const rc = createFn(cb);
        if (rc) {
          reject(rc);
        }
      },
      (resolve, reject) =>
        ffi.Callback(
          'void',
          ['uint32', 'uint32', 'uint32'],
          (xHandle: number, err: number, handle: number) => {
            if (err) {
              reject(err);
              return;
            }
            resolve(handle);
          },
        ),
    );
    this._setHandle(handleRes);
  }

  private async _initFromData(objData: ISerializedData<{ source_id: string }>): Promise<void> {
    const objHandle = this._deserializeFn(JSON.stringify(objData))
    this._setHandle(objHandle);
  }

  protected _setHandle(handle: number): void {
    this._handleRef = handle;
  }

  get handle(): number {
    return this._handleRef;
  }
}
