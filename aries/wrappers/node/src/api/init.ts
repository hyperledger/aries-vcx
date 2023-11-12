import * as ffiNapi from '@hyperledger/vcx-napi-rs';
import { VCXInternalError } from '../errors';

export function createAgencyClientForMainWallet(config: object): void {
  try {
    ffiNapi.createAgencyClientForMainWallet(JSON.stringify(config));
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function openMainPool(config: object): Promise<void> {
  try {
    return await ffiNapi.openMainPool(JSON.stringify(config));
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}