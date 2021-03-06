import { Callback } from 'ffi-napi';

import { VCXInternalError } from '../errors';
import { rustAPI } from '../rustlib';
import { createFFICallbackPromise } from '../utils/ffi-helpers';

export async function initThreadpool (config: object) {
  const rc = rustAPI().vcx_init_threadpool(JSON.stringify(config))
  if (rc !== 0) {
    throw new VCXInternalError(rc)
  }
}

export async function createAgencyClientForMainWallet (config: object): Promise<void> {
  try {
    return await createFFICallbackPromise<void>(
            (resolve, reject, cb) => {
              const rc = rustAPI().vcx_create_agency_client_for_main_wallet(0, JSON.stringify(config), cb)
              if (rc) {
                reject(rc)
              }
            },
            (resolve, reject) => Callback(
                'void',
                ['uint32', 'uint32'],
                (xhandle: number, err: number) => {
                  if (err) {
                    reject(err)
                    return
                  }
                  resolve()
                })
        )
  } catch (err) {
    throw new VCXInternalError(err)
  }
}

export async function initIssuerConfig (config: object): Promise<void> {
  try {
    return await createFFICallbackPromise<void>(
            (resolve, reject, cb) => {
              const rc = rustAPI().vcx_init_issuer_config(0, JSON.stringify(config), cb)
              if (rc) {
                reject(rc)
              }
            },
            (resolve, reject) => Callback(
                'void',
                ['uint32', 'uint32'],
                (xhandle: number, err: number) => {
                  if (err) {
                    reject(err)
                    return
                  }
                  resolve()
                })
        )
  } catch (err) {
    throw new VCXInternalError(err)
  }
}

export async function openMainPool (config: object): Promise<void> {
  try {
    return await createFFICallbackPromise<void>(
            (resolve, reject, cb) => {
              const rc = rustAPI().vcx_open_main_pool(0, JSON.stringify(config), cb)
              if (rc) {
                reject(rc)
              }
            },
            (resolve, reject) => Callback(
                'void',
                ['uint32', 'uint32'],
                (xhandle: number, err: number) => {
                  if (err) {
                    reject(err)
                    return
                  }
                  resolve()
                })
        )
  } catch (err) {
    throw new VCXInternalError(err)
  }
}

/**
 * Initializes VCX memory with provided configuration
 *
 * Example:
 * ```
 * const config = {
 *   "agency_did": "VsKV7grR1BUE29mG2Fm2kX",
 *   "agency_verkey": "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR",
 *   "agency_endpoint": "http://localhost:8080",
 *   "genesis_path":"/var/lib/indy/verity-staging/pool_transactions_genesis",
 *   "institution_name": "institution",
 *   "institution_did": "EwsFhWVoc3Fwqzrwe998aQ",
 *   "institution_verkey": "8brs38hPDkw5yhtzyk2tz7zkp8ijTyWnER165zDQbpK6",
 *   "remote_to_sdk_did": "EtfeMFytvYTKnWwqTScp9D",
 *   "remote_to_sdk_verkey": "8a7hZDyJK1nNCizRCKMr4H4QbDm8Gg2vcbDRab8SVfsi",
 *   "sdk_to_remote_did": "KacwZ2ndG6396KXJ9NDDw6",
 *   "sdk_to_remote_verkey": "B8LgZGxEPcpTJfZkeqXuKNLihM1Awm8yidqsNwYi5QGc"
 *  }
 * await initVcxCore(config)
 * ```
 */

export async function initVcxCore(config: string): Promise<void> {
  const rc = rustAPI().vcx_init_core(config);
  if (rc !== 0) {
    throw new VCXInternalError(rc);
  }
}

/**
 * Opens wallet using information provided via initVcxCore
 */
export async function openVcxWallet(): Promise<void> {
  try {
    return await createFFICallbackPromise<void>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_open_wallet(0, cb);
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

/**
 * Opens pool connection using information provided via initVcxCore
 */
export async function openVcxPool(): Promise<void> {
  try {
    return await createFFICallbackPromise<void>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_open_pool(0, cb);
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
