import * as ffi from '@hyperledger/vcx-napi-rs';
import { VCXInternalError } from '../errors';

export class Trustsping {
  static buildResponse(
    ping: string
  ): string {
    try {
      return ffi.trustpingBuildResponseMsg(ping);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  static buildPing(
    requestResponse: boolean,
    comment?: string
  ): string {
    try {
      return ffi.trustpingBuildPing(requestResponse, comment);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }
}

