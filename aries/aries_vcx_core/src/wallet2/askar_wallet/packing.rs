use aries_askar::{
    kms::{KeyAlg, KeyEntry, LocalKey, ToDecrypt},
    Session,
};

use aries_askar::crypto::alg::Chacha20Types;
use serde::{Deserialize, Serialize};

use crate::{
    errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult},
    wallet2::{
        crypto_box::{CryptoBox, SodiumCryptoBox},
        utils::{
            bs58_to_bytes, bytes_to_bs58, bytes_to_string, decode_urlsafe, encode_urlsafe,
            from_json_str,
        },
        Key, UnpackedMessage,
    },
};

use super::askar_utils::{local_key_to_private_key_bytes, local_key_to_public_key_bytes};
use aries_askar::kms::KeyAlg::Ed25519;

pub const PROTECTED_HEADER_ENC: &str = "xchacha20poly1305_ietf";
pub const PROTECTED_HEADER_TYP: &str = "JWM/1.0";

#[derive(Serialize, Deserialize, Debug)]
pub struct Jwe {
    pub protected: String,
    pub iv: String,
    pub ciphertext: String,
    pub tag: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum JweAlg {
    Authcrypt,
    Anoncrypt,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProtectedData {
    pub enc: String,
    pub typ: String,
    pub alg: JweAlg,
    pub recipients: Vec<Recipient>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Recipient {
    Authcrypt(AuthcryptRecipient),
    Anoncrypt(AnoncryptRecipient),
}

impl Recipient {
    pub fn new_authcrypt(encrypted_key: &str, kid: &str, iv: &str, sender: &str) -> Self {
        Self::Authcrypt(AuthcryptRecipient {
            encrypted_key: encrypted_key.to_owned(),
            header: AuthcryptHeader {
                kid: kid.into(),
                iv: iv.into(),
                sender: sender.into(),
            },
        })
    }

    pub fn new_anoncrypt(encrypted_key: &str, kid: &str) -> Self {
        Self::Anoncrypt(AnoncryptRecipient {
            encrypted_key: encrypted_key.to_owned(),
            header: AnoncryptHeader { kid: kid.into() },
        })
    }

    pub fn unwrap_kid(&self) -> &str {
        match self {
            Self::Anoncrypt(inner) => &inner.header.kid,
            Self::Authcrypt(inner) => &inner.header.kid,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthcryptRecipient {
    pub encrypted_key: String,
    pub header: AuthcryptHeader,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnoncryptRecipient {
    pub encrypted_key: String,
    pub header: AnoncryptHeader,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthcryptHeader {
    pub kid: String,
    pub iv: String,
    pub sender: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnoncryptHeader {
    pub kid: String,
}

pub struct Packing {
    pub crypto_box: Box<dyn CryptoBox + Send + Sync>,
}

impl Packing {
    pub fn new() -> Self {
        Self {
            crypto_box: Box::new(SodiumCryptoBox {}),
        }
    }

    pub async fn unpack(&self, jwe: Jwe, session: &mut Session) -> VcxCoreResult<UnpackedMessage> {
        let protected_data_vec = decode_urlsafe(&jwe.protected)?;
        let protected_data_str = bytes_to_string(protected_data_vec)?;
        let protected_data = from_json_str(&protected_data_str)?;

        let (recipient, key_entry) = self.find_recipient_key(&protected_data, session).await?;
        let local_key = key_entry.load_local_key()?;

        let (enc_key, sender_verkey) = self.unpack_recipient(recipient, &local_key)?;

        let nonce = decode_urlsafe(&jwe.iv)?;
        let ciphertext = decode_urlsafe(&jwe.ciphertext)?;
        let tag = decode_urlsafe(&jwe.tag)?;

        let to_decrypt = ToDecrypt::from((ciphertext.as_ref(), tag.as_ref()));

        let msg = enc_key.aead_decrypt(to_decrypt, &nonce, &jwe.protected.as_bytes())?;

        let unpacked = UnpackedMessage {
            message: bytes_to_string(msg.to_vec())?,
            recipient_verkey: recipient.unwrap_kid().to_owned(),
            sender_verkey,
        };

        Ok(unpacked)
    }

    fn unpack_recipient(
        &self,
        recipient: &Recipient,
        local_key: &LocalKey,
    ) -> VcxCoreResult<(LocalKey, Option<String>)> {
        let res = match recipient {
            Recipient::Authcrypt(auth_recipient) => {
                self.unpack_authcrypt(&local_key, auth_recipient)
            }
            Recipient::Anoncrypt(anon_recipient) => {
                self.unpack_anoncrypt(&local_key, anon_recipient)
            }
        }?;

        Ok(res)
    }

    fn unpack_authcrypt(
        &self,
        local_key: &LocalKey,
        recipient: &AuthcryptRecipient,
    ) -> VcxCoreResult<(LocalKey, Option<String>)> {
        let encrypted_key = decode_urlsafe(&recipient.encrypted_key)?;
        let iv = decode_urlsafe(&recipient.header.iv)?;

        let sender_vk_enc = decode_urlsafe(&recipient.header.sender)?;

        let private_bytes = local_key_to_private_key_bytes(&local_key)?;
        let public_bytes = local_key_to_public_key_bytes(&local_key)?;
        let all_bytes = [private_bytes, public_bytes.clone()].concat();

        let sender_vk_vec =
            self.crypto_box
                .sealedbox_decrypt(&all_bytes, &public_bytes, &sender_vk_enc)?;

        let cek_vec =
            self.crypto_box
                .box_decrypt(&all_bytes, &sender_vk_vec, &encrypted_key, &iv)?;

        let sender_vk = bytes_to_bs58(&sender_vk_vec);

        let enc_key = LocalKey::from_secret_bytes(KeyAlg::Chacha20(Chacha20Types::C20P), &cek_vec)?;

        Ok((enc_key, Some(sender_vk)))
    }

    fn unpack_anoncrypt(
        &self,
        local_key: &LocalKey,
        recipient: &AnoncryptRecipient,
    ) -> VcxCoreResult<(LocalKey, Option<String>)> {
        let encrypted_key = decode_urlsafe(&recipient.encrypted_key)?;

        let private_bytes = local_key_to_private_key_bytes(&local_key)?;
        let public_bytes = local_key_to_public_key_bytes(&local_key)?;
        let all_bytes = [private_bytes, public_bytes.clone()].concat();

        let cek_vec =
            self.crypto_box
                .sealedbox_decrypt(&all_bytes, &public_bytes, &encrypted_key)?;
        let enc_key = LocalKey::from_secret_bytes(KeyAlg::Chacha20(Chacha20Types::C20P), &cek_vec)?;

        Ok((enc_key, None))
    }

    async fn find_recipient_key<'a>(
        &self,
        protected_data: &'a ProtectedData,
        session: &mut Session,
    ) -> VcxCoreResult<(&'a Recipient, KeyEntry)> {
        for recipient in protected_data.recipients.iter() {
            if let Some(key_entry) = session.fetch_key(&recipient.unwrap_kid(), false).await? {
                return Ok((recipient, key_entry));
            };
        }

        Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::WalletRecordNotFound,
            "recipient key not found in wallet",
        ))
    }

    pub fn pack_all(
        &self,
        base64_data: &str,
        enc_key: LocalKey,
        msg: &[u8],
    ) -> VcxCoreResult<Vec<u8>> {
        let nonce = enc_key.aead_random_nonce()?;
        let enc = enc_key.aead_encrypt(msg, &nonce, base64_data.as_bytes())?;

        let jwe = Jwe {
            protected: base64_data.to_string(),
            iv: encode_urlsafe(enc.nonce()),
            ciphertext: encode_urlsafe(enc.ciphertext()),
            tag: encode_urlsafe(enc.tag()),
        };

        serde_json::to_vec(&jwe).map_err(|err| {
            AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::EncodeError,
                format!("Failed to serialize JWE {}", err),
            )
        })
    }

    pub fn pack_authcrypt(
        &self,
        enc_key: &LocalKey,
        recipient_keys: Vec<Key>,
        sender_local_key: LocalKey,
    ) -> VcxCoreResult<String> {
        self.check_supported_key_alg(&sender_local_key)?;

        let encrypted_recipients =
            self.pack_authcrypt_recipients(enc_key, recipient_keys, sender_local_key)?;

        Ok(self.encode_protected_data(encrypted_recipients, JweAlg::Authcrypt)?)
    }

    fn check_supported_key_alg(&self, key: &LocalKey) -> VcxCoreResult<()> {
        let supported_algs = vec![Ed25519];

        if !supported_algs.contains(&key.algorithm()) {
            let msg = format!(
                "Unsupported key algorithm, expected one of: {}",
                supported_algs
                    .into_iter()
                    .map(|alg| alg.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            return Err(AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidOption,
                msg,
            ));
        }
        Ok(())
    }

    fn pack_authcrypt_recipients(
        &self,
        enc_key: &LocalKey,
        recipient_keys: Vec<Key>,
        sender_local_key: LocalKey,
    ) -> VcxCoreResult<Vec<Recipient>> {
        let my_secret_bytes = local_key_to_private_key_bytes(&sender_local_key)?;
        let my_public_bytes = local_key_to_public_key_bytes(&sender_local_key)?;
        let my_all_bytes = [my_secret_bytes, my_public_bytes.clone()].concat();

        let enc_key_secret = local_key_to_private_key_bytes(enc_key)?;

        let mut encrypted_recipients = Vec::with_capacity(recipient_keys.len());

        for recipient_key in recipient_keys {
            let recipient_pubkey = bs58_to_bytes(&recipient_key.pubkey_bs58)?;

            let (enc_cek, nonce) =
                self.crypto_box
                    .box_encrypt(&my_all_bytes, &recipient_pubkey, &enc_key_secret)?;

            let enc_sender = self
                .crypto_box
                .sealedbox_encrypt(&recipient_pubkey, &my_public_bytes)?;

            let kid = bytes_to_bs58(&recipient_pubkey);
            let sender = encode_urlsafe(&enc_sender);
            let iv = encode_urlsafe(&nonce);

            encrypted_recipients.push(Recipient::new_authcrypt(
                &encode_urlsafe(&enc_cek),
                &kid,
                &iv,
                &sender,
            ));
        }

        Ok(encrypted_recipients)
    }

    pub fn pack_anoncrypt(
        &self,
        enc_key: &LocalKey,
        recipient_keys: Vec<Key>,
    ) -> VcxCoreResult<String> {
        let encrypted_recipients = self.pack_anoncrypt_recipients(enc_key, recipient_keys)?;

        Ok(self.encode_protected_data(encrypted_recipients, JweAlg::Anoncrypt)?)
    }

    fn pack_anoncrypt_recipients(
        &self,
        enc_key: &LocalKey,
        recipient_keys: Vec<Key>,
    ) -> VcxCoreResult<Vec<Recipient>> {
        let mut encrypted_recipients = Vec::with_capacity(recipient_keys.len());

        let enc_key_secret = local_key_to_private_key_bytes(enc_key)?;

        for recipient_key in recipient_keys {
            let recipient_pubkey = bs58_to_bytes(&recipient_key.pubkey_bs58)?;

            let enc_cek = self
                .crypto_box
                .sealedbox_encrypt(&recipient_pubkey, &enc_key_secret)?;

            let kid = bytes_to_bs58(&recipient_pubkey);

            encrypted_recipients.push(Recipient::new_anoncrypt(&encode_urlsafe(&enc_cek), &kid));
        }

        Ok(encrypted_recipients)
    }

    fn encode_protected_data(
        &self,
        encrypted_recipients: Vec<Recipient>,
        jwe_alg: JweAlg,
    ) -> VcxCoreResult<String> {
        let protected_data = ProtectedData {
            enc: PROTECTED_HEADER_ENC.into(),
            typ: PROTECTED_HEADER_TYP.into(),
            alg: jwe_alg,
            recipients: encrypted_recipients,
        };

        let protected_encoded = serde_json::to_string(&protected_data).map_err(|err| {
            AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::EncodeError,
                format!("Failed to serialize protected field {}", err),
            )
        })?;

        Ok(encode_urlsafe(protected_encoded.as_bytes()))
    }
}
