import * as ffi from '@hyperledger/vcx-napi-rs';
import { ISerializedData, IssuerStateType } from './common';
import { Connection } from './mediated-connection';
import { NonmediatedConnection } from './connection';
import { CredentialDef } from './credential-def';
import { RevocationRegistry } from './revocation-registry';
import { VcxBaseWithState } from './vcx-base-with-state';
import { VCXInternalError } from '../errors';

export interface IIssuerCredentialBuildOfferDataV2 {
  credDef: CredentialDef;
  revReg?: RevocationRegistry;
  attr: {
    [index: string]: string;
  };
  comment?: string;
}

/**
 * Interface that represents the attributes of an Issuer credential object.
 * This interface is expected as the type for deserialize's parameter and serialize's return value
 */
export interface IIssuerCredentialData {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  issuer_sm: Record<string, any>;
  source_id: string;
}

/**
 * A Credential created by the issuing party (institution)
 */
export class IssuerCredential extends VcxBaseWithState<IIssuerCredentialData, IssuerStateType> {
  public static async create(sourceId: string): Promise<IssuerCredential> {
    try {
      const connection = new IssuerCredential();
      connection._setHandle(ffi.issuerCredentialCreate(sourceId));
      return connection;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public static deserialize(
    serializedData: ISerializedData<IIssuerCredentialData>,
  ): IssuerCredential {
    try {
      return super._deserialize(IssuerCredential, serializedData as any);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  protected _releaseFn = ffi.issuerCredentialRelease;
  protected _updateStFnV2 = ffi.issuerCredentialUpdateStateV2;
  protected _getStFn = ffi.issuerCredentialGetState;
  protected _serializeFn = ffi.issuerCredentialSerialize;
  protected _deserializeFn = ffi.issuerCredentialDeserialize;

  public async updateStateWithMessage(connection: Connection, message: string): Promise<number> {
    try {
      return await ffi.issuerCredentialUpdateStateWithMessageV2(
        this.handle,
        connection.handle,
        message,
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async updateStateWithMessageNonmediated(
    connection: NonmediatedConnection,
    message: string,
  ): Promise<number> {
    try {
      return await ffi.issuerCredentialUpdateStateWithMessageNonmediated(
        this.handle,
        connection.handle,
        message,
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async sendOfferV2(connection: Connection): Promise<void> {
    try {
      return await ffi.issuerCredentialSendOfferV2(this.handle, connection.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async sendOfferNonmediated(connection: NonmediatedConnection): Promise<void> {
    try {
      return await ffi.issuerCredentialSendOfferNonmediated(this.handle, connection.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async markCredentialOfferMsgSent(): Promise<void> {
    try {
      return ffi.issuerCredentialMarkOfferMsgSent(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async buildCredentialOfferMsgV2({
    credDef,
    attr,
    revReg,
    comment,
  }: IIssuerCredentialBuildOfferDataV2): Promise<void> {
    try {
      return await ffi.issuerCredentialBuildOfferMsgV2(
        this.handle,
        credDef.handle,
        revReg?.handle || 0,
        JSON.stringify(attr),
        comment || '',
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getCredentialOfferMsg(): string {
    try {
      return ffi.issuerCredentialGetOfferMsg(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getThreadId(): string {
    try {
      return ffi.issuerCredentialGetThreadId(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async sendCredential(connection: Connection): Promise<number> {
    try {
      return await ffi.issuerCredentialSendCredential(this.handle, connection.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async sendCredentialNonmediated(connection: NonmediatedConnection): Promise<number> {
    try {
      return await ffi.issuerCredentialSendCredentialNonmediated(this.handle, connection.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async revokeCredentialLocal() {
    try {
      return await ffi.issuerCredentialRevokeLocal(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public isRevokable(): boolean {
    try {
      return ffi.issuerCredentialIsRevokable(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getRevRegId(): string {
    try {
      return ffi.issuerCredentialGetRevRegId(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getRevRevocationId(): string {
    try {
      return ffi.issuerCredentialGetRevocationId(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  protected _setHandle(handle: number): void {
    super._setHandle(handle);
  }
}
