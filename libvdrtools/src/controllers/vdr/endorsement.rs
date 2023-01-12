use crate::utils::crypto::base58::ToBase58;
use indy_api_types::{errors::*, WalletHandle};
use indy_wallet::RecordOptions;

use crate::domain::{
    crypto::{did::Did, key::Key, ED25519, SECP256K1},
    vdr::prepared_txn::{IndyEndorsement, IndyEndorsementData},
};

use crate::controllers::vdr::VDRController;

impl VDRController {
    /// Endorse Indy transaction (prepare and sign with endorser DID).
    ///
    /// EXPERIMENTAL
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// wallet_handle: handle pointing to an opened wallet (returned by indy_open_wallet)
    /// endorsement_data: data required for transaction endorsing
    ///     {
    ///         "did": string - DID to use for transaction signing
    ///     }
    /// signature_spec: type of the signature used for transaction signing
    /// txn_bytes_to_sign_raw: a pointer to first byte of transaction bytes to sign
    /// txn_bytes_to_sign_len: a transaction length
    /// cb: Callback that takes command result as parameter
    ///
    /// #Returns
    /// Error Code
    /// cb:
    /// - command_handle_: command handle to map callback to caller context.
    /// - err: Error code.
    /// - endorsement: generated endorsement information
    ///         {
    ///             "signature": string - endorser transaction signature as baste58 string
    ///         }
    pub(crate) async fn indy_endorse(
        &self,
        wallet_handle: WalletHandle,
        endorsement_data: String,
        signature_spec: String,
        bytes_to_sign: Vec<u8>,
    ) -> IndyResult<String> {
        trace!(
            "indy_endorse > wallet_handle {:?} endorsement_data {:?} signature_spec {:?} bytes_to_sign {:?}",
            wallet_handle, endorsement_data, signature_spec, bytes_to_sign
        );
        let endorsement_data: IndyEndorsementData = serde_json::from_str(&endorsement_data)
            .map_err(|err| {
                err_msg(
                    IndyErrorKind::InvalidStructure,
                    format!(
                        "Unable to parse Indy Endorsement data from JSON. Err: {:?}",
                        err
                    ),
                )
            })?;

        let did: Did = self
            .wallet_service
            .get_indy_object(
                wallet_handle,
                &endorsement_data.did,
                &RecordOptions::id_value(),
            )
            .await?;

        let key: Key = self
            .wallet_service
            .get_indy_object(wallet_handle, &did.verkey, &RecordOptions::id_value())
            .await?;

        let signature = match signature_spec.as_str() {
            ED25519 => self.crypto_service.sign(&key, &bytes_to_sign).await?,
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
            signature: signature.to_base58(),
        };
        json_string_result!(endorsement)
    }
}
