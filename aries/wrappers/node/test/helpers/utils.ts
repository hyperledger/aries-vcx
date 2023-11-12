import * as vcx from 'src';
import * as uuid from 'uuid';
import '../module-resolver-helper';

const configWalletSample = {
  use_latest_protocols: 'false',
  enable_test_mode: 'true',
  wallet_key: '8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY',
  wallet_key_derivation: 'RAW',
  wallet_type: 'default',
  wallet_name: 'LIBVCX_SDK_WALLET',
  backup_key: 'backup_wallet_key',
  exported_wallet_path: '/var/folders/libvcx_nodetest/sample.wallet',
};

const configAgency = {
  agency_endpoint: 'http://127.0.0.1:8080',
  agency_did: '2hoqvcwupRTUNkXn6ArYzs',
  agency_verkey: 'FuN98eH2eZybECWkofW6A9BKJxxnTatBCopfUiNxo6ZB',
  remote_to_sdk_verkey: 'FuN98eH2eZybECWkofW6A9BKJxxnTatBCopfUiNxo6ZB',
  sdk_to_remote_did: '2hoqvcwupRTUNkXn6ArYzs',
  remote_to_sdk_did: '2hoqvcwupRTUNkXn6ArYzs',
  sdk_to_remote_verkey: 'FuN98eH2eZybECWkofW6A9BKJxxnTatBCopfUiNxo6ZB',
};

const issuerSeed = '000000000000000000000000Trustee1';

function generateWalletConfig() {
  const walletId = uuid.v4();
  return {
    ...configWalletSample,
    wallet_name: `testnodejs_${walletId}`,
    exported_wallet_path: `/var/folders/libvcx_nodetest/wallet_${walletId}.wallet`,
  };
}

export async function initVcx(): Promise<string> {
  const rustLogPattern = process.env.RUST_LOG || 'vcx=error';
  vcx.defaultLogger(rustLogPattern);
  const configWallet = generateWalletConfig();
  await vcx.createWallet(configWallet);
  await vcx.openMainWallet(configWallet);
  const { institution_did } = JSON.parse(await vcx.configureIssuerWallet(issuerSeed));
  const issuerConfig = {
    institution_did,
  };
  vcx.createAgencyClientForMainWallet(configAgency);
  return institution_did
}

export const shouldThrow = (fn: () => any): Promise<any> =>
  new Promise(async (resolve, reject) => {
    try {
      await fn();
      reject(new Error(`${fn.toString()} should have thrown!`));
    } catch (e: any) {
      resolve(e);
    }
  });
