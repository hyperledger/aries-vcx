import '../module-resolver-helper'
import {
  connectionCreateInviterNull
} from '../helpers/entities'
import { initVcxTestMode, sleep } from '../helpers/utils'
import { Connection } from '../../src'

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