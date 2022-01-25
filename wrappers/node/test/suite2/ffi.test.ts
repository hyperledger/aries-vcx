import '../module-resolver-helper';

import { assert } from 'chai';
import * as ffi from 'ffi-napi';
import { initVcxTestMode } from 'helpers/utils';
import * as os from 'os';
import { VCXCode, VCXRuntime } from 'src';

// these tests were created to only test that the ffi could be called with each function

describe('Using the vcx ffi directly', () => {
  const extension = { darwin: '.dylib', linux: '.so', win32: '.dll' };
  const libPath = {
    darwin: '/usr/local/lib/',
    linux: '/usr/lib/',
    win32: 'c:\\windows\\system32\\',
  };

  const platform = os.platform();
  const postfix = extension[platform.toLowerCase() as keyof typeof extension] || extension.linux;
  const libDir = libPath[platform.toLowerCase() as keyof typeof libPath] || libPath.linux;
  const run = new VCXRuntime({ basepath: `${libDir}libvcx${postfix}` });

  before(() => initVcxTestMode());

  it('a call to vcx_connection_create should return 0', () => {
    const result = run.ffi.vcx_connection_create(
      0,
      '1',
      ffi.Callback(
        'void',
        ['uint32', 'uint32', 'uint32'],
        (_xhandle: number, _err: number, _connectionHandle: number) => null,
      ),
    );
    assert.equal(result, 0);
  });
});
