import { Callback } from 'ffi-napi';

import { VCXInternalError } from '../errors';
import { rustAPI } from '../rustlib';
import { createFFICallbackPromise } from '../utils/ffi-helpers';
import { ISerializedData, HolderStateType } from './common';
import { Connection } from './mediated-connection';
import { VCXBaseWithState } from './vcx-base-with-state';

/**
 *    The object of the VCX API representing a Holder side in the credential issuance process.
 *    Assumes that pairwise connection between Issuer and Holder is already established.
 *
 *    # State
 *
 *    The set of object states and transitions depends on communication method is used.
 *    The communication method can be specified as config option on one of *_init function.
 *
 *            VcxStateType::VcxStateRequestReceived - once `vcx_credential_create_with_offer` (create Credential object) is called.
 *
 *            VcxStateType::VcxStateOfferSent - once `vcx_credential_send_request` (send `CredentialRequest` message) is called.
 *
 *            VcxStateType::VcxStateAccepted - once `Credential` messages is received.
 *            VcxStateType::None - once `ProblemReport` messages is received.
 *                                                    use `vcx_credential_update_state` or `vcx_credential_update_state_with_message` functions for state updates.
 *
 *        # Transitions
 *
 *        aries: RFC - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential
 *            VcxStateType::None - `vcx_credential_create_with_offer` - VcxStateType::VcxStateRequestReceived
 *
 *            VcxStateType::VcxStateRequestReceived - `vcx_issuer_send_credential_offer` - VcxStateType::VcxStateOfferSent
 *
 *            VcxStateType::VcxStateOfferSent - received `Credential` - VcxStateType::VcxStateAccepted
 *            VcxStateType::VcxStateOfferSent - received `ProblemReport` - VcxStateType::None
 *
 *        # Messages
 *            CredentialProposal - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential#propose-credential
 *            CredentialOffer - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential#offer-credential
 *            CredentialRequest - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential#request-credential
 *            Credential - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential#issue-credential
 *            ProblemReport - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0035-report-problem#the-problem-report-message-type
 *            Ack - https://github.com/hyperledger/aries-rfcs/tree/master/features/0015-acks#explicit-acks
 */

export interface ICredentialStructData {
  source_id: string;
}

// eslint-disable-next-line @typescript-eslint/ban-types
export type ICredentialOffer = [object, object];

/**
 * @description Interface that represents the parameters for `Credential.create` function.
 * @interface
 */
export interface ICredentialCreateWithOffer {
  // Institution's personal identification for the credential, should be unique.
  sourceId: string;
  // Credential offer received via "getOffers"
  offer: string;
  // We're going to need it in the future
  connection: Connection;
}

/**
 * @description Interface that represents the parameters for `Credential.createWithMsgId` function.
 * @interface
 */
export interface ICredentialCreateWithMsgId {
  // Institution's personal identification for the credential, should be unique.
  sourceId: string;
  // Id of the message that contains the credential offer
  msgId: string;
  // Connection to query for credential offer
  connection: Connection;
}

/**
 * @description Interface that represents the parameters for `Credential.sendRequest` function.
 * @interface
 */
export interface ICredentialSendData {
  // Connection to send credential request
  connection: Connection;
}

export interface ICredentialGetRequestMessageData {
  // Use Connection api (vcx_connection_get_pw_did) with specified connection_handle to retrieve your pw_did
  myPwDid: string;
  // Use Connection api (vcx_connection_get_their_pw_did) with specified connection_handle to retrieve their pw_did
  theirPwDid?: string;
}

/**
 * A Credential Object, which is issued by the issuing party to the prover and stored in the prover's wallet.
 */
export class Credential extends VCXBaseWithState<ICredentialStructData, HolderStateType> {
  /**
   * Creates a credential with an offer.
   *
   * * Requires a credential offer to be submitted to prover.
   *
   * ```
   * credentialOffer = [
   *   {
   *     claim_id: 'defaultCredentialId',
   *     claim_name: 'Credential',
   *     cred_def_id: 'id',
   *     credential_attrs: {
   *     address1: ['101 Tela Lane'],
   *     address2: ['101 Wilson Lane'],
   *     city: ['SLC'],
   *     state: ['UT'],
   *     zip: ['87121']
   *   },
   *   from_did: '8XFh8yBzrpJQmNyZzgoTqB',
   *   libindy_offer: '{}',
   *   msg_ref_id: '123',
   *   msg_type: 'CLAIM_OFFER',
   *   schema_seq_no: 1487,
   *   to_did: '8XFh8yBzrpJQmNyZzgoTqB',
   *   version: '0.1'
   * }]
   *
   * {
   *   JSON.stringify(credentialOffer),
   *   'testCredentialSourceId'
   * }
   * credential = Credential.create(data)
   * ```
   *
   */
  public static async create({ sourceId, offer }: ICredentialCreateWithOffer): Promise<Credential> {
    const credential = new Credential(sourceId);
    try {
      await credential._create((cb) =>
        rustAPI().vcx_credential_create_with_offer(0, sourceId, offer, cb),
      );
      return credential;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  /**
   * Create a Credential object based off of a known message id for a given connection.
   *
   * ```
   * credential = Credential.createWithMsgId({
   *   connection,
   *   msgId: 'testCredentialMsgId',
   *   sourceId: 'testCredentialSourceId'
   * })
   * ```
   */
  public static async createWithMsgId({
    connection,
    sourceId,
    msgId,
  }: ICredentialCreateWithMsgId): Promise<Credential> {
    try {
      return await createFFICallbackPromise<Credential>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_credential_create_with_msgid(
            0,
            sourceId,
            connection.handle,
            msgId,
            cb,
          );
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          Callback(
            'void',
            ['uint32', 'uint32', 'uint32', 'string'],
            (xHandle: number, err: number, handleNum: number, credOffer: string) => {
              if (err) {
                reject(err);
                return;
              }
              const newObj = new Credential(sourceId);
              newObj._setHandle(handleNum);
              newObj._credOffer = credOffer;
              resolve(newObj);
            },
          ),
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  /**
   * Create an object from a JSON Structured data produced from the objects serialize method
   *
   * ```
   * data = credential.deserialize()
   * ```
   */
  public static async deserialize(
    credentialData: ISerializedData<ICredentialStructData>,
  ): Promise<Credential> {
    const credential = await super._deserialize<Credential>(Credential, credentialData);
    return credential;
  }

  /**
   * Retrieves all pending credential offers.
   *
   * ```
   * connection = await Connection.create({id: 'foobar'})
   * inviteDetails = await connection.connect()
   * offers = await Credential.getOffers(connection)
   * ```
   */
  public static async getOffers(connection: Connection): Promise<ICredentialOffer[]> {
    try {
      const offersStr = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_credential_get_offers(0, connection.handle, cb);
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (handle: number, err: number, messages: string) => {
              if (err) {
                reject(err);
                return;
              }
              resolve(messages);
            },
          ),
      );
      const offers: ICredentialOffer[] = JSON.parse(offersStr);
      return offers;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  protected _releaseFn = rustAPI().vcx_credential_release;
  protected _updateStFnV2 = rustAPI().vcx_v2_credential_update_state;
  protected _getStFn = rustAPI().vcx_credential_get_state;
  protected _serializeFn = rustAPI().vcx_credential_serialize;
  protected _deserializeFn = rustAPI().vcx_credential_deserialize;
  protected _credOffer = '';

  /**
   * Approves the credential offer and submits a credential request.
   * The result will be a credential stored in the prover's wallet.
   *
   * ```
   * connection = await Connection.create({id: 'foobar'})
   * inviteDetails = await connection.connect()
   * credential = Credential.create(data)
   * await credential.sendRequest({ connection, 1000 })
   * ```
   *
   */
  public async sendRequest({ connection }: ICredentialSendData): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_credential_send_request(
            0,
            this.handle,
            connection.handle,
            0,
            cb,
          );
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          Callback('void', ['uint32', 'uint32'], (xcommandHandle: number, err: number) => {
            if (err) {
              reject(err);
              return;
            }
            resolve();
          }),
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }
  /**
   * Gets the credential request message for sending to the specified connection.
   *
   * ```
   * connection = await Connection.create({id: 'foobar'})
   * inviteDetails = await connection.connect()
   * credential = Credential.create(data)
   * await credential.getRequestMessage({ '44x8p4HubxzUK1dwxcc5FU', 1000 })
   * ```
   *
   */
  public async getRequestMessage({
    myPwDid,
    theirPwDid,
  }: ICredentialGetRequestMessageData): Promise<string> {
    try {
      return await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_credential_get_request_msg(
            0,
            this.handle,
            myPwDid,
            theirPwDid,
            0,
            cb,
          );
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (xHandle: number, err: number, message: string) => {
              if (err) {
                reject(err);
                return;
              }
              if (!message) {
                reject(`Credential ${this.sourceId} returned empty string`);
                return;
              }
              resolve(message);
            },
          ),
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async getAttributes(): Promise<string> {
    try {
      const attrs = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_credential_get_attributes(0, this.handle, cb);
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (handle: number, err: number, _attrs: string) => {
              if (err) {
                reject(err);
                return;
              }
              resolve(_attrs);
            },
          ),
      );
      return attrs;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async getAttachment(): Promise<string> {
    try {
      const attach = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_credential_get_attachment(0, this.handle, cb);
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (handle: number, err: number, _attach: string) => {
              if (err) {
                reject(err);
                return;
              }
              resolve(_attach);
            },
          ),
      );
      return attach;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async getTailsLocation(): Promise<string> {
    try {
      const location = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_credential_get_tails_location(0, this.handle, cb);
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (handle: number, err: number, _location: string) => {
              if (err) {
                reject(err);
                return;
              }
              resolve(_location);
            },
          ),
      );
      return location;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async getTailsHash(): Promise<string> {
    try {
      const hash = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_credential_get_tails_hash(0, this.handle, cb);
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (handle: number, err: number, _hash: string) => {
              if (err) {
                reject(err);
                return;
              }
              resolve(_hash);
            },
          ),
      );
      return hash;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async getRevRegId(): Promise<string> {
    try {
      const revRegId = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_credential_get_rev_reg_id(0, this.handle, cb);
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (handle: number, err: number, _revRegId: string) => {
              if (err) {
                reject(err);
                return;
              }
              resolve(_revRegId);
            },
          ),
      );
      return revRegId;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async getThreadId(): Promise<string> {
    try {
      const threadId = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_credential_get_thread_id(0, this.handle, cb);
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (handle: number, err: number, threadId: string) => {
              if (err) {
                reject(err);
                return;
              }
              resolve(threadId);
            },
          ),
      );
      return threadId;
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  public async declineOffer(connection: Connection, comment: string): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_credential_decline_offer(
            0,
            this.handle,
            connection.handle,
            comment,
            cb,
          );
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          Callback('void', ['uint32', 'uint32'], (xcommandHandle: number, err: number) => {
            if (err) {
              reject(err);
              return;
            }
            resolve();
          }),
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  get credOffer(): string {
    return this._credOffer;
  }

  protected _setHandle(handle: number): void {
    super._setHandle(handle);
  }
}
