use indy_api_types::{errors::*, WalletHandle};
use indy_wallet::RecordOptions;
use crate::utils::crypto::base58::ToBase58;

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

use crate::controllers::vdr::VDRController;

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
}
