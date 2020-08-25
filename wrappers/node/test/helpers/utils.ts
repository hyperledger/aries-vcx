import '../module-resolver-helper'

import * as vcx from 'src'

const testConfig = {
  agency_verkey: 'FuN98eH2eZybECWkofW6A9BKJxxnTatBCopfUiNxo6ZB',
  remote_to_sdk_verkey: 'FuN98eH2eZybECWkofW6A9BKJxxnTatBCopfUiNxo6ZB',
  // tslint:disable-next-line:object-literal-sort-keys
  link_secret_alias: 'main',
  protocol_version: '2',
  exported_wallet_path: '/var/folders/tz/wkm7f1z147qgkp8p33sf8n3r0000gp/T/wallet.txn',
  threadpool_size: '8',
  use_latest_protocols: 'false',
  enable_test_mode: 'true',
  backup_key: 'backup_wallet_key',
  wallet_type: 'default',
  wallet_name: 'LIBVCX_SDK_WALLET',
  payment_method: 'null',
  institution_logo_url: 'http://127.0.0.1:8080',
  pool_name: 'pool1',
  institution_name: 'default',
  agency_did: '2hoqvcwupRTUNkXn6ArYzs',
  institution_did: '2hoqvcwupRTUNkXn6ArYzs',
  sdk_to_remote_did: '2hoqvcwupRTUNkXn6ArYzs',
  remote_to_sdk_did: '2hoqvcwupRTUNkXn6ArYzs',
  sdk_to_remote_verkey: 'FuN98eH2eZybECWkofW6A9BKJxxnTatBCopfUiNxo6ZB',
  wallet_key: '********',
  sdk_to_remote_role: '0',
  wallet_key_derivation: 'RAW',
  agency_endpoint: 'http://127.0.0.1:8080'
}

export async function initVcxTestMode (protocolType: string) {
  const useTestConfig = { ...testConfig, protocol_type: protocolType }
  await vcx.initVcxWithConfig(JSON.stringify(useTestConfig))
  const rustLogPattern = process.env.RUST_LOG || 'vcx=error'
  await vcx.defaultLogger(rustLogPattern)
  console.debug(`Vcx initialized.`)
}

export const shouldThrow = (fn: () => any): Promise<vcx.VCXInternalError> => new Promise(async (resolve, reject) => {
  try {
    await fn()
    reject(new Error(`${fn.toString()} should have thrown!`))
  } catch (e) {
    resolve(e)
  }
})

export const sleep = (timeout: number) => new Promise((resolve, reject) => setTimeout(resolve, timeout))
