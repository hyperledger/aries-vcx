import * as ffi from '@hyperledger/vcx-napi-rs';
import { ISerializedData, HolderStateType } from './common';
import { Connection } from './mediated-connection';
import { VcxBaseWithState } from './vcx-base-with-state';
import { VCXInternalError } from '../errors';

export interface ICredentialStructData {
  source_id: string;
}

// eslint-disable-next-line @typescript-eslint/ban-types
export type ICredentialOffer = [object, object];

export interface ICredentialCreateWithOffer {
  sourceId: string;
  offer: string;
  connection: Connection;
}

export interface ICredentialSendData {
  connection: Connection;
}

export class Credential extends VcxBaseWithState<ICredentialStructData, HolderStateType> {
  public static create({ sourceId, offer }: ICredentialCreateWithOffer): Credential {
    try {
      const credential = new Credential();
      const handle = ffi.credentialCreateWithOffer(sourceId, offer);
      credential._setHandle(handle);
      return credential;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public static deserialize(
    credentialData: ISerializedData<ICredentialStructData>,
  ): Credential {
    // running into issue https://github.com/microsoft/TypeScript/issues/15300 without using "as any"
    const credential = super._deserialize<Credential>(Credential, credentialData as any);
    return credential;
  }

  protected _releaseFn = ffi.credentialRelease;
  protected _updateStFnV2 = ffi.v2CredentialUpdateState;
  protected _getStFn = ffi.credentialGetState;
  protected _serializeFn = ffi.credentialSerialize;
  protected _deserializeFn = ffi.credentialDeserialize;

  public static async getOffers(connection: Connection): Promise<ICredentialOffer[]> {
    try {
      const offersStr = await ffi.credentialGetOffers(connection.handle);
      const offers: ICredentialOffer[] = JSON.parse(offersStr);
      return offers;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async sendRequest({ connection }: ICredentialSendData): Promise<void> {
    try {
      return await ffi.credentialSendRequest(this.handle, connection.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getAttributes(): string {
    try {
      return ffi.credentialGetAttributes(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getAttachment(): string {
    try {
      return ffi.credentialGetAttachment(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getTailsLocation(): string {
    try {
      return ffi.credentialGetTailsLocation(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getTailsHash(): string {
    try {
      return ffi.credentialGetTailsHash(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getRevRegId(): string {
    try {
      return ffi.credentialGetRevRegId(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getThreadId(): string {
    try {
      return ffi.credentialGetThreadId(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async declineOffer(connection: Connection, comment: string) {
    try {
      await ffi.credentialDeclineOffer(this.handle, connection.handle, comment);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  protected _setHandle(handle: number): void {
    super._setHandle(handle);
  }
}
