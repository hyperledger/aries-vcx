import { assert } from 'chai';
import { initRustAPI, isRustApiInitialized } from 'src';
import * as vcx from 'src';
import * as uuid from 'uuid';
import '../module-resolver-helper';

const oldConfig = {
  // link_secret_alias: 'main',
};

const configThreadpool = {
  threadpool_size: '4',
}

const configWalletSample = {
  use_latest_protocols: 'false',
  enable_test_mode: 'true',
  wallet_key: '8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY',
  wallet_key_derivation: 'RAW',
  wallet_type: 'default',
  wallet_name: 'LIBVCX_SDK_WALLET',
  backup_key: 'backup_wallet_key',
  exported_wallet_path: '/var/folders/libvcx_nodetest/sample.wallet',
}

const configPool = {
  pool_name: 'pool1',
  protocol_version: '2',
}

const configAgency = {
  agency_endpoint: 'http://127.0.0.1:8080',
  agency_did: '2hoqvcwupRTUNkXn6ArYzs',
  agency_verkey: 'FuN98eH2eZybECWkofW6A9BKJxxnTatBCopfUiNxo6ZB',
  remote_to_sdk_verkey: 'FuN98eH2eZybECWkofW6A9BKJxxnTatBCopfUiNxo6ZB',
  sdk_to_remote_did: '2hoqvcwupRTUNkXn6ArYzs',
  remote_to_sdk_did: '2hoqvcwupRTUNkXn6ArYzs',
  sdk_to_remote_verkey: 'FuN98eH2eZybECWkofW6A9BKJxxnTatBCopfUiNxo6ZB',
}

const issuerConfig = {

  institution_did: '2hoqvcwupRTUNkXn6ArYzs',
}

const issuerSeed = "000000000000000000000000Trustee1"

function generateWalletConfig() {
  const walletId = uuid.v4();
  return {
    ...configWalletSample,
    wallet_name: `testnodejs_${walletId}`,
    exported_wallet_path: `/var/folders/libvcx_nodetest/wallet_${walletId}.wallet`
  };
}

export async function initVcxTestMode(): Promise<void> {
  scheduleGarbageCollectionBeforeExit();
  if (!isRustApiInitialized()) {
    initRustAPI();
  }
  const rustLogPattern = process.env.RUST_LOG || 'vcx=error';
  vcx.defaultLogger(rustLogPattern);
  vcx.initThreadpool(configThreadpool)
  const configWallet = generateWalletConfig();
  await vcx.createWallet(configWallet)
  await vcx.openMainWallet(configWallet)
  const { institution_did, institution_verkey }  = JSON.parse(await vcx.configureIssuerWallet(issuerSeed))
  const issuerConfig = {
    institution_name: 'default',
    institution_did,
    institution_verkey
  }
  await vcx.initIssuerConfig(issuerConfig)
  await vcx.createAgencyClientForMainWallet(configAgency)
  vcx.enableMocks()
}

export const shouldThrow = (fn: () => any): Promise<vcx.VCXInternalError> =>
  new Promise(async (resolve, reject) => {
    try {
      await fn();
      reject(new Error(`${fn.toString()} should have thrown!`));
    } catch (e) {
      resolve(e);
    }
  });

export const sleep = (timeout: number): Promise<void> =>
  new Promise((resolve, _reject) => {
    setTimeout(resolve, timeout);
  });

let garbageCollectionBeforeExitIsScheduled = false;

// For some (yet unknown) reason, The Rust library segfaults on exit if global.gc() is not called explicitly.
// To solve this issue, we call global.gc() on `beforeExit` event.
// NB: This solution only works with Mocha.
//     With Jest the 'beforeExit' event doesn't seem fired, so we are instead still using --forceExit before it segfaults.
const scheduleGarbageCollectionBeforeExit = () => {
  if (!garbageCollectionBeforeExitIsScheduled) {
    assert(global.gc);
    process.on('beforeExit', () => {
      if (typeof global.gc != 'undefined') {
        global.gc();
      }
    });
  }
  garbageCollectionBeforeExitIsScheduled = true;
};
