import { Callback } from 'ffi-napi';
import * as ref from 'ref-napi';

import { VCXInternalError } from '../errors';
import { rustAPI } from '../rustlib';
import { createFFICallbackPromise } from '../utils/ffi-helpers';

/**
 * @interface An interface representing a record that can be added to the wallet
 */
export interface IRecord {
  type_: string;
  id: string;
  value: any; // eslint-disable-line @typescript-eslint/no-explicit-any
  tags_json: any; // eslint-disable-line @typescript-eslint/no-explicit-any
}

export interface IRecordUpdate {
  type_: string;
  id: string;
  value: any; // eslint-disable-line @typescript-eslint/no-explicit-any
}

export interface IDeleteRecordTagsOptions {
  tagList: string[];
}

export interface IDeleteRecordData {
  type: string;
  id: string;
}

export interface IGetRecordOptions {
  retrieveType: boolean;
  retrieveValue: boolean;
  retrieveTags: boolean;
}

export interface IGerRecordData {
  type: string;
  id: string;
  optionsJson: IGetRecordOptions;
}

export interface IOpenSearchData {
  type: string;
  queryJson: string;
  optionsJson: string;
}

export interface ISearchNextRecordsOptions {
  count: number;
}

export async function createWallet (config: object): Promise<void> {
  try {
    await createFFICallbackPromise<void>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_create_wallet(0, JSON.stringify(config), cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32'],
        (xhandle: number, err: number) => {
          if (err) {
            reject(err)
            return
          }
          resolve()
        })
    )
  } catch (err: any) {
    throw new VCXInternalError(err)
  }
}

export async function configureIssuerWallet (seed: string): Promise<string> {
  try {
    const issuerConfig = await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_configure_issuer_wallet(0, seed, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32','string'],
        (xhandle: number, err: number, config: string) => {
          if (err) {
            reject(err)
            return
          }
          resolve(config)
        })
    )
    return issuerConfig
  } catch (err: any) {
    throw new VCXInternalError(err)
  }
}

export async function openMainWallet (config: object): Promise<void> {
  try {
    await createFFICallbackPromise<void>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_open_main_wallet(0, JSON.stringify(config), cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32'],
        (xhandle: number, err: number) => {
          if (err) {
            reject(err)
            return
          }
          resolve()
        })
    )
  } catch (err: any) {
    throw new VCXInternalError(err)
  }
}

export async function closeMainWallet (): Promise<void> {
  try {
    await createFFICallbackPromise<void>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_close_main_wallet(0, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32'],
        (xhandle: number, err: number) => {
          if (err) {
            reject(err)
            return
          }
          resolve()
        })
    )
  } catch (err: any) {
    throw new VCXInternalError(err)
  }
}

/**
 * @class Class representing a Wallet
 */
export class Wallet {
   /**
   * Adds a record to the wallet for storage
   * Example:
   * ```
   * await Wallet.addRecord({
   *    id: 'RecordId',
   *    tags: {},
   *    type_: 'TestType',
   *    value: 'RecordValue'
   * })
   * ```
   * @async
   * @param {Record} record
   * @returns {Promise<void>}
   */
  public static async addRecord(record: IRecord): Promise<void> {
    const commandHandle = 0;
    try {
      await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_add_record(
            commandHandle,
            record.type_,
            record.id,
            record.value,
            JSON.stringify(record.tags_json),
            cb,
          );
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          Callback('void', ['uint32', 'uint32'], (xhandle: number, err: number) => {
            if (err) {
              reject(err);
              return;
            }
            resolve();
          }),
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  /**
   * Updates a record already in the wallet
   *
   * Example:
   * ```
   * await Wallet.addRecord({
   *    id: 'RecordId',
   *    tags: {},
   *    type_: 'TestType',
   *    value: 'RecordValue'
   * })
   * await Wallet.updateRecordValue({
   *   id: 'RecordId',
   *   type_: 'TestType',
   *   value: 'RecordValueNew'
   * })
   * ```
   */
  public static async updateRecordValue(record: IRecordUpdate): Promise<void> {
    const commandHandle = 0;
    try {
      await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_update_record_value(
            commandHandle,
            record.type_,
            record.id,
            record.value,
            cb,
          );
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          Callback('void', ['uint32', 'uint32'], (xhandle: number, err: number) => {
            if (err) {
              reject(err);
              return;
            }
            resolve();
          }),
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  /**
   * Updates a record's tags already in the wallet
   *
   * Example:
   * ```
   * await Wallet.addRecord({
   *     id: 'RecordId',
   *     tags: {},
   *     type_: 'TestType',
   *     value: 'RecordValue'
   * })
   *
   * updateRecordTags({
   *     id: 'RecordId',
   *     tags: {},
   *     type_: 'TestType',
   *     value: ''
   * })
   * ```
   */
  public static async updateRecordTags(record: IRecord): Promise<void> {
    const commandHandle = 0;
    try {
      await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_update_record_tags(
            commandHandle,
            record.type_,
            record.id,
            JSON.stringify(record.tags_json),
            cb,
          );
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          Callback('void', ['uint32', 'uint32'], (xhandle: number, err: number) => {
            if (err) {
              reject(err);
              return;
            }
            resolve();
          }),
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  /**
   * Adds tags to a record already in the wallet
   *
   * Example:
   * ```
   * await Wallet.addRecord({
   *     id: 'RecordId',
   *     tags: {},
   *     type_: 'TestType',
   *     value: 'RecordValue'
   * })
   *
   * addRecordTags({  id: 'RecordId',
   *     tags: {
   *          "tagName1": "tag value 1",
   *          "~tagName2": "tag value 2 unencrypted",
   *           "tagName3", 1
   *     },
   *     type_: 'TestType',
   *     value: ''
   * })
   * ```
   */
  public static async addRecordTags(record: IRecord): Promise<void> {
    const commandHandle = 0;
    try {
      await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_add_record_tags(
            commandHandle,
            record.type_,
            record.id,
            JSON.stringify(record.tags_json),
            cb,
          );
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          Callback('void', ['uint32', 'uint32'], (xhandle: number, err: number) => {
            if (err) {
              reject(err);
              return;
            }
            resolve();
          }),
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  /**
   * Tags to delete from a record already in the wallet
   *
   * Example:
   * ```
   * await Wallet.addRecord({
   *    id: 'RecordId',
   *    tags: {
   *        "foo": "bar",
   *        "~fizz": "buzz",
   *        "unencyrptedStringTag": "tag value 1",
   *        "~encryptedStringTag": "tag value 2 unencrypted",
   *        "unencyrptedIntTag": 1
   *    },
   *    type_: 'TestType',
   *    value: 'RecordValue'
   * })
   *
   * deleteRecordTags({
   *     id: 'RecordId',
   *     tags: { tagList: [ "foo", "buzz", "~encryptedStringTag" ] }
   *     type_: 'TestType',
   *     value: ''
   * })
   * ```
   */
  public static async deleteRecordTags(
    record: IRecord,
    { tagList }: IDeleteRecordTagsOptions,
  ): Promise<void> {
    const commandHandle = 0;
    try {
      await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_delete_record_tags(
            commandHandle,
            record.type_,
            record.id,
            JSON.stringify(tagList),
            cb,
          );
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          Callback('void', ['uint32', 'uint32'], (xhandle: number, err: number) => {
            if (err) {
              reject(err);
              return;
            }
            resolve();
          }),
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  /**
   * Delete a record already in the wallet
   *
   * Example:
   * ```
   * await Wallet.addRecord({
   *    id: 'RecordId',
   *    tags: {
   *        "foo": "bar",
   *        "~fizz": "buzz",
   *        "unencyrptedStringTag": "tag value 1",
   *        "~encryptedStringTag": "tag value 2 unencrypted",
   *        "unencyrptedIntTag": 1
   *    },
   *    type_: 'TestType',
   *    value: 'RecordValue'
   * })
   *
   * await Wallet.deleteRecord({
   *    id: 'RecordId',
   *    type_: 'TestType'
   * })
   * ```
   */
  public static async deleteRecord({ type, id }: IDeleteRecordData): Promise<void> {
    const commandHandle = 0;
    try {
      await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_delete_record(commandHandle, type, id, cb);
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          Callback('void', ['uint32', 'uint32'], (xhandle: number, err: number) => {
            if (err) {
              reject(err);
              return;
            }
            resolve();
          }),
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  /**
   * Retrieve a record already in the wallet
   *
   * Example:
   * ```
   * await Wallet.addRecord({
   *    id: 'RecordId',
   *    tags: {
   *        "foo": "bar",
   *        "~fizz": "buzz",
   *        "unencyrptedStringTag": "tag value 1",
   *        "~encryptedStringTag": "tag value 2 unencrypted",
   *        "unencyrptedIntTag": 1
   *    },
   *    type_: 'TestType',
   *    value: 'RecordValue'
   * })
   *
   * record = await Wallet.getReocrd({ type: 'TestType', id: 'RecordId'})
   * ```
   */
  public static async getRecord({ type, id, optionsJson }: IGerRecordData): Promise<string> {
    const commandHandle = 0;
    try {
      return await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_get_record(
            commandHandle,
            type,
            id,
            JSON.stringify(optionsJson),
            cb,
          );
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (xhandle: number, err: number, info: string) => {
              if (err) {
                reject(err);
                return;
              }
              resolve(info);
            },
          ),
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  /**
   * Open a search handle
   *
   * Example:
   * ```
   * searchHandle = await openSearch({type: 'TestType'})
   * ```
   */
  public static async openSearch({
    type,
    queryJson,
    optionsJson,
  }: IOpenSearchData): Promise<number> {
    const commandHandle = 0;
    try {
      return await createFFICallbackPromise<number>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_open_search(
            commandHandle,
            type,
            queryJson,
            optionsJson,
            cb,
          );
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          Callback(
            'void',
            ['uint32', 'uint32', 'uint32'],
            (xhandle: number, err: number, handle: number) => {
              if (err) {
                reject(err);
                return;
              }
              resolve(handle);
            },
          ),
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  /**
   * Close a search handle
   *
   * Example:
   * ```
   * searchHandle = await Wallet.openSearch({type: 'TestType'})
   * await Wallet.closeSearch(searchHandle)
   * ```
   */
  public static async closeSearch(handle: number): Promise<void> {
    const commandHandle = 0;
    try {
      await createFFICallbackPromise<number>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_close_search(commandHandle, handle, cb);
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          Callback('void', ['uint32', 'uint32'], (xhandle: number, err: number) => {
            if (err) {
              reject(err);
              return;
            }
            resolve(handle);
          }),
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  /**
   * Initiate or continue a search
   *
   * Example:
   * ```
   * searchHandle = await Wallet.openSearch({type: 'TestType'})
   * records = await Wallet.searchNextRecords(searchHandle, {count:5})
   * await Wallet.closeSearch(searchHandle)
   * ```
   */
  public static async searchNextRecords(
    handle: number,
    { count }: ISearchNextRecordsOptions,
  ): Promise<string> {
    const commandHandle = 0;
    try {
      return await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_search_next_records(commandHandle, handle, count, cb);
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (xhandle: number, err: number, info: string) => {
              if (err) {
                reject(err);
                return;
              }
              resolve(info);
            },
          ),
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  /**
   * Imports wallet from file with given key.
   * Cannot be used if wallet is already opened.
   *
   * Example:
   * ```
   * config = {
   *     "wallet_name":"",
   *     "wallet_key":"",
   *     "exported_wallet_path":"",
   *     "backup_key":""
   * }
   * await Wallet.import(JSON.stringify(config))
   * ```
   */
  public static async import(config: string): Promise<void> {
    const commandHandle = 0;
    try {
      await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_import(commandHandle, config, cb);
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          Callback('void', ['uint32', 'uint32'], (xhandle: number, err: number) => {
            if (err) {
              reject(err);
              return;
            }
            resolve();
          }),
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  /**
   * Export a file to a wallet, backup key used for decrypting the file.
   *
   * Example:
   * ```
   * await Wallet.export('/tmp/foobar.wallet', 'key_for_wallet')
   * ```
   */
  public static async export(path: string, backupKey: string): Promise<void> {
    const commandHandle = 0;
    try {
      await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_export(commandHandle, path, backupKey, cb);
          if (rc) {
            reject(rc);
          }
        },
        (resolve, reject) =>
          Callback('void', ['uint32', 'uint32'], (xhandle: number, err: number) => {
            if (err) {
              reject(err);
              return;
            }
            resolve();
          }),
      );
    } catch (err: any) {
      throw new VCXInternalError(err);
    }
  }

  /**
   * Set the wallet handle for libvcx to use, called before vcxInitPostIndy
   *
   * Example:
   * ```
   * Wallet.setHandle(1)
   * setPoolHandle(1)
   * vcxInitPostIndy(config)
   */
  public static setHandle(handle: number): void {
    rustAPI().vcx_wallet_set_handle(handle);
  }
}
