use aries_askar::{
    crypto::alg::Chacha20Types,
    kms::{KeyAlg, KeyAlg::Ed25519, KeyEntry, LocalKey, ToDecrypt},
    Session,
};
use public_key::{Key, KeyType};

use super::{
    askar_utils::{
        bs58_to_bytes, bytes_to_bs58, bytes_to_string, ed25519_to_x25519_pair,
        ed25519_to_x25519_private, ed25519_to_x25519_public, from_json_str,
        local_key_to_private_key_bytes, local_key_to_public_key_bytes,
    },
    packing_types::{
        AnoncryptRecipient, AuthcryptRecipient, Base64String, Jwe, JweAlg, ProtectedData,
        ProtectedHeaderEnc, ProtectedHeaderTyp, Recipient,
    },
};
use crate::{
    errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult},
    wallet::{
        askar::crypto_box::{CryptoBox, SodiumCryptoBox},
        structs_io::UnpackMessageOutput,
    },
};

pub struct Packing {
    crypto_box: Box<dyn CryptoBox + Send + Sync>,
}

impl Packing {
    pub fn new() -> Self {
        Self {
            crypto_box: Box::new(SodiumCryptoBox::new()),
        }
    }

    pub async fn unpack(
        &self,
        jwe: Jwe,
        session: &mut Session,
    ) -> VcxCoreResult<UnpackMessageOutput> {
        let protected_data = self.unpack_protected_data(&jwe)?;
        let (recipient, key_entry) = self.find_recipient_key(&protected_data, session).await?;
        let local_key = key_entry.load_local_key()?;
        let (enc_key, sender_verkey) = self.unpack_recipient(recipient, &local_key)?;
        Ok(UnpackMessageOutput {
            message: self.unpack_msg(&jwe, enc_key)?,
            recipient_verkey: recipient.unwrap_kid().to_owned(),
            sender_verkey: sender_verkey.map(|key| key.base58()),
        })
    }

    pub fn unpack_protected_data(&self, jwe: &Jwe) -> VcxCoreResult<ProtectedData> {
        from_json_str(&jwe.protected.decode_to_string()?)
    }

    pub fn unpack_msg(&self, jwe: &Jwe, enc_key: LocalKey) -> VcxCoreResult<String> {
        let ciphertext = &jwe.ciphertext.decode()?;
        let tag = &jwe.tag.decode()?;

        bytes_to_string(
            enc_key
                .aead_decrypt(
                    ToDecrypt::from((ciphertext.as_ref(), tag.as_ref())),
                    &jwe.iv.decode()?,
                    &jwe.protected.as_bytes(),
                )?
                .to_vec(),
        )
    }

    fn unpack_recipient(
        &self,
        recipient: &Recipient,
        local_key: &LocalKey,
    ) -> VcxCoreResult<(LocalKey, Option<Key>)> {
        match recipient {
            Recipient::Authcrypt(auth_recipient) => {
                self.unpack_authcrypt(local_key, auth_recipient)
            }
            Recipient::Anoncrypt(anon_recipient) => {
                self.unpack_anoncrypt(local_key, anon_recipient)
            }
        }
    }

    fn unpack_authcrypt(
        &self,
        local_key: &LocalKey,
        recipient: &AuthcryptRecipient,
    ) -> VcxCoreResult<(LocalKey, Option<Key>)> {
        let (private_bytes, public_bytes) = ed25519_to_x25519_pair(local_key)?;
        let sender_vk_vec = self.crypto_box.sealedbox_decrypt(
            &private_bytes,
            &public_bytes,
            &recipient.header.sender.decode()?,
        )?;
        Ok((
            LocalKey::from_secret_bytes(
                KeyAlg::Chacha20(Chacha20Types::C20P),
                &self.crypto_box.box_decrypt(
                    &private_bytes,
                    &ed25519_to_x25519_public(&LocalKey::from_public_bytes(
                        Ed25519,
                        &bs58_to_bytes(&bytes_to_string(sender_vk_vec.clone())?)?,
                    )?)?,
                    &recipient.encrypted_key.decode()?,
                    &recipient.header.iv.decode()?,
                )?,
            )?,
            Some(Key::new(sender_vk_vec, KeyType::Ed25519)?),
        ))
    }

    fn unpack_anoncrypt(
        &self,
        local_key: &LocalKey,
        recipient: &AnoncryptRecipient,
    ) -> VcxCoreResult<(LocalKey, Option<Key>)> {
        let (private_bytes, public_bytes) = ed25519_to_x25519_pair(local_key)?;
        Ok((
            LocalKey::from_secret_bytes(
                KeyAlg::Chacha20(Chacha20Types::C20P),
                &self.crypto_box.sealedbox_decrypt(
                    &private_bytes,
                    &public_bytes,
                    &recipient.encrypted_key.decode()?,
                )?,
            )?,
            None,
        ))
    }

    async fn find_recipient_key<'a>(
        &self,
        protected_data: &'a ProtectedData,
        session: &mut Session,
    ) -> VcxCoreResult<(&'a Recipient, KeyEntry)> {
        for recipient in protected_data.recipients.iter() {
            if let Some(key_entry) = session.fetch_key(recipient.key_name(), false).await? {
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
        base64_data: Base64String,
        enc_key: LocalKey,
        msg: &[u8],
    ) -> VcxCoreResult<Vec<u8>> {
        let enc =
            enc_key.aead_encrypt(msg, &enc_key.aead_random_nonce()?, &base64_data.as_bytes())?;
        serde_json::to_vec(&Jwe {
            protected: base64_data,
            iv: Base64String::from_bytes(enc.nonce()),
            ciphertext: Base64String::from_bytes(enc.ciphertext()),
            tag: Base64String::from_bytes(enc.tag()),
        })
        .map_err(|err| {
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
    ) -> VcxCoreResult<Base64String> {
        self.check_supported_key_alg(&sender_local_key)?;
        self.encode_protected_data(
            self.pack_authcrypt_recipients(enc_key, recipient_keys, sender_local_key)?,
            JweAlg::Authcrypt,
        )
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
            Err(AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidOption,
                msg,
            ))
        } else {
            Ok(())
        }
    }

    fn pack_authcrypt_recipients(
        &self,
        enc_key: &LocalKey,
        recipient_keys: Vec<Key>,
        sender_local_key: LocalKey,
    ) -> VcxCoreResult<Vec<Recipient>> {
        let mut encrypted_recipients = Vec::with_capacity(recipient_keys.len());

        for recipient_key in recipient_keys {
            let recipient_public_bytes = ed25519_to_x25519_public(&LocalKey::from_public_bytes(
                Ed25519,
                recipient_key.key(),
            )?)?;

            let (enc_cek, nonce) = self.crypto_box.box_encrypt(
                &ed25519_to_x25519_private(&sender_local_key)?,
                &recipient_public_bytes,
                &local_key_to_private_key_bytes(enc_key)?,
            )?;

            let enc_sender = self.crypto_box.sealedbox_encrypt(
                &recipient_public_bytes,
                bytes_to_bs58(&local_key_to_public_key_bytes(&sender_local_key)?).as_bytes(),
            )?;

            encrypted_recipients.push(Recipient::new_authcrypt(
                Base64String::from_bytes(&enc_cek),
                &recipient_key.base58(),
                Base64String::from_bytes(&nonce),
                Base64String::from_bytes(&enc_sender),
            ));
        }

        Ok(encrypted_recipients)
    }

    pub fn pack_anoncrypt(
        &self,
        enc_key: &LocalKey,
        recipient_keys: Vec<Key>,
    ) -> VcxCoreResult<Base64String> {
        let encrypted_recipients = self.pack_anoncrypt_recipients(enc_key, recipient_keys)?;

        self.encode_protected_data(encrypted_recipients, JweAlg::Anoncrypt)
    }

    fn pack_anoncrypt_recipients(
        &self,
        enc_key: &LocalKey,
        recipient_keys: Vec<Key>,
    ) -> VcxCoreResult<Vec<Recipient>> {
        let mut encrypted_recipients = Vec::with_capacity(recipient_keys.len());

        let enc_key_secret = local_key_to_private_key_bytes(enc_key)?;

        for recipient_key in recipient_keys {
            let recipient_pubkey = bs58_to_bytes(&recipient_key.base58())?;
            let recipient_local_key = LocalKey::from_public_bytes(Ed25519, &recipient_pubkey)?;
            let recipient_public_bytes = ed25519_to_x25519_public(&recipient_local_key)?;

            let enc_cek = self
                .crypto_box
                .sealedbox_encrypt(&recipient_public_bytes, &enc_key_secret)?;

            let kid = bytes_to_bs58(&recipient_pubkey);

            encrypted_recipients.push(Recipient::new_anoncrypt(
                Base64String::from_bytes(&enc_cek),
                &kid,
            ));
        }

        Ok(encrypted_recipients)
    }

    fn encode_protected_data(
        &self,
        encrypted_recipients: Vec<Recipient>,
        jwe_alg: JweAlg,
    ) -> VcxCoreResult<Base64String> {
        let protected_data = ProtectedData {
            enc: ProtectedHeaderEnc::XChaCha20Poly1305,
            typ: ProtectedHeaderTyp::Jwm,
            alg: jwe_alg,
            recipients: encrypted_recipients,
        };

        let protected_encoded = serde_json::to_string(&protected_data).map_err(|err| {
            AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::EncodeError,
                format!("Failed to serialize protected field {}", err),
            )
        })?;

        Ok(Base64String::from_bytes(protected_encoded.as_bytes()))
    }
}
