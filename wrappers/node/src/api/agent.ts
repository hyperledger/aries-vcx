import * as ffi from 'ffi-napi';
import * as ref from 'ref-napi';
import { VCXInternalError } from '../errors';
import { rustAPI } from '../rustlib';
import { createFFICallbackPromise, ICbRef } from '../utils/ffi-helpers';
import { ISerializedData, ConnectionStateType } from './common';
import { VCXBaseWithState } from './vcx-base-with-state';
import { PtrBuffer } from './utils';
import { VCXBase } from './vcx-base';
import { PaymentManager } from './vcx-payment-txn';

export interface IAgentInfo {
  agent_did: string,
  agent_vk: string
}

export interface IPairwiseInfo {
  my_did: string,
  my_vk: string
}

export interface IAgentSerializedData {
  agent_info: IAgentInfo,
  pairwise_info: IPairwiseInfo,
  institution_did: string
}

export class Agent extends VCXBase<IAgentSerializedData> {
  public static async create(institution_did: string): Promise<Agent> {
    const agent = new Agent('dummy_source_id');
    const commandHandle = 0;
    try {
      await agent._create((cb) =>
        rustAPI().vcx_public_agent_create(commandHandle, institution_did, cb),
      );
      return agent;
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  public async generatePublicInvite(label: string): Promise<string> {
    try {
      const data = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const commandHandle = 0;
          const rc = rustAPI().vcx_public_agent_generate_public_invite(
            commandHandle,
            this.handle,
            label,
            cb,
          );
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          ffi.Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (handle: number, err: number, invite: string) => {
              if (err) {
                reject(err);
                return;
              }
              if (!invite) {
                reject('no public invite returned');
                return;
              }
              resolve(invite);
            },
          ),
      );
      return data;
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  protected _serializeFn = rustAPI().vcx_public_agent_serialize;
  protected _deserializeFn = rustAPI().vcx_public_agent_deserialize;
  protected _releaseFn = rustAPI().vcx_public_agent_release;
}
