use std::{collections::HashMap, str, sync::Arc};

use indy_api_types::{errors::prelude::*, WalletHandle};
use indy_utils::crypto::{base64, chacha20poly1305_ietf};
use indy_wallet::RecordOptions;

use crate::{
    domain::crypto::{
        key::{Key, KeyInfo},
        pack::*,
    },
    services::{CryptoService, WalletService},
};

pub const PROTECTED_HEADER_ENC: &str = "xchacha20poly1305_ietf";
pub const PROTECTED_HEADER_TYP: &str = "JWM/1.0";
pub const PROTECTED_HEADER_ALG_AUTH: &str = "Authcrypt";
pub const PROTECTED_HEADER_ALG_ANON: &str = "Anoncrypt";

pub struct CryptoController {
    wallet_service: Arc<WalletService>,
    crypto_service: Arc<CryptoService>,
}

impl CryptoController {
    pub(crate) fn new(
        wallet_service: Arc<WalletService>,
        crypto_service: Arc<CryptoService>,
    ) -> CryptoController {
        CryptoController {
            wallet_service,
            crypto_service,
        }
    }

    /// Creates keys pair and stores in the wallet.
    ///
    /// #Params

    /// wallet_handle: Wallet handle (created by open_wallet).
    /// key_json: Key information as json. Example:
    /// {
    ///     "seed": string, (optional) Seed that allows deterministic key creation (if not set
    /// random one will be created).                                Can be UTF-8, base64 or hex
    /// string.     "crypto_type": string, // Optional (if not set then ed25519 curve is used);
    /// Currently only 'ed25519' value is supported for this field. }
    ///
    /// #Returns
    /// verkey: Ver key of generated key pair, also used as key identifier
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Crypto*
    pub async fn create_key(
        &self,
        wallet_handle: WalletHandle,
        key_info: &KeyInfo,
    ) -> IndyResult<String> {
        debug!(
            "create_key >>> wallet_handle: {:?}, key_info: {:?}",
            wallet_handle,
            secret!(key_info)
        );

        let key = self.crypto_service.create_key(key_info).await?;

        self.wallet_service
            .add_indy_object(wallet_handle, &key.verkey, &key, &HashMap::new())
            .await?;

        let res = key.verkey.to_string();
        debug!("create_key <<< res: {:?}", res);
        Ok(res)
    }

    /// Signs a message with a key.
    ///
    /// Note to use DID keys with this function you can call indy_key_for_did to get key id (verkey)
    /// for specific DID.
    ///
    /// #Params

    /// wallet_handle: wallet handler (created by open_wallet).
    /// signer_vk: id (verkey) of message signer. The key must be created by calling indy_create_key
    /// or indy_create_and_store_my_did message_raw: a pointer to first byte of message to be
    /// signed message_len: a message length
    ///
    /// #Returns
    /// a signature string
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Crypto*
    pub async fn crypto_sign(
        &self,
        wallet_handle: WalletHandle,
        my_vk: &str,
        msg: &[u8],
    ) -> IndyResult<Vec<u8>> {
        trace!(
            "crypto_sign >>> wallet_handle: {:?}, sender_vk: {:?}, msg: {:?}",
            wallet_handle,
            my_vk,
            msg
        );

        self.crypto_service.validate_key(my_vk).await?;

        let key: Key = self
            .wallet_service
            .get_indy_object(wallet_handle, my_vk, &RecordOptions::id_value())
            .await?;

        let res = self.crypto_service.sign(&key, msg).await?;

        trace!("crypto_sign <<< res: {:?}", res);

        Ok(res)
    }

    /// Verify a signature with a verkey.
    ///
    /// Note to use DID keys with this function you can call indy_key_for_did to get key id (verkey)
    /// for specific DID.
    ///
    /// #Params

    /// signer_vk: verkey of the message signer
    /// message_raw: a pointer to first byte of message that has been signed
    /// message_len: a message length
    /// signature_raw: a pointer to first byte of signature to be verified
    /// signature_len: a signature length
    ///
    /// #Returns
    /// valid: true - if signature is valid, false - otherwise
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Ledger*
    /// Crypto*
    pub async fn crypto_verify(
        &self,
        their_vk: &str,
        msg: &[u8],
        signature: &[u8],
    ) -> IndyResult<bool> {
        trace!(
            "crypto_verify >>> their_vk: {:?}, msg: {:?}, signature: {:?}",
            their_vk,
            msg,
            signature
        );

        self.crypto_service.validate_key(their_vk).await?;

        let res = self.crypto_service.verify(their_vk, msg, signature).await?;

        trace!("crypto_verify <<< res: {:?}", res);

        Ok(res)
    }

    /// Packs a message by encrypting the message and serializes it in a JWE-like format
    /// (Experimental)
    ///
    /// Note to use DID keys with this function you can call indy_key_for_did to get key id (verkey)
    /// for specific DID.
    ///
    /// #Params

    /// wallet_handle: wallet handle (created by open_wallet).
    /// message: a pointer to the first byte of the message to be packed
    /// message_len: the length of the message
    /// receivers: a string in the format of a json list which will contain the list of receiver's
    /// keys                the message is being encrypted for.
    ///                Example:
    ///                "[<receiver edge_agent_1 verkey>, <receiver edge_agent_2 verkey>]"
    /// sender: the sender's verkey as a string When null pointer is used in this parameter,
    /// anoncrypt is used
    ///
    /// #Returns
    /// a JWE using authcrypt alg is defined below:
    /// {
    ///     "protected": "b64URLencoded({
    ///        "enc": "xsalsa20poly1305",
    ///        "typ": "JWM/1.0",
    ///        "alg": "Authcrypt",
    ///        "recipients": [
    ///            {
    ///                "encrypted_key": base64URLencode(libsodium.crypto_box(my_key, their_vk, cek,
    /// cek_iv))                "header": {
    ///                     "kid": "base58encode(recipient_verkey)",
    ///                     "sender" : base64URLencode(libsodium.crypto_box_seal(their_vk,
    /// base58encode(sender_vk)),                     "iv" : base64URLencode(cek_iv)
    ///                }
    ///            },
    ///        ],
    ///     })",
    ///     "iv": <b64URLencode(iv)>,
    ///     "ciphertext": b64URLencode(encrypt_detached({'@type'...}, protected_value_encoded, iv,
    /// cek),     "tag": <b64URLencode(tag)>
    /// }
    ///
    /// Alternative example in using anoncrypt alg is defined below:
    /// {
    ///     "protected": "b64URLencoded({
    ///        "enc": "xsalsa20poly1305",
    ///        "typ": "JWM/1.0",
    ///        "alg": "Anoncrypt",
    ///        "recipients": [
    ///            {
    ///                "encrypted_key": base64URLencode(libsodium.crypto_box_seal(their_vk, cek)),
    ///                "header": {
    ///                    "kid": base58encode(recipient_verkey),
    ///                }
    ///            },
    ///        ],
    ///     })",
    ///     "iv": b64URLencode(iv),
    ///     "ciphertext": b64URLencode(encrypt_detached({'@type'...}, protected_value_encoded, iv,
    /// cek),     "tag": b64URLencode(tag)
    /// }
    ///
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Ledger*
    /// Crypto*
    // TODO: Refactor pack to be more modular to version changes or crypto_scheme changes
    // this match statement is super messy, but the easiest way to comply with current architecture
    pub async fn pack_msg(
        &self,
        message: Vec<u8>,
        receiver_list: Vec<String>,
        sender_vk: Option<String>,
        wallet_handle: WalletHandle,
    ) -> IndyResult<Vec<u8>> {
        //break early and error out if no receivers keys are provided
        if receiver_list.is_empty() {
            return Err(err_msg(
                IndyErrorKind::InvalidStructure,
                "No receiver keys found".to_string(),
            ));
        }

        //generate content encryption key that will encrypt `message`
        let cek = chacha20poly1305_ietf::gen_key();

        let base64_protected = if let Some(sender_vk) = sender_vk {
            self.crypto_service.validate_key(&sender_vk).await?;

            //returns authcrypted pack_message format. See Wire message format HIPE for details
            self._prepare_protected_authcrypt(&cek, receiver_list, &sender_vk, wallet_handle)
                .await?
        } else {
            //returns anoncrypted pack_message format. See Wire message format HIPE for details
            self._prepare_protected_anoncrypt(&cek, receiver_list)
                .await?
        };

        // Use AEAD to encrypt `message` with "protected" data as "associated data"
        let (ciphertext, iv, tag) =
            self.crypto_service
                .encrypt_plaintext(message, &base64_protected, &cek);

        self._format_pack_message(&base64_protected, &ciphertext, &iv, &tag)
    }

    async fn _prepare_protected_anoncrypt(
        &self,
        cek: &chacha20poly1305_ietf::Key,
        receiver_list: Vec<String>,
    ) -> IndyResult<String> {
        let mut encrypted_recipients_struct: Vec<Recipient> =
            Vec::with_capacity(receiver_list.len());

        for their_vk in receiver_list {
            //encrypt sender verkey
            let enc_cek = self
                .crypto_service
                .crypto_box_seal(&their_vk, &cek[..])
                .await?;

            //create recipient struct and push to encrypted list
            encrypted_recipients_struct.push(Recipient {
                encrypted_key: base64::encode_urlsafe(enc_cek.as_slice()),
                header: Header {
                    kid: their_vk,
                    sender: None,
                    iv: None,
                },
            });
        } // end for-loop

        self._base64_encode_protected(encrypted_recipients_struct, false)
    }

    async fn _prepare_protected_authcrypt(
        &self,
        cek: &chacha20poly1305_ietf::Key,
        receiver_list: Vec<String>,
        sender_vk: &str,
        wallet_handle: WalletHandle,
    ) -> IndyResult<String> {
        let mut encrypted_recipients_struct: Vec<Recipient> = vec![];

        //get my_key from my wallet
        let my_key = self
            .wallet_service
            .get_indy_object(wallet_handle, sender_vk, &RecordOptions::id_value())
            .await?;

        //encrypt cek for recipient
        for their_vk in receiver_list {
            let (enc_cek, iv) = self
                .crypto_service
                .crypto_box(&my_key, &their_vk, &cek[..])
                .await?;

            let enc_sender = self
                .crypto_service
                .crypto_box_seal(&their_vk, sender_vk.as_bytes())
                .await?;

            //create recipient struct and push to encrypted list
            encrypted_recipients_struct.push(Recipient {
                encrypted_key: base64::encode_urlsafe(enc_cek.as_slice()),
                header: Header {
                    kid: their_vk,
                    sender: Some(base64::encode_urlsafe(enc_sender.as_slice())),
                    iv: Some(base64::encode_urlsafe(iv.as_slice())),
                },
            });
        } // end for-loop

        self._base64_encode_protected(encrypted_recipients_struct, true)
    }

    fn _base64_encode_protected(
        &self,
        encrypted_recipients_struct: Vec<Recipient>,
        alg_is_authcrypt: bool,
    ) -> IndyResult<String> {
        let alg_val = if alg_is_authcrypt {
            String::from(PROTECTED_HEADER_ALG_AUTH)
        } else {
            String::from(PROTECTED_HEADER_ALG_ANON)
        };

        //structure protected and base64URL encode it
        let protected_struct = Protected {
            enc: PROTECTED_HEADER_ENC.to_string(),
            typ: PROTECTED_HEADER_TYP.to_string(),
            alg: alg_val,
            recipients: encrypted_recipients_struct,
        };
        let protected_encoded = serde_json::to_string(&protected_struct).map_err(|err| {
            err_msg(
                IndyErrorKind::InvalidStructure,
                format!("Failed to serialize protected field {}", err),
            )
        })?;

        Ok(base64::encode_urlsafe(protected_encoded.as_bytes()))
    }

    fn _format_pack_message(
        &self,
        base64_protected: &str,
        ciphertext: &str,
        iv: &str,
        tag: &str,
    ) -> IndyResult<Vec<u8>> {
        //serialize pack message and return as vector of bytes
        let jwe_struct = JWE {
            protected: base64_protected.to_string(),
            iv: iv.to_string(),
            ciphertext: ciphertext.to_string(),
            tag: tag.to_string(),
        };

        serde_json::to_vec(&jwe_struct).map_err(|err| {
            err_msg(
                IndyErrorKind::InvalidStructure,
                format!("Failed to serialize JWE {}", err),
            )
        })
    }

    /// Unpacks a JWE-like formatted message outputted by indy_pack_message (Experimental)
    ///
    /// #Params

    /// wallet_handle: wallet handle (created by open_wallet).
    /// jwe_data: a pointer to the first byte of the JWE to be unpacked
    /// jwe_len: the length of the JWE message in bytes
    ///
    /// #Returns
    /// if authcrypt was used to pack the message returns this json structure:
    /// {
    ///     message: <decrypted message>,
    ///     sender_verkey: <sender_verkey>,
    ///     recipient_verkey: <recipient_verkey>
    /// }
    ///
    /// OR
    ///
    /// if anoncrypt was used to pack the message returns this json structure:
    /// {
    ///     message: <decrypted message>,
    ///     recipient_verkey: <recipient_verkey>
    /// }
    ///
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Ledger*
    /// Crypto*
    pub async fn unpack_msg(
        &self,
        jwe_struct: JWE,
        wallet_handle: WalletHandle,
    ) -> IndyResult<Vec<u8>> {
        //decode protected data
        let protected_decoded_vec = base64::decode_urlsafe(&jwe_struct.protected)?;
        let protected_decoded_str = String::from_utf8(protected_decoded_vec).map_err(|err| {
            err_msg(
                IndyErrorKind::InvalidStructure,
                format!("Failed to utf8 encode data {}", err),
            )
        })?;
        //convert protected_data_str to struct
        let protected_struct: Protected =
            serde_json::from_str(&protected_decoded_str).map_err(|err| {
                err_msg(
                    IndyErrorKind::InvalidStructure,
                    format!("Failed to deserialize protected data {}", err),
                )
            })?;

        //extract recipient that matches a key in the wallet
        let (recipient, is_auth_recipient) = self
            ._find_correct_recipient(protected_struct, wallet_handle)
            .await?;

        //get cek and sender data
        let (sender_verkey_option, cek) = if is_auth_recipient {
            self._unpack_cek_authcrypt(recipient.clone(), wallet_handle)
                .await
        } else {
            self._unpack_cek_anoncrypt(recipient.clone(), wallet_handle)
                .await
        }?; //close cek and sender_data match statement

        //decrypt message
        let message = self.crypto_service.decrypt_ciphertext(
            &jwe_struct.ciphertext,
            &jwe_struct.protected,
            &jwe_struct.iv,
            &jwe_struct.tag,
            &cek,
        )?;

        //serialize and return decrypted message
        let res = UnpackMessage {
            message,
            sender_verkey: sender_verkey_option,
            recipient_verkey: recipient.header.kid,
        };

        serde_json::to_vec(&res).map_err(|err| {
            err_msg(
                IndyErrorKind::InvalidStructure,
                format!("Failed to serialize message {}", err),
            )
        })
    }

    async fn _find_correct_recipient(
        &self,
        protected_struct: Protected,
        wallet_handle: WalletHandle,
    ) -> IndyResult<(Recipient, bool)> {
        for recipient in protected_struct.recipients {
            let my_key_res = self
                .wallet_service
                .get_indy_object::<Key>(
                    wallet_handle,
                    &recipient.header.kid,
                    &RecordOptions::id_value(),
                )
                .await;

            if my_key_res.is_ok() {
                return Ok((recipient.clone(), recipient.header.sender.is_some()));
            }
        }
        Err(IndyError::from(IndyErrorKind::WalletItemNotFound))
    }

    async fn _unpack_cek_authcrypt(
        &self,
        recipient: Recipient,
        wallet_handle: WalletHandle,
    ) -> IndyResult<(Option<String>, chacha20poly1305_ietf::Key)> {
        let encrypted_key_vec = base64::decode_urlsafe(&recipient.encrypted_key)?;
        let iv = base64::decode_urlsafe(&recipient.header.iv.unwrap())?;
        let enc_sender_vk = base64::decode_urlsafe(&recipient.header.sender.unwrap())?;

        //get my private key
        let my_key = self
            .wallet_service
            .get_indy_object(
                wallet_handle,
                &recipient.header.kid,
                &RecordOptions::id_value(),
            )
            .await?;

        //decrypt sender_vk
        let sender_vk_vec = self
            .crypto_service
            .crypto_box_seal_open(&my_key, enc_sender_vk.as_slice())
            .await?;
        let sender_vk = String::from_utf8(sender_vk_vec).map_err(|err| {
            err_msg(
                IndyErrorKind::InvalidStructure,
                format!("Failed to utf-8 encode sender_vk {}", err),
            )
        })?;

        //decrypt cek
        let cek_as_vec = self
            .crypto_service
            .crypto_box_open(
                &my_key,
                &sender_vk,
                encrypted_key_vec.as_slice(),
                iv.as_slice(),
            )
            .await?;

        //convert cek to chacha Key struct
        let cek: chacha20poly1305_ietf::Key =
            chacha20poly1305_ietf::Key::from_slice(&cek_as_vec[..]).map_err(|err| {
                err_msg(
                    IndyErrorKind::InvalidStructure,
                    format!("Failed to decrypt cek {}", err),
                )
            })?;

        Ok((Some(sender_vk), cek))
    }

    async fn _unpack_cek_anoncrypt(
        &self,
        recipient: Recipient,
        wallet_handle: WalletHandle,
    ) -> IndyResult<(Option<String>, chacha20poly1305_ietf::Key)> {
        let encrypted_key_vec = base64::decode_urlsafe(&recipient.encrypted_key)?;

        //get my private key
        let my_key: Key = self
            .wallet_service
            .get_indy_object(
                wallet_handle,
                &recipient.header.kid,
                &RecordOptions::id_value(),
            )
            .await?;

        //decrypt cek
        let cek_as_vec = self
            .crypto_service
            .crypto_box_seal_open(&my_key, encrypted_key_vec.as_slice())
            .await?;

        //convert cek to chacha Key struct
        let cek: chacha20poly1305_ietf::Key =
            chacha20poly1305_ietf::Key::from_slice(&cek_as_vec[..]).map_err(|err| {
                err_msg(
                    IndyErrorKind::InvalidStructure,
                    format!("Failed to decrypt cek {}", err),
                )
            })?;

        Ok((None, cek))
    }
}
