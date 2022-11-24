import * as ffiNapi from 'node-napi-rs';
import { Callback } from 'ffi-napi';

import { VCXInternalError } from '../errors';
import { rustAPI } from '../rustlib';
import { createFFICallbackPromise } from '../utils/ffi-helpers';

export function initThreadpool (config: object) {
  const rc = rustAPI().vcx_init_threadpool(JSON.stringify(config))
  if (rc !== 0) {
    throw new VCXInternalError(rc)
  }
}

export async function createAgencyClientForMainWallet (config: object): Promise<void> {
  try {
    ffiNapi.createAgencyClientForMainWallet(JSON.stringify(config))
  } catch (err: any) {
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
  } catch (err: any) {
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
  } catch (err: any) {
    throw new VCXInternalError(err)
  }
}

export function enableMocks(): void {
    return ffiNapi.enableMocks()
}
