import { assert } from 'chai'
import { initRustAPI } from 'src'
import * as vcx from 'src'
import * as uuid from 'uuid'
import '../module-resolver-helper'

const testConfig = {
  agency_verkey: 'FuN98eH2eZybECWkofW6A9BKJxxnTatBCopfUiNxo6ZB',
  remote_to_sdk_verkey: 'FuN98eH2eZybECWkofW6A9BKJxxnTatBCopfUiNxo6ZB',
  // tslint:disable-next-line:object-literal-sort-keys
  link_secret_alias: 'main',
  protocol_version: '2',
  exported_wallet_path: '/var/folders/libvcx_nodetest/sample.wallet',
  threadpool_size: '8',
  use_latest_protocols: 'false',
  enable_test_mode: 'true',
  backup_key: 'backup_wallet_key',
  wallet_type: 'default',
  wallet_name: 'LIBVCX_SDK_WALLET',
  payment_method: 'null',
  pool_name: 'pool1',
  institution_name: 'default',
  agency_did: '2hoqvcwupRTUNkXn6ArYzs',
  institution_did: '2hoqvcwupRTUNkXn6ArYzs',
  sdk_to_remote_did: '2hoqvcwupRTUNkXn6ArYzs',
  remote_to_sdk_did: '2hoqvcwupRTUNkXn6ArYzs',
  sdk_to_remote_verkey: 'FuN98eH2eZybECWkofW6A9BKJxxnTatBCopfUiNxo6ZB',
  wallet_key: '********',
  wallet_key_derivation: 'RAW',
  agency_endpoint: 'http://127.0.0.1:8080'
}

function generateTestConfig () {
  const sampleConfig = { ...testConfig }
  const configId = uuid.v4()
  sampleConfig.wallet_name = `testnodejs_${configId}`
  sampleConfig.exported_wallet_path = `/var/folders/libvcx_nodetest/wallet_${configId}.wallet`
  return sampleConfig
}

export async function initVcxTestMode() {
  scheduleGarbageCollectionBeforeExit();
  initRustAPI()
  const rustLogPattern = process.env.RUST_LOG || 'vcx=error'
  await vcx.defaultLogger(rustLogPattern)
  const useTestConfig = generateTestConfig()
  await vcx.initVcxCore(JSON.stringify(useTestConfig))
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




let garbageCollectionBeforeExitIsScheduled = false

// For some reason, The Rust library segfaults if global.gc() is not called explicitly.
const scheduleGarbageCollectionBeforeExit = () => {
  if (!garbageCollectionBeforeExitIsScheduled) {
    assert(global.gc)
    process.on('beforeExit', () => {
      global.gc();
    });
  }
  garbageCollectionBeforeExitIsScheduled = true;
}
