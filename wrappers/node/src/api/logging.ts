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

// The _logger must in fact be instance of Struct type we generated above using buildStructType(ref)
export function loggerToVoidPtr(_logger: LoggerType): PtrBuffer {
  const _pointer = ref.alloc('void *') as PtrBuffer;
  ref.writePointer(_pointer, 0, _logger.ref());
  return _pointer;
}

function voidPtrToLogger(loggerPtr: LoggerPtr): LoggerType {
  const loggerPtrType = ref.refType(Logger);
  loggerPtr.type = loggerPtrType;
  return loggerPtr.deref().deref();
}

const Ilogger = {
  context: ref.refType(ref.refType('void')),
  file: 'string',
  level: 'uint32',
  line: 'uint32',
  message: 'string',
  module_path: 'string',
  target: 'string',
};

function flushFunction(loggerPtr: LoggerPtr) {
  const _logger = voidPtrToLogger(loggerPtr);
  _logger.flushFn();
}

export function loggerFunction(
  loggerPtr: LoggerPtr,
  level: number,
  target: string,
  message: string,
  modulePath: string,
  file: string,
  line: number,
): void {
  const _logger = voidPtrToLogger(loggerPtr);
  _logger.logFn(level, target, message, modulePath, file, line);
}

const loggerFnCb = ffi.Callback(
  'void',
  [
    Ilogger.context,
    Ilogger.level,
    Ilogger.target,
    Ilogger.message,
    Ilogger.module_path,
    Ilogger.file,
    Ilogger.line,
  ],
  (
    loggerPtr: LoggerPtr,
    level: number,
    target: string,
    message: string,
    modulePath: string,
    file: string,
    line: number,
  ) => {
    loggerFunction(loggerPtr, level, target, message, modulePath, file, line);
  },
);

const flushFnCb = ffi.Callback('void', [Ilogger.context], (loggerPtr: LoggerPtr) => {
  flushFunction(loggerPtr);
});
// need to keep these in this scope so they are not garbage collected.
const logger = Logger();
let pointer;

/**
 *
 * Set the Logger to A Custom Logger
 *
 * Example:
 * ```
 * var logFn = (level: number, target: string, message: string, modulePath: string, file: string, line: number) => {
 *   count = count + 1
 *   console.log('level: ' + level)
 *   console.log('target: ' + target)
 *   console.log('message: ' + message)
 *   console.log('modulePath: ' + modulePath)
 *   console.log('file: ' + file)
 *   console.log('line: ' + line)
 * }
 * setLogger(logFn)
 * ```
 *
 */
export function setLogger(userLogFn: LogFunction): void {
  logger.logFn = userLogFn;
  logger.flushFn = () => {};
  pointer = loggerToVoidPtr(logger);
  try {
    rustAPI().vcx_set_logger(pointer, ref.NULL, loggerFnCb, flushFnCb);
  } catch (err) {
    throw new VCXInternalError(err);
  }
}

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
  } catch (err) {
    throw new VCXInternalError(err);
  }
}
