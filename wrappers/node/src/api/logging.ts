import * as ffi from 'ffi-napi';
import * as ref from 'ref-napi';
import * as buildStructType from 'ref-struct-di';

import { VCXInternalError } from '../errors';
import { rustAPI } from '../rustlib';
import { PtrBuffer } from './utils';

export type LogFunction = (
  level: number,
  target: string,
  message: string,
  modulePath: string,
  file: string,
  line: number,
) => void;

const Struct = buildStructType(ref);

export const Logger = Struct({
  flushFn: ffi.Function('void', []),
  logFn: ffi.Function('void', ['int', 'string', 'string', 'string', 'string', 'int']),
});

type LoggerType = typeof Logger;

type LoggerPtr = PtrBuffer;

const Ilogger = {
  context: ref.refType(ref.refType('void')),
  file: 'string',
  level: 'uint32',
  line: 'uint32',
  message: 'string',
  module_path: 'string',
  target: 'string',
};

/**
 * Sets the Logger to Default
 *
 * Accepts a string indicating what level to log at.
 * Example:
 * ```
 * defaultLogger('info')
 * ```
 *
 */

export function defaultLogger(level: string): void {
  try {
    rustAPI().vcx_set_default_logger(level);
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}
