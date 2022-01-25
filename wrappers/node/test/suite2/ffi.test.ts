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

  it(`a call to vcx_connection_connect should return ${VCXCode.SUCCESS}`, () => {
    const result = run.ffi.vcx_connection_connect(
      0,
      1,
      JSON.stringify({ connection_type: 'sms', phone: 123 }),
      ffi.Callback(
        'void',
        ['uint32', 'uint32', 'uint32'],
        (_xhandle: number, _err: number, _connectionHandle: number) => null,
      ),
    );
    assert.equal(result, VCXCode.SUCCESS);
  });

  it(`a call to vcx_connection_serialize should return ${VCXCode.SUCCESS}`, () => {
    const result = run.ffi.vcx_connection_serialize(
      0,
      1,
      ffi.Callback(
        'void',
        ['uint32', 'uint32', 'string'],
        (_xhandle: number, _err: number, _data: string) => null,
      ),
    );
    assert.equal(result, VCXCode.SUCCESS);
  });

  it(`a call to vcx_connection_get_state should return ${VCXCode.SUCCESS}`, () => {
    const result = run.ffi.vcx_connection_update_state(
      0,
      1,
      ffi.Callback(
        'void',
        ['uint32', 'uint32', 'uint32'],
        (_xhandle: number, _err: number, _state: number) => null,
      ),
    );
    assert.equal(result, VCXCode.SUCCESS);
  });
});
