import * as ffi from '@hyperledger/vcx-napi-rs';
import { VCXInternalError } from '../errors';

export class OutOfBand {
  public static buildHandshakeReuseAcceptedMsg(handshakeReuse: string): string {
    try {
      return ffi.outOfBandBuildHandshakeReuseAcceptedMsg(handshakeReuse);
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }
}
