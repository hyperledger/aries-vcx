import { errorMessage } from './utils/error-message';

export class ConnectionTimeoutError extends Error {}

export class VCXInternalError extends Error {
  public readonly vcxCode: number;
  public readonly inheritedStackTraces: string[] = [];

  constructor(code: number | Error) {
    super(errorMessage(code));
    if (code instanceof Error) {
      if (code.stack) {
        this.inheritedStackTraces.push(code.stack);
      }
      if (code instanceof VCXInternalError) {
        this.vcxCode = code.vcxCode;
        this.inheritedStackTraces.unshift(...code.inheritedStackTraces);
        return this;
      }
      // Here we assume that Error was thrown by VCX through NAPI
      this.vcxCode = parseInt(code.message)
      return this;
    }
    this.vcxCode = code;
  }
}
