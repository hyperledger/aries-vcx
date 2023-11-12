import { VCXInternalError } from '../errors';
import { ISerializedData } from './common';
import { GCWatcher } from '../utils/gc-watcher';

export abstract class VcxBase<SerializedData> extends GCWatcher {
  protected static _deserialize<T extends VcxBase<unknown>, P = unknown>(
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    VCXClass: new (args?: any) => T,
    serializedData: Record<string, unknown>, // this represents "any JSON object"
    constructorParams?: P,
  ): T {
    try {
      const instance = new VCXClass(constructorParams);
      instance._initFromData(serializedData);
      return instance;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  protected abstract _serializeFn: (handle: number) => string;
  protected abstract _deserializeFn: (data: string) => number;

  public serialize(): ISerializedData<SerializedData> {
    try {
      return JSON.parse(this._serializeFn(this.handle));
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  private _initFromData(serializedData: Record<string, unknown>): void {
    const objHandle = this._deserializeFn(JSON.stringify(serializedData));
    this._setHandle(objHandle);
  }
}
