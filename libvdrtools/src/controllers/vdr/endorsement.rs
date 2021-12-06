#[cfg(feature = "cheqd")]
use std::convert::TryFrom;

use indy_api_types::{errors::*, WalletHandle};
use indy_wallet::RecordOptions;
use rust_base58::ToBase58;

#[cfg(feature = "cheqd")]
use indy_utils::crypto::base64;

use crate::domain::{
    vdr::{
        prepared_txn::{
            IndyEndorsementData,
            IndyEndorsement,
        },
    },
    crypto::{
        did::Did,
        key::Key,
        ED25519,
        SECP256K1,
    },
};

#[cfg(feature = "cheqd")]
use crate::domain::id::FullyQualifiedId;

#[cfg(feature = "cheqd")]
use crate::domain::{
    cheqd_keys::{
        Key as CheqdKey,
        KeyInfo,
    },
    cheqd_ledger::{
        abci_info::ABCIInfo,
        auth::{QueryAccountResponse, Account},
    },
    vdr::{
        prepared_txn::{
            CheqdEndorsementData,
            CheqdEndorsement,
        },
    },
    cheqd_ledger::tx::query_simulate_response::QuerySimulateResponse,
};
use crate::controllers::vdr::VDRController;
#[cfg(feature = "cheqd")]
use crate::controllers::vdr::VDR;

#[cfg(feature = "cheqd")]
mod constants {
    pub(crate) const DEFAULT_GAS_AMOUN: u64 = 300000;
    pub(crate) const DEFAULT_GAS_PRICE: u64 = 0;
    pub(crate) const ESTIMATE_GAS_FACTOR: f64 = 1.2;
    pub(crate) const COIN_DENOM: &'static str = "ncheq";
    pub(crate) const TIMEOUT_SHIFT: u64 = 17280;
}

impl VDRController {
    pub(crate) async fn indy_endorse(&self,
                                     wallet_handle: WalletHandle,
                                     endorsement_data: String,
                                     signature_spec: String,
                                     bytes_to_sign: Vec<u8>) -> IndyResult<String> {
        trace!(
            "indy_endorse > wallet_handle {:?} endorsement_data {:?} signature_spec {:?} bytes_to_sign {:?}",
            wallet_handle, endorsement_data, signature_spec, bytes_to_sign
        );
        let endorsement_data: IndyEndorsementData = serde_json::from_str(&endorsement_data)
            .map_err(|err| err_msg(
                IndyErrorKind::InvalidStructure,
                format!("Unable to parse Indy Endorsement data from JSON. Err: {:?}", err))
            )?;

        let did: Did = self
            .wallet_service
            .get_indy_object(wallet_handle, &endorsement_data.did, &RecordOptions::id_value())
            .await?;

        let key: Key = self
            .wallet_service
            .get_indy_object(wallet_handle, &did.verkey, &RecordOptions::id_value())
            .await?;

        let signature = match signature_spec.as_str() {
            ED25519 => {
                self.crypto_service.sign(&key, &bytes_to_sign).await?
            }
            SECP256K1 => {
                return Err(err_msg(
                    IndyErrorKind::UnknownCrypto,
                    "Secp256k1 signature type is not supported for Indy endorsement spec.",
                ));
            }
            type_ => {
                return Err(err_msg(
                    IndyErrorKind::UnknownCrypto,
                    format!("Unexpected signature type \"{}\".", type_),
                ));
            }
        };

        let endorsement = IndyEndorsement {
            signature: signature.to_base58()
        };
        json_string_result!(endorsement)
    }

    #[cfg(feature = "cheqd")]
    pub(crate) async fn prepare_cheqd_endorsement_data(&self,
                                                       vdr: &VDR,
                                                       wallet_handle: WalletHandle,
                                                       key_alias: String,
                                                       txn_author_did: String,
                                                       txn_bytes: Vec<u8>,
                                                       signature: Vec<u8>,
                                                       gas_price: u64,
                                                       memo: String) -> IndyResult<String> {
        trace!(
            "prepare_cheqd_endorsement_data > key_alias {:?} txn_author_did {:?} gas_price {:?} memo {:?}",
            key_alias, txn_author_did, gas_price, memo
        );

        let parsed_did: FullyQualifiedId = FullyQualifiedId::try_from(txn_author_did.as_str())
            .map_err(|err| err_msg(IndyErrorKind::InvalidStructure,
                                   format!("Error while converting fully-qualified DID to short representation. Err: {:?}", err))
            )?;

        let ledger = vdr.resolve_ledger_for_namespace(&parsed_did.namespace).await?;
        let name = ledger.name();

        let key: CheqdKey = self.wallet_service
            .get_indy_object(wallet_handle, &key_alias, &RecordOptions::id_value())
            .await
            .to_indy(IndyErrorKind::WalletItemNotFound, "Can't read cheqd key")?;

        let key_info: KeyInfo = self.cheqd_crypto_service.get_info(&key)?;

        let (account_number, sequence_number) = self.get_account_number_and_sequence(&key_info.account_id, &name).await?;
        let timeout_height = self.get_timeout_height(&name).await?;

        let chain_id = self.cheqd_pool_service.get_config(&name).await?.chain_id;

        let tx_bytes = self.cheqd_ledger_service.build_signed_message(&txn_bytes,
                                                                      &txn_author_did,
                                                                      &signature)?;

        let (_, tx_bytes) =
            self.cheqd_ledger_service
                .auth_build_tx(
                    &chain_id,
                    &key_info.pub_key,
                    &tx_bytes,
                    account_number,
                    sequence_number,
                    constants::DEFAULT_GAS_AMOUN,
                    constants::DEFAULT_GAS_PRICE,
                    constants::COIN_DENOM,
                    &key_info.account_id,
                    timeout_height,
                    &memo,
                )
                .await?;

        let max_gas = self.get_estimated_gas(&tx_bytes, &name).await?;

        let endorsement_data = CheqdEndorsementData {
            txn_author_did,
            chain_id,
            key_alias,
            account_number,
            sequence_number,
            max_gas,
            max_coin_amount: max_gas * gas_price,
            max_coin_denom: constants::COIN_DENOM.to_string(),
            timeout_height,
            memo,
        };

        json_string_result!(endorsement_data)
    }

    #[cfg(feature = "cheqd")]
    pub(crate) async fn cheqd_endorse(&self,
                                      wallet_handle: WalletHandle,
                                      endorsement_data: String,
                                      signature_spec: String,
                                      bytes_to_sign: Vec<u8>,
                                      signature: Vec<u8>) -> IndyResult<String> {
        trace!(
            "cheqd_endorse > wallet_handle {:?} endorsement_data {:?} signature_spec {:?} bytes_to_sign {:?} signature {:?}",
            wallet_handle, endorsement_data, signature_spec, bytes_to_sign, signature
        );
        let endorsement_data: CheqdEndorsementData = serde_json::from_str(&endorsement_data)
            .map_err(|err| err_msg(
                IndyErrorKind::InvalidStructure,
                format!("Unable to parse Cheqd Endorsement data from JSON. Err: {:?}", err)))?;

        let key: CheqdKey = self.wallet_service
            .get_indy_object(wallet_handle, &endorsement_data.key_alias, &RecordOptions::id_value())
            .await
            .to_indy(IndyErrorKind::WalletItemNotFound, "Can't read cheqd key")?;

        let key_info: KeyInfo = self.cheqd_crypto_service.get_info(&key)?;

        let signature = match signature_spec.as_str() {
            ED25519 => {
                return Err(err_msg(
                    IndyErrorKind::UnknownCrypto,
                    "Ed25519 signature type is not supported for Cheqd endorsement spec.",
                ));
            }
            SECP256K1 => {
                let tx_bytes = self.cheqd_ledger_service.build_signed_message(&bytes_to_sign,
                                                                              &endorsement_data.txn_author_did,
                                                                              &signature)?;

                let (_, sign_doc_bytes) =
                    self.cheqd_ledger_service
                        .auth_build_tx(
                            &endorsement_data.chain_id,
                            &key_info.pub_key,
                            &tx_bytes,
                            endorsement_data.account_number,
                            endorsement_data.sequence_number,
                            endorsement_data.max_gas,
                            endorsement_data.max_coin_amount,
                            &endorsement_data.max_coin_denom,
                            &key_info.account_id,
                            endorsement_data.timeout_height,
                            &endorsement_data.memo,
                        )
                        .await?;

                self.cheqd_crypto_service.sign(&key, &sign_doc_bytes).await?
            }
            type_ => {
                return Err(err_msg(
                    IndyErrorKind::UnknownCrypto,
                    format!("Unexpected signature type \"{}\".", type_),
                ));
            }
        };

        let signature = base64::encode(&signature);

        let endorsement = CheqdEndorsement {
            chain_id: endorsement_data.chain_id,
            txn_author_did: endorsement_data.txn_author_did,
            public_key: key_info.pub_key,
            account_id: key_info.account_id,
            account_number: endorsement_data.account_number,
            sequence_number: endorsement_data.sequence_number,
            max_gas: endorsement_data.max_gas,
            max_coin_amount: endorsement_data.max_coin_amount,
            max_coin_denom: endorsement_data.max_coin_denom,
            timeout_height: endorsement_data.timeout_height,
            memo: endorsement_data.memo,
            signature,
        };

        json_string_result!(endorsement)
    }

    #[cfg(feature = "cheqd")]
    pub async fn get_timeout_height(&self, pool_alias: &str) -> IndyResult<u64> {
        let info = self.cheqd_pool_service.abci_info(pool_alias).await?;
        let info: ABCIInfo = serde_json::from_str(&info)
            .map_err(|err| err_msg(
                IndyErrorKind::InvalidState,
                format!("Unable to parse ABCI Info from response. Err: {:?}", err),
            ))?;

        let current_height = info.response.last_block_height.parse::<u64>()
            .map_err(|err| err_msg(
                IndyErrorKind::InvalidState,
                format!("Unable to parse pool height. Err: {:?}", err),
            ))?;

        Ok(current_height + constants::TIMEOUT_SHIFT)
    }

    #[cfg(feature = "cheqd")]
    pub async fn get_account_number_and_sequence(&self, account_id: &str, pool_alias: &str) -> IndyResult<(u64, u64)> {
        let request = self.cheqd_ledger_service.auth_build_query_account(account_id)?;
        let response = self.cheqd_pool_service.abci_query(pool_alias, &request).await?;
        let parsed_response = self.cheqd_ledger_service.auth_parse_query_account_resp(&response)?;

        let account_info: QueryAccountResponse = serde_json::from_str(&parsed_response)
            .map_err(|err| err_msg(
                IndyErrorKind::InvalidState,
                format!("Unable to parse AccountInfo from response. Err: {:?}", err),
            ))?;

        let account: &Account = account_info.account.as_ref()
            .ok_or(err_msg(IndyErrorKind::InvalidState, "Unable to get Account Info"))?;

        Ok((account.account_number(), account.account_sequence()))
    }

    #[cfg(feature = "cheqd")]
    pub async fn get_estimated_gas(&self, tx: &[u8], pool_alias: &str) -> IndyResult<u64> {
        let request = self.cheqd_ledger_service.tx_build_query_simulate(tx)?;
        let response = self.cheqd_pool_service.abci_query(pool_alias, &request).await?;
        let parsed_response = self.cheqd_ledger_service.tx_parse_query_simulate_resp(&response)?;
        let parsed_response: QuerySimulateResponse = serde_json::from_str(&parsed_response)
            .map_err(|err| err_msg(
                IndyErrorKind::InvalidStructure,
                format!("Unable to parse QuerySimulateResponse. Err: {:?}", err),
            ))?;
        let gas_used = parsed_response.gas_info.map(|gas_info| gas_info.gas_used)
            .ok_or(err_msg(
                IndyErrorKind::InvalidState,
                "Unable to estimate gas amount for request",
            ))?;

        let max_gas = (gas_used as f64 * constants::ESTIMATE_GAS_FACTOR).round() as u64;
        Ok(max_gas)
    }
}
