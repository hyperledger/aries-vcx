use aries_askar::{
    crypto::alg::Chacha20Types,
    kms::{
        KeyAlg::{self, Ed25519},
        KeyEntry, LocalKey, ToDecrypt,
    },
    Session,
};

use public_key::{Key, KeyType};

use crate::{
    errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult},
    wallet::structs_io::UnpackMessageOutput,
};

use super::{
    askar_utils::{
        bs58_to_bytes, bytes_to_string, ed25519_to_x25519_pair, ed25519_to_x25519_public,
        from_json_str,
    },
    crypto_box::{CryptoBox, SodiumCryptoBox},
    packing_types::{AnoncryptRecipient, AuthcryptRecipient, Jwe, ProtectedData, Recipient},
};

trait Unpack {
    fn unpack(&self, recipient: &Recipient, jwe: Jwe) -> VcxCoreResult<UnpackMessageOutput>;
}

impl Unpack for LocalKey {
    fn unpack(&self, recipient: &Recipient, jwe: Jwe) -> VcxCoreResult<UnpackMessageOutput> {
        let (enc_key, sender_verkey) = unpack_recipient(recipient, self)?;
        Ok(UnpackMessageOutput {
            message: unpack_msg(&jwe, enc_key)?,
            recipient_verkey: recipient.unwrap_kid().to_owned(),
            sender_verkey: sender_verkey.map(|key| key.base58()),
        })
    }
}

pub async fn unpack(jwe: Jwe, session: &mut Session) -> VcxCoreResult<UnpackMessageOutput> {
    let protected_data = unpack_protected_data(&jwe)?;
    let (recipient, key_entry) = find_recipient_key(&protected_data, session).await?;
    let local_key = key_entry.load_local_key()?;
    local_key.unpack(recipient, jwe)
}

fn unpack_recipient(
    recipient: &Recipient,
    local_key: &LocalKey,
) -> VcxCoreResult<(LocalKey, Option<Key>)> {
    match recipient {
        Recipient::Authcrypt(auth_recipient) => unpack_authcrypt(local_key, auth_recipient),
        Recipient::Anoncrypt(anon_recipient) => unpack_anoncrypt(local_key, anon_recipient),
    }
}

fn unpack_protected_data(jwe: &Jwe) -> VcxCoreResult<ProtectedData> {
    from_json_str(&jwe.protected.decode_to_string()?)
}

fn unpack_msg(jwe: &Jwe, enc_key: LocalKey) -> VcxCoreResult<String> {
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

fn unpack_authcrypt(
    local_key: &LocalKey,
    recipient: &AuthcryptRecipient,
) -> VcxCoreResult<(LocalKey, Option<Key>)> {
    let (private_bytes, public_bytes) = ed25519_to_x25519_pair(local_key)?;

    let crypto_box = SodiumCryptoBox::new();
    let sender_vk_vec = crypto_box.sealedbox_decrypt(
        &private_bytes,
        &public_bytes,
        &recipient.header.sender.decode()?,
    )?;
    Ok((
        LocalKey::from_secret_bytes(
            KeyAlg::Chacha20(Chacha20Types::C20P),
            &crypto_box.box_decrypt(
                &private_bytes,
                &ed25519_to_x25519_public(&LocalKey::from_public_bytes(
                    Ed25519,
                    &bs58_to_bytes(&sender_vk_vec.clone())?,
                )?)?,
                &recipient.encrypted_key.decode()?,
                &recipient.header.iv.decode()?,
            )?,
        )?,
        Some(Key::new(sender_vk_vec, KeyType::Ed25519)?),
    ))
}

fn unpack_anoncrypt(
    local_key: &LocalKey,
    recipient: &AnoncryptRecipient,
) -> VcxCoreResult<(LocalKey, Option<Key>)> {
    let (private_bytes, public_bytes) = ed25519_to_x25519_pair(local_key)?;

    let crypto_box = SodiumCryptoBox::new();
    Ok((
        LocalKey::from_secret_bytes(
            KeyAlg::Chacha20(Chacha20Types::C20P),
            &crypto_box.sealedbox_decrypt(
                &private_bytes,
                &public_bytes,
                &recipient.encrypted_key.decode()?,
            )?,
        )?,
        None,
    ))
}

async fn find_recipient_key<'a>(
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
