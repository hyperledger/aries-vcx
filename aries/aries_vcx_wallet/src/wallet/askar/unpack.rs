use aries_askar::{
    crypto::alg::Chacha20Types,
    kms::{
        crypto_box_open, crypto_box_seal_open,
        KeyAlg::{self, Ed25519},
        KeyEntry, LocalKey, ToDecrypt,
    },
    Session,
};
use public_key::{Key, KeyType};

use super::{
    askar_utils::{ed25519_to_x25519, from_json_str},
    packing_types::{AnoncryptRecipient, AuthcryptRecipient, Jwe, ProtectedData, Recipient},
};
use crate::{
    errors::error::{VcxWalletError, VcxWalletResult},
    wallet::{
        structs_io::UnpackMessageOutput,
        utils::{bs58_to_bytes, bytes_to_string},
    },
};

trait Unpack {
    fn unpack(&self, recipient: &Recipient, jwe: Jwe) -> VcxWalletResult<UnpackMessageOutput>;
}

impl Unpack for LocalKey {
    fn unpack(&self, recipient: &Recipient, jwe: Jwe) -> VcxWalletResult<UnpackMessageOutput> {
        let (enc_key, sender_verkey) = unpack_recipient(recipient, self)?;
        Ok(UnpackMessageOutput {
            message: unpack_msg(&jwe, enc_key)?,
            recipient_verkey: recipient.unwrap_kid().to_owned(),
            sender_verkey: sender_verkey.map(|key| key.base58()),
        })
    }
}

pub async fn unpack(jwe: Jwe, session: &mut Session) -> VcxWalletResult<UnpackMessageOutput> {
    let protected_data = unpack_protected_data(&jwe)?;
    let (recipient, key_entry) = find_recipient_key(&protected_data, session).await?;
    let local_key = key_entry.load_local_key()?;
    local_key.unpack(recipient, jwe)
}

fn unpack_recipient(
    recipient: &Recipient,
    local_key: &LocalKey,
) -> VcxWalletResult<(LocalKey, Option<Key>)> {
    match recipient {
        Recipient::Authcrypt(auth_recipient) => unpack_authcrypt(local_key, auth_recipient),
        Recipient::Anoncrypt(anon_recipient) => unpack_anoncrypt(local_key, anon_recipient),
    }
}

fn unpack_protected_data(jwe: &Jwe) -> VcxWalletResult<ProtectedData> {
    from_json_str(&jwe.protected.decode_to_string()?)
}

fn unpack_msg(jwe: &Jwe, enc_key: LocalKey) -> VcxWalletResult<String> {
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
) -> VcxWalletResult<(LocalKey, Option<Key>)> {
    let recipient_key = ed25519_to_x25519(local_key)?;
    let sender_vk = crypto_box_seal_open(&recipient_key, &recipient.header.sender.decode()?)?;
    let sender_key = ed25519_to_x25519(&LocalKey::from_public_bytes(
        Ed25519,
        &bs58_to_bytes(&sender_vk.clone())?,
    )?)?;

    let secret = crypto_box_open(
        &recipient_key,
        &sender_key,
        &recipient.encrypted_key.decode()?,
        &recipient.header.iv.decode()?,
    )?;

    Ok((
        LocalKey::from_secret_bytes(KeyAlg::Chacha20(Chacha20Types::C20P), &secret)?,
        Some(Key::new(sender_vk.to_vec(), KeyType::Ed25519)?),
    ))
}

fn unpack_anoncrypt(
    local_key: &LocalKey,
    recipient: &AnoncryptRecipient,
) -> VcxWalletResult<(LocalKey, Option<Key>)> {
    let recipient_key = ed25519_to_x25519(local_key)?;
    let key = crypto_box_seal_open(&recipient_key, &recipient.encrypted_key.decode()?)?;

    Ok((
        LocalKey::from_secret_bytes(KeyAlg::Chacha20(Chacha20Types::C20P), &key)?,
        None,
    ))
}

async fn find_recipient_key<'a>(
    protected_data: &'a ProtectedData,
    session: &mut Session,
) -> VcxWalletResult<(&'a Recipient, KeyEntry)> {
    for recipient in protected_data.recipients.iter() {
        if let Some(key_entry) = session.fetch_key(recipient.unwrap_kid(), false).await? {
            return Ok((recipient, key_entry));
        };
    }

    Err(VcxWalletError::NoRecipientKeyFound)
}
