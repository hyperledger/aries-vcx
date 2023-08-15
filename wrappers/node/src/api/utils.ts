import { VCXInternalError } from '../errors';
import * as ffi from '@hyperledger/vcx-napi-rs';

export interface IPwInfo {
  pw_did: string;
  pw_vk: string;
}

export interface IMsgUnpacked {
  sender_verkey: string;
  message: string;
}

export interface IAriesService {
  id: string;
  type: string;
  priority: number;
  recipientKeys: string[];
  routingKeys: string[];
  serviceEndpoint: string;
}

export interface IAriesServiceV2 {
  endpoint: string;
  routingKeys?: string[];
  types?: string[];
}

export async function provisionCloudAgent(configAgent: object): Promise<string> {
  try {
    return await ffi.provisionCloudAgent(JSON.stringify(configAgent));
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export function getVersion(): string {
  try {
    return ffi.getVersion();
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function getLedgerAuthorAgreement(): Promise<string> {
  try {
    return await ffi.getLedgerAuthorAgreement();
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export function setActiveTxnAuthorAgreementMeta(
  text: string,
  version: string,
  acceptanceMechanismType: string,
): void {
  try {
    ffi.setActiveTxnAuthorAgreementMeta(
      text,
      version,
      acceptanceMechanismType,
    );
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export function shutdownVcx(deleteWallet: boolean): void {
  try {
    ffi.shutdown(deleteWallet);
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export interface IUpdateWebhookUrl {
  webhookUrl: string;
}

export async function vcxUpdateWebhookUrl({ webhookUrl }: IUpdateWebhookUrl): Promise<void> {
  try {
    await ffi.updateWebhookUrl(webhookUrl);
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export interface IUpdateMessagesConfigs {
  msgJson: string;
}

export async function updateMessages(updateConfig: IUpdateMessagesConfigs): Promise<void> {
  try {
    await ffi.messagesUpdateStatus('MS-106', updateConfig.msgJson);
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function rotateVerkey(did: string): Promise<void> {
  try {
    await ffi.rotateVerkey(did);
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function rotateVerkeyStart(did: string): Promise<string> {
  try {
    return await ffi.rotateVerkeyStart(did);
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function rotateVerkeyApply(did: string, tempVk: string): Promise<void> {
  try {
    await ffi.rotateVerkeyApply(did, tempVk);
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function getVerkeyFromWallet(did: string): Promise<string> {
  try {
    return await ffi.getVerkeyFromWallet(did);
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function getVerkeyFromLedger(did: string): Promise<string> {
  try {
    return await ffi.getVerkeyFromLedger(did);
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function getLedgerTxn(did: string, seqNo: number): Promise<string> {
  try {
    return await ffi.getLedgerTxn(seqNo, did);
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function createAndStoreDid(seed?: string | undefined | null): Promise<IPwInfo> {
  try {
    return JSON.parse(await ffi.createAndStoreDid(seed));
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function createService(
  target_did: string,
  endpoint: string,
  recipientKeys: string[],
  routingKeys: string[],
): Promise<IAriesService> {
  try {
    return JSON.parse(await ffi.createService(target_did, recipientKeys, routingKeys, endpoint));
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function createServiceV2(
  target_did: string,
  endpoint: string,
  routingKeys: string[],
): Promise<IAriesServiceV2> {
  try {
    return JSON.parse(await ffi.createServiceV2(target_did, routingKeys, endpoint));
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function getServiceFromLedger(did: string): Promise<IAriesService | IAriesServiceV2> {
  try {
    return JSON.parse(await ffi.getServiceFromLedger(did));
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function getAttrFromLedger(did: string, attr: string): Promise<string> {
  try {
    return await ffi.getAttrFromLedger(did, attr);
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function clearAttrFromLedger(did: string, attr: string): Promise<void> {
  try {
    await ffi.clearAttrFromLedger(did, attr);
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function unpack(data: Buffer): Promise<IMsgUnpacked> {
  try {
    return JSON.parse(await ffi.unpack(data));
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}

export async function writeEndorserDid(submitterDid: string, targetDid: string, targetVk: string, alias?: string | undefined | null): Promise<IMsgUnpacked> {
  try {
    return JSON.parse(await ffi.writeEndorserDid(submitterDid, targetDid, targetVk, alias));
  } catch (err: any) {
    throw new VCXInternalError(err);
  }
}
