import { VCXInternalError1 } from '../errors-1';
import { ISerializedData } from './common';

export abstract class VCXBase1<SerializedData> {
  private _handleRef!: number;

  protected static _deserialize<T extends VCXBase1<unknown>, P = unknown>(
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
      throw new VCXInternalError1(err);
    }
  }

  protected abstract _serializeFn: (handle: number) => string;
  protected abstract _deserializeFn: (data: string) => number;
  protected _sourceId: string;

  constructor(sourceId: string) {
    this._sourceId = sourceId;
  }

  public serialize(): ISerializedData<SerializedData> {
    try {
      return JSON.parse(this._serializeFn(this.handle));
    } catch (err: any) {
      throw new VCXInternalError1(err);
    }
  }

  /** The source Id assigned by the user for this object */
  get sourceId(): string {
    return this._sourceId;
  }

  private _initFromData(objData: ISerializedData<{ source_id: string }>): void {
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
