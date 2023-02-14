import * as ffiNapi from '@hyperledger/vcx-napi-rs';
import { VCXInternalError } from '../errors';
import { ISerializedData, ConnectionStateType } from './common';
import { VcxBaseWithState } from './vcx-base-with-state';
import { IPwInfo } from './utils';

export interface IConnectionData {
  source_id: string;
  invite_detail: string;
  handle: number;
  pw_did: string;
  pw_verkey: string;
  did_endpoint: string;
  endpoint: string;
  uuid: string;
  wallet: string;
  state: ConnectionStateType;
}

/**
 * @description Interface that represents the parameters for `Connection.create` function.
 * @interface
 */
export interface IConnectionCreateData {
  // Institution's personal identification for the connection
  id: string;
}

// A string representing a invitation json object.
export type IConnectionInvite = string;

/**
 * @description Interface that represents the parameters for `Connection.createWithInvite` function.
 * @interface
 */
export interface IRecipientInviteInfo extends IConnectionCreateData {
  // Invitation provided by an entity that wishes to make a connection.
  invite: IConnectionInvite;
}

export interface IFromRequestInfoV2 extends IConnectionCreateData {
  pwInfo: IPwInfo;
  request: string;
}

/**
 * @description Interface that represents the parameters for `Connection.sendMessage` function.
 * @interface
 */
export interface IMessageData {
  // Actual message to send
  msg: string;
  // Type of message to send. Can be any string
  type: string;
  // Message title (user notification)
  title: string;
  // If responding to a message, id of the message
  refMsgId?: string;
}

/**
 * @description Interface that represents the parameters for `Connection.verifySignature` function.
 * @interface
 */
export interface ISignatureData {
  // Message was signed
  data: Buffer;
  // Generated signature
  signature: Buffer;
}

/**
 * @description A string representing a connection info json object.
 *      {
 *         "current": {
 *             "did": <str>
 *             "recipientKeys": array<str>
 *             "routingKeys": array<str>
 *             "serviceEndpoint": <str>,
 *             "protocols": array<str> -  The set of protocol supported by current side.
 *         },
 *         "remote: { <Option> - details about remote connection side
 *             "did": <str> - DID of remote side
 *             "recipientKeys": array<str> - Recipient keys
 *             "routingKeys": array<str> - Routing keys
 *             "serviceEndpoint": <str> - Endpoint
 *             "protocols": array<str> - The set of protocol supported by side. Is filled after DiscoveryFeatures process was completed.
 *          }
 *    }
 */
export type IConnectionInfo = string;

export interface IDownloadMessagesConfigsV2 {
  connections: [Connection];
  status: string;
  uids: string;
}

export interface IConnectionDownloadMessages {
  status: string;
  uids: string;
}

export async function downloadMessagesV2({
  connections,
  status,
  uids,
}: IDownloadMessagesConfigsV2): Promise<string> {
  try {
    const handles = connections.map((connection) => connection.handle);
    return await ffiNapi.mediatedConnectionMessagesDownload(handles, status, uids);
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export function generatePublicInvite(public_did: string, label: string): string {
  try {
    return ffiNapi.mediatedConnectionGeneratePublicInvite(public_did, label);
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export class Connection extends VcxBaseWithState<IConnectionData, ConnectionStateType> {
  public static async create({ id }: IConnectionCreateData): Promise<Connection> {
    try {
      const connection = new Connection();
      connection._setHandle(await ffiNapi.mediatedConnectionCreate(id));
      return connection;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public static async createWithInvite({ id, invite }: IRecipientInviteInfo): Promise<Connection> {
    try {
      const connection = new Connection();
      connection._setHandle(await ffiNapi.mediatedConnectionCreateWithInvite(id, invite));
      return connection;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getThreadId(): string {
    try {
      return ffiNapi.mediatedConnectionGetThreadId(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public static async createWithConnectionRequestV2({
    id,
    pwInfo,
    request,
  }: IFromRequestInfoV2): Promise<Connection> {
    try {
      const connection = new Connection();
      connection._setHandle(
        await ffiNapi.mediatedConnectionCreateWithConnectionRequestV2(
          request,
          JSON.stringify(pwInfo),
        ),
      );
      return connection;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public static deserialize(connectionData: ISerializedData<IConnectionData>): Connection {
    try {
      return super._deserialize(Connection, connectionData as any);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  protected _releaseFn = ffiNapi.mediatedConnectionRelease;
  protected _updateStFn = ffiNapi.mediatedConnectionUpdateState;
  protected _updateStFnV2 = async (_handle: number, _connHandle: number): Promise<number> => {
    throw new Error('_updateStFnV2 cannot be called for a Connection object');
  };
  protected _getStFn = ffiNapi.mediatedConnectionGetState;
  protected _serializeFn = ffiNapi.mediatedConnectionSerialize;
  protected _deserializeFn = ffiNapi.mediatedConnectionDeserialize;
  protected _inviteDetailFn = ffiNapi.mediatedConnectionInviteDetails;
  protected _infoFn = ffiNapi.mediatedConnectionInfo;

  public async updateStateWithMessage(message: string): Promise<number> {
    try {
      return await ffiNapi.mediatedConnectionUpdateStateWithMessage(this.handle, message);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async handleMessage(message: string): Promise<void> {
    try {
      return await ffiNapi.mediatedConnectionHandleMessage(this.handle, message);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async updateState(): Promise<number> {
    try {
      return await ffiNapi.mediatedConnectionUpdateState(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async delete(): Promise<void> {
    try {
      await ffiNapi.mediatedConnectionDeleteConnection(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async connect(): Promise<void> {
    try {
      return await ffiNapi.mediatedConnectionConnect(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async sendMessage(msgData: IMessageData): Promise<void> {
    try {
      return await ffiNapi.mediatedConnectionSendMessage(
        this.handle,
        msgData.msg,
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async sendHandshakeReuse(oobMsg: string): Promise<void> {
    try {
      return await ffiNapi.mediatedConnectionSendHandshakeReuse(this.handle, oobMsg);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async signData(data: Buffer): Promise<Buffer> {
    try {
      return await ffiNapi.mediatedConnectionSignData(this.handle, data);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async verifySignature(signatureData: ISignatureData): Promise<boolean> {
    try {
      return await ffiNapi.mediatedConnectionVerifySignature(
        this.handle,
        signatureData.data,
        signatureData.signature,
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public inviteDetails(): IConnectionInvite {
    try {
      return ffiNapi.mediatedConnectionInviteDetails(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async sendPing(comment: string | null | undefined): Promise<void> {
    try {
      return await ffiNapi.mediatedConnectionSendPing(this.handle, comment);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async sendDiscoveryFeatures(
    query: string | null | undefined,
    comment: string | null | undefined,
  ): Promise<void> {
    try {
      return await ffiNapi.mediatedConnectionSendDiscoveryFeatures(this.handle, query, comment);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getPwDid(): string {
    try {
      return ffiNapi.mediatedConnectionGetPwDid(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public getTheirDid(): string {
    try {
      return ffiNapi.mediatedConnectionGetTheirPwDid(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async info(): Promise<IConnectionInfo> {
    try {
      return await ffiNapi.mediatedConnectionInfo(this.handle);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async downloadMessages({ status, uids }: IConnectionDownloadMessages): Promise<string> {
    try {
      return await ffiNapi.mediatedConnectionMessagesDownload([this.handle], status, uids);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }
}
