import * as ffi from 'ffi-napi';
import { VCXInternalError } from '../errors';
import { rustAPI } from '../rustlib';
import { createFFICallbackPromise } from '../utils/ffi-helpers';
import { ISerializedData } from './common';
import { VCXBase } from './vcx-base';

export interface IAgentInfo {
  agent_did: string,
  agent_vk: string
}

export interface IPairwiseInfo {
  my_did: string,
  my_vk: string
}

export interface IAgentSerializedData {
  source_id: string;
  agent_info: IAgentInfo,
  pairwise_info: IPairwiseInfo,
  institution_did: string
}

export class PublicAgent extends VCXBase<IAgentSerializedData> {
  public static async create(sourceId: string, institution_did: string): Promise<PublicAgent> {
    const agent = new PublicAgent(sourceId);
    const commandHandle = 0;
    try {
      await agent._create((cb) =>
        rustAPI().vcx_public_agent_create(commandHandle, sourceId, institution_did, cb),
      );
      return agent;
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  public async downloadConnectionRequests(uids: string): Promise<string> {
    try {
      const data = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const commandHandle = 0;
          const rc = rustAPI().vcx_public_agent_download_connection_requests(
            commandHandle,
            this.handle,
            uids,
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
            (handle: number, err: number, requests: string) => {
              if (err) {
                reject(err);
                return;
              }
              if (!requests) {
                reject('no connection requests returned');
                return;
              }
              resolve(requests);
            },
          ),
      );
      return data;
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  public async getService(): Promise<string> {
    try {
      const data = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const commandHandle = 0;
          const rc = rustAPI().vcx_public_agent_get_service(
            commandHandle,
            this.handle,
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
            (handle: number, err: number, service: string) => {
              if (err) {
                reject(err);
                return;
              }
              if (!service) {
                reject('no service returned');
                return;
              }
              resolve(service);
            },
          ),
      );
      return data;
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  public async downloadMessage(uid: string): Promise<string> {
    try {
      const data = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const commandHandle = 0;
          const rc = rustAPI().vcx_public_agent_download_message(
            commandHandle,
            this.handle,
            uid,
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
            (handle: number, err: number, msg: string) => {
              if (err) {
                reject(err);
                return;
              }
              if (!msg) {
                reject('no message returned');
                return;
              }
              resolve(msg);
            },
          ),
      );
      return data;
    } catch (err) {
      throw new VCXInternalError(err);
    }
  }

  public static async deserialize(
    agentData: ISerializedData<IAgentSerializedData>,
  ): Promise<PublicAgent> {
    const agent = await super._deserialize<PublicAgent>(PublicAgent, agentData);
    return agent;
  }

  protected _serializeFn = rustAPI().vcx_public_agent_serialize;
  protected _deserializeFn = rustAPI().vcx_public_agent_deserialize;
  protected _releaseFn = rustAPI().vcx_public_agent_release;
}
