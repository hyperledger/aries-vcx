import * as ffi from '@hyperledger/vcx-napi-rs';
import { VCXInternalError } from '../errors';

export class Trustsping {
  static async buildResponse(
    ping: string
  ): Promise<string> {
    try {
      return JSON.parse(ffi.trustpingBuildResponseMsg(ping));
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  static async buildPing(
    requestResponse: boolean,
    comment?: string
  ): Promise<string> {
    try {
      return JSON.parse(ffi.trustpingBuildPing(requestResponse, comment));
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }
}

