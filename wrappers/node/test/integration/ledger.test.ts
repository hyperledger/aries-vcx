import '../module-resolver-helper';
import * as path from 'path';
import { createAndStoreDid, openMainPool, shutdownVcx, writeEndorserDid } from 'src';
import { initVcx } from '../helpers/utils';
import { expect } from 'chai';

const seed = '00000LookAtMeIAmARandomSeed00000';

// @ts-ignore
let publicDid: string;

describe('wallet', () => {
  before(async () => publicDid = await initVcx());

  it('write new endorser from random seed', async () => {
    const genesisPath = path.join(__dirname, '/../../resources/localhost.txn');
    await openMainPool({ genesis_path: genesisPath });
    const pwInfo = await createAndStoreDid();
    expect(pwInfo.pw_did.length).equal(22);
    expect(pwInfo.pw_vk.length).equal(44);
  });

  it('write new endorser did from seed', async () => {
    const genesisPath = path.join(__dirname, '/../../resources/localhost.txn');
    await openMainPool({ genesis_path: genesisPath });
    const pwInfo = await createAndStoreDid(seed);
    expect(pwInfo.pw_did).equal('WSM9V5siLPUbx4BNnLH6Fj');
    expect(pwInfo.pw_vk).equal('H3Zchkv2nSYRZTVL66rhLnwJF9veJCYAm5eeBPoANyTH');
  });

  it('write new endorser did', async () => {
    const genesisPath = path.join(__dirname, '/../../resources/localhost.txn');
    await openMainPool({ genesis_path: genesisPath });
    const pwInfo = await createAndStoreDid();
    await writeEndorserDid(publicDid, pwInfo.pw_did, pwInfo.pw_vk, 'acme');
  });

  after(async () => await shutdownVcx(false));
});
