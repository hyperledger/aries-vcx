import '../module-resolver-helper'

// import { VCX_CONFIG_TEST_MODE } from 'helpers/test-constants'
// import { SinonStub, stub } from 'sinon'
import * as vcx from 'src'

// let _initVCXCalled = false
// let _patchInitVCXObj: SinonStub | undefined
// const _patchInitVCX = () => {
//   const initVCXOriginal = vcx.initVcx as any
//   const stubInitVCX = stub(vcx, 'initVcx')
//   // tslint:disable-next-line only-arrow-functions
//   stubInitVCX.callsFake(async function (...args) {
//     if (_initVCXCalled) {
//       console.log('calling a stub -> already called')
//       return
//     }
//     console.log('calling a stub -> calling original')
//     await initVCXOriginal(...args)
//     _initVCXCalled = true
//   })
//   return stubInitVCX
// }
// export const patchInitVCX = () => {
//   if (!_patchInitVCXObj) {
//     _patchInitVCXObj = _patchInitVCX()
//   }
//   return _patchInitVCXObj
// }

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
  agency_endpoint: 'http://127.0.0.1:8080',
  // protocol_type: '1.0'
}

export async function initVcxTestMode (protocolType: string) {
  const useTestConfig = { ... testConfig, protocol_type: protocolType}
  console.debug('Patching vcx init')
  // patchInitVCX()
  console.debug('Patched vcx init')
  console.debug(`Going to ini vcx with config ${JSON.stringify(useTestConfig)}`)
  await vcx.initVcxWithConfig(JSON.stringify(useTestConfig))
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
