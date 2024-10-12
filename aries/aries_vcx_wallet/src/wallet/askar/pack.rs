use aries_askar::kms::{
    crypto_box, crypto_box_random_nonce, crypto_box_seal, KeyAlg::Ed25519, LocalKey,
};
use public_key::Key;

use super::{
    askar_utils::ed25519_to_x25519,
    packing_types::{
        Jwe, JweAlg, ProtectedData, ProtectedHeaderEnc, ProtectedHeaderTyp, Recipient,
    },
};
use crate::{
    errors::error::{VcxWalletError, VcxWalletResult},
    wallet::{
        base_wallet::base64_string::Base64String,
        utils::{bs58_to_bytes, bytes_to_bs58},
    },
};

fn check_supported_key_alg(key: &LocalKey) -> VcxWalletResult<()> {
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
        Err(VcxWalletError::InvalidInput(msg))
    } else {
        Ok(())
    }
}

fn encode_protected_data(
    encrypted_recipients: Vec<Recipient>,
    jwe_alg: JweAlg,
) -> VcxWalletResult<Base64String> {
    let protected_data = ProtectedData {
        enc: ProtectedHeaderEnc::XChaCha20Poly1305,
        typ: ProtectedHeaderTyp::Jwm,
        alg: jwe_alg,
        recipients: encrypted_recipients,
    };

    let protected_encoded = serde_json::to_string(&protected_data)?;

    Ok(Base64String::from_bytes(protected_encoded.as_bytes()))
}

fn pack_authcrypt_recipients(
    enc_key: &LocalKey,
    recipient_keys: Vec<Key>,
    sender_local_key: LocalKey,
) -> VcxWalletResult<Vec<Recipient>> {
    let mut encrypted_recipients = Vec::with_capacity(recipient_keys.len());

    let sender_converted_key = ed25519_to_x25519(&sender_local_key)?;

    for recipient_key in recipient_keys {
        let recipient_public_key = &LocalKey::from_public_bytes(Ed25519, recipient_key.key())?;

        let nonce = crypto_box_random_nonce()?;
        let recipient_converted_key = ed25519_to_x25519(recipient_public_key)?;

        let enc_cek = crypto_box(
            &recipient_converted_key,
            &sender_converted_key,
            &enc_key.to_secret_bytes()?,
            &nonce,
        )?;

        let sender_ed25519_pk = sender_local_key.to_public_bytes()?;
        let enc_sender = crypto_box_seal(
            &recipient_converted_key,
            bytes_to_bs58(&sender_ed25519_pk).as_bytes(),
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

fn pack_anoncrypt_recipients(
    enc_key: &LocalKey,
    recipient_keys: Vec<Key>,
) -> VcxWalletResult<Vec<Recipient>> {
    let mut encrypted_recipients = Vec::with_capacity(recipient_keys.len());

    let enc_key_secret = &enc_key.to_secret_bytes()?;

    for recipient_key in recipient_keys {
        let recipient_pubkey = bs58_to_bytes(recipient_key.base58().as_bytes())?;
        let recipient_local_key = LocalKey::from_public_bytes(Ed25519, &recipient_pubkey)?;
        let enc_cek = crypto_box_seal(&ed25519_to_x25519(&recipient_local_key)?, enc_key_secret)?;

        let kid = bytes_to_bs58(&recipient_pubkey);

        encrypted_recipients.push(Recipient::new_anoncrypt(
            Base64String::from_bytes(&enc_cek),
            &kid,
        ));
    }

    Ok(encrypted_recipients)
}

pub trait Pack {
    fn pack_authcrypt(
        &self,
        recipient_keys: Vec<Key>,
        sender_local_key: LocalKey,
    ) -> VcxWalletResult<Base64String>;

    fn pack_anoncrypt(&self, recipient_keys: Vec<Key>) -> VcxWalletResult<Base64String>;

    fn pack_all(&self, base64_data: Base64String, msg: &[u8]) -> VcxWalletResult<Vec<u8>>;
}

impl Pack for LocalKey {
    fn pack_authcrypt(
        &self,
        recipient_keys: Vec<Key>,
        sender_local_key: LocalKey,
    ) -> VcxWalletResult<Base64String> {
        check_supported_key_alg(&sender_local_key)?;
        encode_protected_data(
            pack_authcrypt_recipients(self, recipient_keys, sender_local_key)?,
            JweAlg::Authcrypt,
        )
    }

    fn pack_anoncrypt(&self, recipient_keys: Vec<Key>) -> VcxWalletResult<Base64String> {
        let encrypted_recipients = pack_anoncrypt_recipients(self, recipient_keys)?;

        encode_protected_data(encrypted_recipients, JweAlg::Anoncrypt)
    }

    fn pack_all(&self, base64_data: Base64String, msg: &[u8]) -> VcxWalletResult<Vec<u8>> {
        let enc = self.aead_encrypt(msg, &self.aead_random_nonce()?, &base64_data.as_bytes())?;
        Ok(serde_json::to_vec(&Jwe {
            protected: base64_data,
            iv: Base64String::from_bytes(enc.nonce()),
            ciphertext: Base64String::from_bytes(enc.ciphertext()),
            tag: Base64String::from_bytes(enc.tag()),
        })?)
    }
}
