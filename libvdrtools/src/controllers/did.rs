use std::{collections::HashMap, sync::Arc};

use indy_api_types::{errors::prelude::*, WalletHandle};
use indy_wallet::{RecordOptions, SearchOptions, WalletService};

use crate::{
    domain::{
        crypto::{
            did::{
                Did, DidMetadata, DidMethod, DidValue, DidWithMeta, MyDidInfo, TemporaryDid,
                TheirDid, TheirDidInfo,
            },
            key::KeyInfo,
        },
        ledger::{
            attrib::{AttribData, Endpoint, GetAttrReplyResult},
            did::{GetNymReplyResult, GetNymResultDataV0},
            response::Reply,
        },
        pairwise::Pairwise,
    },
    services::CryptoService,
    utils::crypto::base58::{DecodeBase58, ToBase58},
};

pub struct DidController {
    wallet_service: Arc<WalletService>,
    crypto_service: Arc<CryptoService>,
}

impl DidController {
    pub(crate) fn new(
        wallet_service: Arc<WalletService>,
        crypto_service: Arc<CryptoService>,
    ) -> DidController {
        DidController {
            wallet_service,
            crypto_service,
        }
    }

    /// Creates keys (signing and encryption keys) for a new
    /// DID (owned by the caller of the library).
    /// Identity's DID must be either explicitly provided, or taken as the first 16 bit of verkey.
    /// Saves the Identity DID with keys in a secured Wallet, so that it can be used to sign
    /// and encrypt transactions.
    ///
    /// #Params
    /// wallet_handle: wallet handler (created by open_wallet).

    /// did_info: Identity information as json. See domain::crypto::did::MyDidInfo
    /// Example:
    /// {
    ///     "did": string, (optional;
    ///             if not provided and cid param is false then the first 16 bit of the verkey will
    /// be used as a new DID;             if not provided and cid is true then the full verkey
    /// will be used as a new DID;             if provided, then keys will be replaced - key
    /// rotation use case)     "seed": string, (optional) Seed that allows deterministic did
    /// creation (if not set random one will be created).                                Can be
    /// UTF-8, base64 or hex string.     "crypto_type": string, (optional; if not set then
    /// ed25519 curve is used;               currently only 'ed25519' value is supported for
    /// this field)     "cid": bool, (optional; if not set then false is used;)
    ///     "ledger_type": string, (optional) type of the ledger to create fully qualified did.
    ///     "method_name": string, (optional) method name to create fully qualified did.
    /// }
    ///
    /// #Returns
    /// did: DID generated and stored in the wallet
    /// verkey: The DIDs verification key
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Crypto*
    pub async fn create_and_store_my_did(
        &self,
        wallet_handle: WalletHandle,
        my_did_info: MyDidInfo,
    ) -> IndyResult<(String, String)> {
        trace!(
            "create_and_store_my_did > wallet_handle {:?} my_did_info_json {:?}",
            wallet_handle,
            secret!(&my_did_info)
        );

        let (did, key) = self.crypto_service.create_my_did(&my_did_info).await?;

        if let Ok(current_did) = self._wallet_get_my_did(wallet_handle, &did.did).await {
            if did.verkey == current_did.verkey {
                let res = Ok((did.did.0, did.verkey));
                trace!("create_and_store_my_did < already exists {:?}", res);
                return res;
            } else {
                Err(err_msg(
                    IndyErrorKind::DIDAlreadyExists,
                    format!(
                        "DID \"{}\" already exists but with different Verkey. You should specify \
                         Seed used for initial generation",
                        did.did.0
                    ),
                ))?;
            }
        }

        self.wallet_service
            .add_indy_object(wallet_handle, &did.did.0, &did, &HashMap::new())
            .await?;

        let _ = self
            .wallet_service
            .add_indy_object(wallet_handle, &key.verkey, &key, &HashMap::new())
            .await
            .ok();

        let res = Ok((did.did.0, did.verkey));
        trace!("create_and_store_my_did < {:?}", res);
        res
    }

    /// Generated temporary keys (signing and encryption keys) for an existing
    /// DID (owned by the caller of the library).
    ///
    /// #Params
    /// wallet_handle: wallet handler (created by open_wallet).

    /// did: target did to rotate keys.
    /// key_info: key information as json. Example:
    /// {
    ///     "seed": string, (optional) Seed that allows deterministic key creation (if not set
    /// random one will be created).                                Can be UTF-8, base64 or hex
    /// string.     "crypto_type": string, (optional; if not set then ed25519 curve is used;
    ///               currently only 'ed25519' value is supported for this field)
    /// }
    ///
    /// #Returns
    /// verkey: The DIDs verification key
    ///
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Crypto*
    pub async fn replace_keys_start(
        &self,
        wallet_handle: WalletHandle,
        key_info: KeyInfo,
        my_did: DidValue,
    ) -> IndyResult<String> {
        trace!(
            "replace_keys_start > wallet_handle {:?} key_info_json {:?} my_did {:?}",
            wallet_handle,
            secret!(&key_info),
            my_did
        );

        self.crypto_service.validate_did(&my_did)?;

        let my_did = self._wallet_get_my_did(wallet_handle, &my_did).await?;

        let temporary_key = self.crypto_service.create_key(&key_info).await?;

        let my_temporary_did = TemporaryDid {
            did: my_did.did,
            verkey: temporary_key.verkey.clone(),
        };

        self.wallet_service
            .add_indy_object(
                wallet_handle,
                &temporary_key.verkey,
                &temporary_key,
                &HashMap::new(),
            )
            .await?;

        self.wallet_service
            .add_indy_object(
                wallet_handle,
                &my_temporary_did.did.0,
                &my_temporary_did,
                &HashMap::new(),
            )
            .await?;

        let res = Ok(my_temporary_did.verkey);
        trace!("replace_keys_start < {:?}", res);
        res
    }

    /// Apply temporary keys as main for an existing DID (owned by the caller of the library).
    ///
    /// #Params
    /// wallet_handle: wallet handler (created by open_wallet).

    /// did: DID stored in the wallet
    ///
    /// #Returns
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Crypto*
    pub async fn replace_keys_apply(
        &self,
        wallet_handle: WalletHandle,
        my_did: DidValue,
    ) -> IndyResult<()> {
        trace!(
            "replace_keys_apply > wallet_handle {:?} my_did {:?}",
            wallet_handle,
            my_did
        );

        self.crypto_service.validate_did(&my_did)?;

        let my_did = self._wallet_get_my_did(wallet_handle, &my_did).await?;

        let my_temporary_did: TemporaryDid = self
            .wallet_service
            .get_indy_object(wallet_handle, &my_did.did.0, &RecordOptions::id_value())
            .await?;

        let my_did = Did::from(my_temporary_did);

        self.wallet_service
            .update_indy_object(wallet_handle, &my_did.did.0, &my_did)
            .await?;

        self.wallet_service
            .delete_indy_record::<TemporaryDid>(wallet_handle, &my_did.did.0)
            .await?;

        let res = Ok(());
        trace!("replace_keys_apply < {:?}", res);
        res
    }

    /// Saves their DID for a pairwise connection in a secured Wallet,
    /// so that it can be used to verify transaction.
    /// Updates DID associated verkey in case DID already exists in the Wallet.
    ///
    /// #Params
    /// wallet_handle: wallet handler (created by open_wallet).

    /// identity_json: Identity information as json. Example:
    ///     {
    ///        "did": string, (required)
    ///        "verkey": string
    ///             - optional is case of adding a new DID, and DID is cryptonym: did == verkey,
    ///             - mandatory in case of updating an existing DID
    ///     }
    ///
    /// #Returns
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Crypto*
    pub async fn store_their_did(
        &self,
        wallet_handle: WalletHandle,
        their_did_info: TheirDidInfo,
    ) -> IndyResult<()> {
        trace!(
            "store_their_did > wallet_handle {:?} their_did_info {:?}",
            wallet_handle,
            their_did_info
        );

        let their_did = self
            .crypto_service
            .create_their_did(&their_did_info)
            .await?;

        self.wallet_service
            .upsert_indy_object(wallet_handle, &their_did.did.0, &their_did)
            .await?;

        let res = Ok(());
        trace!("store_their_did < {:?}", res);
        res
    }

    /// Retrieves the information about the giving DID in the wallet.
    ///
    /// #Params

    /// wallet_handle: Wallet handle (created by open_wallet).
    /// did - The DID to retrieve information.
    ///
    /// #Returns
    /// did_with_meta: {
    ///   "did": string - DID stored in the wallet,
    ///   "verkey": string - The DIDs transport key (ver key, key id),
    ///   "tempVerkey": string - Temporary DIDs transport key (ver key, key id), exist only during
    /// the rotation of the keys.                          After rotation is done, it becomes a
    /// new verkey.   "metadata": string - The meta information stored with the DID
    /// }
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Crypto*
    pub async fn get_my_did_with_meta(
        &self,
        wallet_handle: WalletHandle,
        my_did: DidValue,
    ) -> IndyResult<String> {
        trace!(
            "get_my_did_with_meta > wallet_handle {:?} my_did {:?}",
            wallet_handle,
            my_did
        );

        let did = self
            .wallet_service
            .get_indy_object::<Did>(wallet_handle, &my_did.0, &RecordOptions::id_value())
            .await?;

        let metadata = self
            .wallet_service
            .get_indy_opt_object::<DidMetadata>(
                wallet_handle,
                &did.did.0,
                &RecordOptions::id_value(),
            )
            .await?;

        let temp_verkey = self
            .wallet_service
            .get_indy_opt_object::<TemporaryDid>(
                wallet_handle,
                &did.did.0,
                &RecordOptions::id_value(),
            )
            .await?;

        let did_with_meta = DidWithMeta {
            did: did.did,
            verkey: did.verkey,
            temp_verkey: temp_verkey.map(|tv| tv.verkey),
            metadata: metadata.map(|m| m.value),
        };

        let did_with_meta = serde_json::to_string(&did_with_meta)
            .to_indy(IndyErrorKind::InvalidState, "Can't serialize DID")?;

        let res = Ok(did_with_meta);
        trace!("get_my_did_with_meta < {:?}", res);
        res
    }

    /// Retrieves the information about all DIDs stored in the wallet.
    ///
    /// #Params

    /// wallet_handle: Wallet handle (created by open_wallet).
    ///
    /// #Returns
    /// dids:  [{
    ///   "did": string - DID stored in the wallet,
    ///   "verkey": string - The DIDs transport key (ver key, key id).,
    ///   "metadata": string - The meta information stored with the DID
    /// }]
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Crypto*
    pub async fn list_my_dids_with_meta(&self, wallet_handle: WalletHandle) -> IndyResult<String> {
        trace!("list_my_dids_with_meta > wallet_handle {:?}", wallet_handle);

        let mut did_search = self
            .wallet_service
            .search_indy_records::<Did>(wallet_handle, "{}", &SearchOptions::id_value())
            .await?;

        let mut metadata_search = self
            .wallet_service
            .search_indy_records::<DidMetadata>(wallet_handle, "{}", &SearchOptions::id_value())
            .await?;

        let mut temporarydid_search = self
            .wallet_service
            .search_indy_records::<TemporaryDid>(wallet_handle, "{}", &SearchOptions::id_value())
            .await?;

        let mut dids: Vec<DidWithMeta> = Vec::new();

        let mut metadata_map: HashMap<String, String> = HashMap::new();
        let mut temporarydid_map: HashMap<String, String> = HashMap::new();

        while let Some(record) = metadata_search.fetch_next_record().await? {
            let did_id = record.get_id();

            let tup: DidMetadata = record
                .get_value()
                .ok_or(err_msg(
                    IndyErrorKind::InvalidState,
                    "No value for DID record",
                ))
                .and_then(|tags_json| {
                    serde_json::from_str(&tags_json).to_indy(
                        IndyErrorKind::InvalidState,
                        format!("Cannot deserialize Did {:?}", did_id),
                    )
                })?;

            metadata_map.insert(String::from(did_id), tup.value);
        }

        while let Some(record) = temporarydid_search.fetch_next_record().await? {
            let did_id = record.get_id();

            let did: TemporaryDid = record
                .get_value()
                .ok_or(err_msg(
                    IndyErrorKind::InvalidState,
                    "No value for DID record",
                ))
                .and_then(|tags_json| {
                    serde_json::from_str(&tags_json).to_indy(
                        IndyErrorKind::InvalidState,
                        format!("Cannot deserialize Did {:?}", did_id),
                    )
                })?;

            temporarydid_map.insert(did.did.0, did.verkey);
        }

        while let Some(did_record) = did_search.fetch_next_record().await? {
            let did_id = did_record.get_id();

            let did: Did = did_record
                .get_value()
                .ok_or_else(|| err_msg(IndyErrorKind::InvalidState, "No value for DID record"))
                .and_then(|tags_json| {
                    serde_json::from_str(&tags_json).to_indy(
                        IndyErrorKind::InvalidState,
                        format!("Cannot deserialize Did {:?}", did_id),
                    )
                })?;

            let temp_verkey = temporarydid_map.remove(&did.did.0);
            let metadata = metadata_map.remove(&did.did.0);

            let did_with_meta = DidWithMeta {
                did: did.did,
                verkey: did.verkey,
                temp_verkey: temp_verkey,
                metadata: metadata,
            };

            dids.push(did_with_meta);
        }

        let dids = serde_json::to_string(&dids)
            .to_indy(IndyErrorKind::InvalidState, "Can't serialize DIDs list")?;

        let res = Ok(dids);
        trace!("list_my_dids_with_meta < {:?}", res);
        res
    }

    /// Returns ver key (key id) for the given DID.
    ///
    /// "indy_key_for_local_did" call looks data stored in the local wallet only and skips freshness
    /// checking.
    ///
    /// Note if you want to get fresh data from the ledger you can use "indy_key_for_did" call
    /// instead.
    ///
    /// Note that "indy_create_and_store_my_did" makes similar wallet record as "indy_create_key".
    /// As result we can use returned ver key in all generic crypto and messaging functions.
    ///
    /// #Params

    /// wallet_handle: Wallet handle (created by open_wallet).
    /// did - The DID to resolve key.
    ///
    /// #Returns
    /// key - The DIDs ver key (key id).
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Crypto*
    pub async fn key_for_local_did(
        &self,
        wallet_handle: WalletHandle,
        did: DidValue,
    ) -> IndyResult<String> {
        trace!(
            "key_for_local_did > wallet_handle {:?} did {:?}",
            wallet_handle,
            did
        );

        self.crypto_service.validate_did(&did)?;

        // Look to my did
        let my_did = match self._wallet_get_my_did(wallet_handle, &did).await {
            Ok(my_did) => Some(my_did),
            Err(err) if err.kind() == IndyErrorKind::WalletItemNotFound => None,
            Err(err) => Err(err)?,
        };

        if let Some(my_did) = my_did {
            let res = Ok(my_did.verkey);
            trace!("key_for_local_did < my {:?}", res);
            return res;
        }

        // look to their did
        let their_did = self._wallet_get_their_did(wallet_handle, &did).await?;

        let res = Ok(their_did.verkey);
        trace!("key_for_local_did < {:?}", res);
        res
    }

    /// Set/replaces endpoint information for the given DID.
    ///
    /// #Params

    /// wallet_handle: Wallet handle (created by open_wallet).
    /// did - The DID to resolve endpoint.
    /// address -  The DIDs endpoint address. indy-node and indy-plenum restrict this to
    /// ip_address:port transport_key - The DIDs transport key (ver key, key id).
    ///
    /// #Returns
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Crypto*
    pub async fn set_endpoint_for_did(
        &self,
        wallet_handle: WalletHandle,
        did: DidValue,
        endpoint: Endpoint,
    ) -> IndyResult<()> {
        trace!(
            "set_endpoint_for_did > wallet_handle {:?} did {:?} endpoint {:?}",
            wallet_handle,
            did,
            endpoint
        );

        self.crypto_service.validate_did(&did)?;

        if let Some(ref transport_key) = endpoint.verkey {
            self.crypto_service.validate_key(transport_key).await?;
        }

        self.wallet_service
            .upsert_indy_object(wallet_handle, &did.0, &endpoint)
            .await?;

        let res = Ok(());
        trace!("set_endpoint_for_did < {:?}", res);
        res
    }

    /// Saves/replaces the meta information for the giving DID in the wallet.
    ///
    /// #Params

    /// wallet_handle: Wallet handle (created by open_wallet).
    /// did - the DID to store metadata.
    /// metadata - the meta information that will be store with the DID.
    ///
    /// #Returns
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Crypto*
    pub async fn set_did_metadata(
        &self,
        wallet_handle: WalletHandle,
        did: DidValue,
        metadata: String,
    ) -> IndyResult<()> {
        trace!(
            "set_did_metadata > wallet_handle {:?} did {:?} metadata {:?}",
            wallet_handle,
            did,
            metadata
        );

        self.crypto_service.validate_did(&did)?;

        let metadata = DidMetadata { value: metadata };

        self.wallet_service
            .upsert_indy_object(wallet_handle, &did.0, &metadata)
            .await?;

        let res = Ok(());
        trace!("set_did_metadata < {:?}", res);
        res
    }

    /// Retrieves the meta information for the giving DID in the wallet.
    ///
    /// #Params

    /// wallet_handle: Wallet handle (created by open_wallet).
    /// did - The DID to retrieve metadata.
    ///
    /// #Returns
    /// metadata - The meta information stored with the DID; Can be null if no metadata was saved
    /// for this DID.
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Crypto*
    pub async fn get_did_metadata(
        &self,
        wallet_handle: WalletHandle,
        did: DidValue,
    ) -> IndyResult<String> {
        trace!(
            "get_did_metadata > wallet_handle {:?} did {:?}",
            wallet_handle,
            did
        );

        self.crypto_service.validate_did(&did)?;

        let metadata = self
            .wallet_service
            .get_indy_object::<DidMetadata>(wallet_handle, &did.0, &RecordOptions::id_value())
            .await?;

        let res = Ok(metadata.value);
        trace!("get_did_metadata < {:?}", res);
        res
    }

    /// Retrieves abbreviated verkey if it is possible otherwise return full verkey.
    ///
    /// #Params

    /// did: DID.
    /// full_verkey: The DIDs verification key,
    ///
    /// #Returns
    /// verkey: The DIDs verification key in either abbreviated or full form
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Crypto*
    pub async fn abbreviate_verkey(&self, did: DidValue, verkey: String) -> IndyResult<String> {
        trace!("abbreviate_verkey > did {:?} verkey {:?}", did, verkey);

        self.crypto_service.validate_did(&did)?;
        self.crypto_service.validate_key(&verkey).await?;

        if !did.is_abbreviatable() {
            let res = Ok(verkey);
            trace!("abbreviate_verkey < not abbreviatable {:?}", res);
            return res;
        }

        let did = &did.to_unqualified().0.decode_base58()?;
        let dverkey = &verkey.decode_base58()?;

        let (first_part, second_part) = dverkey.split_at(16);

        let res = if first_part.eq(did.as_slice()) {
            format!("~{}", second_part.to_base58())
        } else {
            verkey
        };

        let res = Ok(res);
        trace!("abbreviate_verkey < {:?}", res);
        res
    }

    /// Update DID stored in the wallet to make fully qualified, or to do other DID maintenance.
    ///     - If the DID has no method, a method will be appended (prepend did:peer to a legacy did)
    ///     - If the DID has a method, a method will be updated (migrate did:peer to did:peer-new)
    ///
    /// Update DID related entities stored in the wallet.
    ///
    /// #Params

    /// wallet_handle: Wallet handle (created by open_wallet).
    /// did: target DID stored in the wallet.
    /// method: method to apply to the DID.
    ///
    /// #Returns
    /// did: fully qualified form of did
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Crypto*
    pub async fn qualify_did(
        &self,
        wallet_handle: WalletHandle,
        did: DidValue,
        method: DidMethod,
    ) -> IndyResult<String> {
        trace!(
            "qualify_did > wallet_handle {:?} curr_did {:?} method {:?}",
            wallet_handle,
            did,
            method
        );

        self.crypto_service.validate_did(&did)?;

        let mut curr_did: Did = self
            .wallet_service
            .get_indy_object::<Did>(wallet_handle, &did.0, &RecordOptions::id_value())
            .await?;

        curr_did.did = DidValue::new(&did.to_short().0, None, Some(&method.0))?;

        self.wallet_service
            .delete_indy_record::<Did>(wallet_handle, &did.0)
            .await?;

        self.wallet_service
            .add_indy_object(wallet_handle, &curr_did.did.0, &curr_did, &HashMap::new())
            .await?;

        // move temporary Did
        if let Ok(mut temp_did) = self
            .wallet_service
            .get_indy_object::<TemporaryDid>(wallet_handle, &did.0, &RecordOptions::id_value())
            .await
        {
            temp_did.did = curr_did.did.clone();

            self.wallet_service
                .delete_indy_record::<TemporaryDid>(wallet_handle, &did.0)
                .await?;

            self.wallet_service
                .add_indy_object(wallet_handle, &curr_did.did.0, &temp_did, &HashMap::new())
                .await?;
        }

        // move metadata
        self._update_dependent_entity_reference::<DidMetadata>(
            wallet_handle,
            &did.0,
            &curr_did.did.0,
        )
        .await?;

        // move endpoint
        self._update_dependent_entity_reference::<Endpoint>(wallet_handle, &did.0, &curr_did.did.0)
            .await?;

        // move all pairwise
        let mut pairwise_search = self
            .wallet_service
            .search_indy_records::<Pairwise>(wallet_handle, "{}", &RecordOptions::id_value())
            .await?;

        while let Some(pairwise_record) = pairwise_search.fetch_next_record().await? {
            let mut pairwise: Pairwise = pairwise_record
                .get_value()
                .ok_or_else(|| err_msg(IndyErrorKind::InvalidState, "No value for Pairwise record"))
                .and_then(|pairwise_json| {
                    serde_json::from_str(&pairwise_json).map_err(|err| {
                        IndyError::from_msg(
                            IndyErrorKind::InvalidState,
                            format!("Cannot deserialize Pairwise {:?}", err),
                        )
                    })
                })?;

            if pairwise.my_did.eq(&did) {
                pairwise.my_did = curr_did.did.clone();

                self.wallet_service
                    .update_indy_object(wallet_handle, &pairwise.their_did.0, &pairwise)
                    .await?;
            }
        }

        let res = Ok(curr_did.did.0);
        trace!("qualify_did < {:?}", res);
        res
    }

    pub async fn get_nym_ack_process_and_store_their_did(
        &self,
        wallet_handle: WalletHandle,
        did: DidValue,
        get_nym_reply_result: IndyResult<String>,
    ) -> IndyResult<TheirDid> {
        trace!(
            "get_nym_ack_process_and_store_their_did > wallet_handle {:?} get_nym_reply_result \
             {:?}",
            wallet_handle,
            get_nym_reply_result
        );

        let get_nym_reply = get_nym_reply_result?;

        let get_nym_response: Reply<GetNymReplyResult> = serde_json::from_str(&get_nym_reply)
            .to_indy(
                IndyErrorKind::InvalidState,
                "Invalid GetNymReplyResult json",
            )?;

        let their_did_info = match get_nym_response.result() {
            GetNymReplyResult::GetNymReplyResultV0(res) => {
                if let Some(data) = &res.data {
                    let gen_nym_result_data: GetNymResultDataV0 = serde_json::from_str(data)
                        .to_indy(IndyErrorKind::InvalidState, "Invalid GetNymResultData json")?;

                    TheirDidInfo::new(
                        gen_nym_result_data.dest.qualify(did.get_method()),
                        gen_nym_result_data.verkey,
                    )
                } else {
                    return Err(err_msg(
                        IndyErrorKind::WalletItemNotFound,
                        "Their DID isn't found on the ledger",
                    )); //TODO FIXME use separate error
                }
            }
            GetNymReplyResult::GetNymReplyResultV1(res) => TheirDidInfo::new(
                res.txn.data.did.qualify(did.get_method()),
                res.txn.data.verkey,
            ),
        };

        let their_did = self
            .crypto_service
            .create_their_did(&their_did_info)
            .await?;

        self.wallet_service
            .add_indy_object(wallet_handle, &their_did.did.0, &their_did, &HashMap::new())
            .await?;

        trace!("get_nym_ack_process_and_store_their_did <<<");

        Ok(their_did)
    }

    async fn _update_dependent_entity_reference<T>(
        &self,
        wallet_handle: WalletHandle,
        id: &str,
        new_id: &str,
    ) -> IndyResult<()>
    where
        T: ::serde::Serialize + ::serde::de::DeserializeOwned + Sized,
    {
        if let Ok(record) = self
            .wallet_service
            .get_indy_record_value::<T>(wallet_handle, id, "{}")
            .await
        {
            self.wallet_service
                .delete_indy_record::<T>(wallet_handle, id)
                .await?;
            self.wallet_service
                .add_indy_record::<T>(wallet_handle, new_id, &record, &HashMap::new())
                .await?;
        }

        Ok(())
    }

    async fn _get_attrib_ack_process_store_endpoint_to_wallet(
        &self,
        wallet_handle: WalletHandle,
        get_attrib_reply_result: IndyResult<String>,
    ) -> IndyResult<Endpoint> {
        trace!(
            "_get_attrib_ack_process_store_endpoint_to_wallet > wallet_handle {:?} \
             get_attrib_reply_result {:?}",
            wallet_handle,
            get_attrib_reply_result
        );

        let get_attrib_reply = get_attrib_reply_result?;

        let get_attrib_reply: Reply<GetAttrReplyResult> = serde_json::from_str(&get_attrib_reply)
            .to_indy(
            IndyErrorKind::InvalidState,
            "Invalid GetAttrReplyResult json",
        )?;

        let (raw, did) = match get_attrib_reply.result() {
            GetAttrReplyResult::GetAttrReplyResultV0(res) => (res.data, res.dest),
            GetAttrReplyResult::GetAttrReplyResultV1(res) => (res.txn.data.raw, res.txn.data.did),
        };

        let attrib_data: AttribData = serde_json::from_str(&raw)
            .to_indy(IndyErrorKind::InvalidState, "Invalid GetAttReply json")?;

        let endpoint = Endpoint::new(attrib_data.endpoint.ha, attrib_data.endpoint.verkey);

        self.wallet_service
            .add_indy_object(wallet_handle, &did.0, &endpoint, &HashMap::new())
            .await?;

        let res = Ok(endpoint);

        trace!(
            "_get_attrib_ack_process_store_endpoint_to_wallet < {:?}",
            res
        );

        res
    }

    async fn _wallet_get_my_did(
        &self,
        wallet_handle: WalletHandle,
        my_did: &DidValue,
    ) -> IndyResult<Did> {
        self.wallet_service
            .get_indy_object(wallet_handle, &my_did.0, &RecordOptions::id_value())
            .await
    }

    async fn _wallet_get_their_did(
        &self,
        wallet_handle: WalletHandle,
        their_did: &DidValue,
    ) -> IndyResult<TheirDid> {
        self.wallet_service
            .get_indy_object(wallet_handle, &their_did.0, &RecordOptions::id_value())
            .await
    }
}
