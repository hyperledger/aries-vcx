import * as SegfaultHandler from 'segfault-handler';


import '../module-resolver-helper'
import {
  connectionCreateInviterNull
} from '../helpers/entities'
import { initVcxTestMode, sleep } from '../helpers/utils'
import { Connection } from '../../src'


SegfaultHandler.registerHandler("crash.log"); // With no argument, SegfaultHandler will generate a generic log file name


// SegfaultHandler.causeSegfault(); // simulates a buggy native module that dereferences NULL


process.on('beforeExit', () => {
  // global.gc();
})


const run = async () => {
  await initVcxTestMode()
  {
    const connection = await Connection.create({ id: 'foo' })
  }
  // await sleep(5000);
  // global.gc();
  // await sleep(5000);
}


run();