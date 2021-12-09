import { Callback } from 'ffi-napi';

import { VCXInternalError } from '../errors'
import { rustAPI } from '../rustlib'
import { IConnectionDownloadAllMessages } from './connection'
import { createFFICallbackPromise } from '../utils/ffi-helpers'

export async function provisionCloudAgent (configAgent: object): Promise<string> {
  try {
    return await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_provision_cloud_agent(0, JSON.stringify(configAgent), cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32','string'],
        (xhandle: number, err: number, config: string) => {
          if (err) {
            reject(err)
            return
          }
          resolve(config)
        })
    )
  } catch (err) {
    throw new VCXInternalError(err)
  }
}

export interface PtrBuffer extends Buffer {
  // Buffer.deref typing provided by @types/ref-napi is wrong, so we overwrite the typing/
  // An issue is currently dealing with fixing it https://github.com/DefinitelyTyped/DefinitelyTyped/pull/44004#issuecomment-744497037
  deref: () => PtrBuffer;
}

export function getVersion(): string {
  return rustAPI().vcx_version();
}

export async function getLedgerAuthorAgreement(): Promise<string> {
  /**
   * Retrieve author agreement set on the sovrin network
   */
  try {
    const agreement = await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_get_ledger_author_agreement(0, cb);
        if (rc) {
          reject(rc);
        }
      },
      (resolve, reject) =>
        Callback(
          'void',
          ['uint32', 'uint32', 'string'],
          (xhandle: number, err: number, agreement: string) => {
            if (err) {
              reject(err);
              return;
            }
            resolve(agreement);
          },
        ),
    );
    return agreement;
  } catch (err) {
    throw new VCXInternalError(err);
  }
}

export function setActiveTxnAuthorAgreementMeta(
  text: string | null | undefined,
  version: string | null | undefined,
  hash: string | null | undefined,
  acc_mech_type: string,
  time_of_acceptance: number,
): number {
  /**
   * Set some accepted agreement as active.
   * As result of successful call of this function appropriate metadata will be appended to each write request.
   */
  return rustAPI().vcx_set_active_txn_author_agreement_meta(
    text,
    version,
    hash,
    acc_mech_type,
    time_of_acceptance,
  );
}

export function shutdownVcx(deleteWallet: boolean): number {
  return rustAPI().vcx_shutdown(deleteWallet);
}

export interface IUpdateWebhookUrl {
  webhookUrl: string;
}

export async function vcxUpdateWebhookUrl({ webhookUrl }: IUpdateWebhookUrl): Promise<void> {
  try {
    return await createFFICallbackPromise<void>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_update_webhook_url(0, webhookUrl, cb);
        if (rc) {
          reject(rc);
        }
      },
      (resolve, reject) =>
        Callback('void', ['uint32', 'uint32'], (xhandle: number, err: number) => {
          if (err) {
            reject(err);
            return;
          }
          resolve();
        }),
    );
  } catch (err) {
    throw new VCXInternalError(err);
  }
}

export interface IUpdateMessagesConfigs {
  msgJson: string;
}

export async function updateMessages({ msgJson }: IUpdateMessagesConfigs): Promise<number> {
  /**
   * Update the status of messages from the specified connection
   */
  try {
    return await createFFICallbackPromise<number>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_messages_update_status(0, 'MS-106', msgJson, cb);
        if (rc) {
          reject(rc);
        }
      },
      (resolve, reject) =>
        Callback('void', ['uint32', 'uint32'], (xhandle: number, err: number) => {
          if (err) {
            reject(err);
            return;
          }
          resolve(err);
        }),
    );
  } catch (err) {
    throw new VCXInternalError(err);
  }
}

export function setPoolHandle(handle: number): void {
  rustAPI().vcx_pool_set_handle(handle);
}

export async function endorseTransaction(transaction: string): Promise<void> {
  /**
   * Endorse transaction to the ledger preserving an original author
   */
  try {
    return await createFFICallbackPromise<void>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_endorse_transaction(0, transaction, cb);
        if (rc) {
          reject(rc);
        }
      },
      (resolve, reject) =>
        Callback('void', ['uint32', 'uint32'], (xhandle: number, err: number) => {
          if (err) {
            reject(err);
            return;
          }
          resolve();
        }),
    );
  } catch (err) {
    throw new VCXInternalError(err);
  }
}

export async function downloadAllMessages({ status, uids, pwdids }: IConnectionDownloadAllMessages): Promise<string> {
  try {
    return await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_messages_download(0, status, uids, pwdids, cb);
        if (rc) {
          reject(rc);
        }
      },
      (resolve, reject) =>
        Callback(
          'void',
          ['uint32', 'uint32', 'string'],
          (xhandle: number, err: number, messages: string) => {
            if (err) {
              reject(err);
              return;
            }
            resolve(messages);
          },
        ),
    );
  } catch (err) {
    throw new VCXInternalError(err);
  }
}
