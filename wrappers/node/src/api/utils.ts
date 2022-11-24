import { Callback } from 'ffi-napi';

import { VCXInternalError } from '../errors'
import { rustAPI } from '../rustlib'
import { IConnectionDownloadAllMessages } from './mediated-connection'
import { createFFICallbackPromise } from '../utils/ffi-helpers'
import * as ref from 'ref-napi';

export interface IPwInfo {
  pw_did: string;
  pw_vk: string;
}

export interface IMsgUnpacked {
  sender_verkey: string;
  message: string;
}

export interface IAriesService {
  id: string;
  type: string;
  priority: number;
  recipientKeys: string[];
  routingKeys: string[];
  serviceEndpoint: string;
}

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
  } catch (err: any) {
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
  } catch (err: any) {
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
  } catch (err: any) {
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
  } catch (err: any) {
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
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function rotateVerkey(did: string): Promise<void> {
  try {
    return await createFFICallbackPromise<void>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_rotate_verkey(0, did, cb);
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
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function rotateVerkeyStart(did: string): Promise<string> {
  try {
    return await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_rotate_verkey_start(0, did, cb);
        if (rc) {
          reject(rc);
        }
      },
      (resolve, reject) =>
        Callback('string', ['uint32', 'uint32', 'string'], (xhandle: number, err: number, tempVk: string) => {
          if (err) {
            reject(err);
            return;
          }
          resolve(tempVk);
        }),
    );
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function rotateVerkeyApply(did: string, tempVk: string): Promise<void> {
  try {
    return await createFFICallbackPromise<void>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_rotate_verkey_apply(0, did, tempVk, cb);
        if (rc) {
          reject(rc);
        }
      },
      (resolve, reject) =>
        Callback('string', ['uint32', 'uint32'], (xhandle: number, err: number) => {
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

export async function getVerkeyFromWallet(did: string): Promise<string> {
  try {
    return await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_get_verkey_from_wallet(0, did, cb);
        if (rc) {
          reject(rc);
        }
      },
      (resolve, reject) =>
        Callback('void', ['uint32', 'uint32', 'string'], (xhandle: number, err: number, vk: string) => {
          if (err) {
            reject(err);
            return;
          }
          resolve(vk);
        }),
    );
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function getVerkeyFromLedger(did: string): Promise<string> {
  try {
    return await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_get_verkey_from_ledger(0, did, cb);
        if (rc) {
          reject(rc);
        }
      },
      (resolve, reject) =>
        Callback('void', ['uint32', 'uint32', 'string'], (xhandle: number, err: number, vk: string) => {
          if (err) {
            reject(err);
            return;
          }
          resolve(vk);
        }),
    );
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function getLedgerTxn(did: string, seqNo: number): Promise<string> {
  try {
    return await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_get_ledger_txn(0, did, seqNo, cb);
        if (rc) {
          reject(rc);
        }
      },
      (resolve, reject) =>
        Callback('void', ['uint32', 'uint32', 'string'], (xhandle: number, err: number, txn: string) => {
          if (err) {
            reject(err);
            return;
          }
          resolve(txn);
        }),
    );
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function createPwInfo(): Promise<IPwInfo> {
  try {
    return await createFFICallbackPromise<IPwInfo>(
      (_resolve, reject, cb) => {
        const rc = rustAPI().vcx_create_pairwise_info(0, cb);
        if (rc) {
          reject(rc);
        }
      },
      (resolve, reject) =>
        Callback('void', ['uint32', 'uint32', 'string'], (_xhandle: number, err: number, pwInfo: string) => {
          if (err) {
            reject(err);
            return;
          }
          resolve(JSON.parse(pwInfo));
        }),
    );
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function createService(
  did: string, endpoint: string, recipientKeys: string[], routingKeys: string[]
): Promise<IAriesService> {
  try {
    return await createFFICallbackPromise<IAriesService>(
      (_resolve, reject, cb) => {
        const rc = rustAPI()
          .vcx_create_service(0, did, endpoint, JSON.stringify(recipientKeys), JSON.stringify(routingKeys), cb);
        if (rc) {
          reject(rc);
        }
      },
      (resolve, reject) =>
        Callback('void', ['uint32', 'uint32', 'string'], (_xhandle: number, err: number, service: string) => {
          if (err) {
            reject(err);
            return;
          }
          resolve(JSON.parse(service));
        }),
    );
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function getServiceFromLedger (did: string): Promise<IAriesService> {
  try {
    return await createFFICallbackPromise<IAriesService>(
      (_resolve, reject, cb) => {
        const rc = rustAPI()
          .vcx_get_service_from_ledger(0, did, cb);
        if (rc) {
          reject(rc);
        }
      },
      (resolve, reject) =>
        Callback('void', ['uint32', 'uint32', 'string'], (_xhandle: number, err: number, service: string) => {
          if (err) {
            reject(err);
            return;
          }
          resolve(JSON.parse(service));
        }),
    );
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function unpack(payload: Buffer): Promise<IMsgUnpacked> {
  try {
    return await createFFICallbackPromise<IMsgUnpacked>(
      (_resolve, reject, cb) => {
        const rc = rustAPI()
          .vcx_unpack(0, ref.address(payload), payload.length, cb);
        if (rc) {
          reject(rc);
        }
      },
      (resolve, reject) =>
        Callback('void', ['uint32', 'uint32', 'string'], (_xhandle: number, err: number, decryptedPayload: string) => {
          if (err) {
            reject(err);
            return;
          }
          resolve(JSON.parse(decryptedPayload));
        }),
    );
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}
