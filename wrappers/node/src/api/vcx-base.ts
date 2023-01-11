import { VCXInternalError } from '../errors';
import { ISerializedData } from './common';
import {GCWatcher} from "../utils/gc-watcher";

export abstract class VcxBase<SerializedData> extends GCWatcher {

  protected static _deserialize<T extends VcxBase<unknown>, P = unknown> (
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    VCXClass: new (sourceId: string, args?: any) => T,
    objData: ISerializedData<{ source_id: string }>,
    constructorParams?: P,
  ): T {
    try {
      const obj = new VCXClass(objData.source_id || objData.data.source_id, constructorParams);
      obj._initFromData(objData);
      return obj;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  protected abstract _serializeFn: (handle: number) => string;
  protected abstract _deserializeFn: (data: string) => number;
  protected _sourceId: string;

  constructor(sourceId: string) {
    super();
    this._sourceId = sourceId;
  }

  public serialize(): ISerializedData<SerializedData> {
    try {
      return JSON.parse(this._serializeFn(this.handle));
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  get sourceId(): string {
    return this._sourceId;
  }

  private _initFromData(objData: ISerializedData<{ source_id: string }>): void {
    const objHandle = this._deserializeFn(JSON.stringify(objData))
    this._setHandle(objHandle);
  }
}
