use sodiumoxide::crypto::{
    box_::{self, Nonce},
    sealedbox,
};

use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};

pub trait CryptoBox {
    fn box_encrypt(
        &self,
        private_key: &[u8],
        public_key: &[u8],
        msg: &[u8],
    ) -> VcxCoreResult<(Vec<u8>, Vec<u8>)>;

    fn box_decrypt(
        &self,
        private_key: &[u8],
        public_key: &[u8],
        msg: &[u8],
        iv: &[u8],
    ) -> VcxCoreResult<Vec<u8>>;

    fn sealedbox_encrypt(&self, public_key: &[u8], msg: &[u8]) -> VcxCoreResult<Vec<u8>>;

    fn sealedbox_decrypt(
        &self,
        secret_key: &[u8],
        public_key: &[u8],
        msg: &[u8],
    ) -> VcxCoreResult<Vec<u8>>;
}

pub struct SodiumCryptoBox {}

impl SodiumCryptoBox {
    pub fn new() -> Self {
        Self {}
    }
}

impl CryptoBox for SodiumCryptoBox {
    fn box_encrypt(
        &self,
        private_key: &[u8],
        public_key: &[u8],
        msg: &[u8],
    ) -> VcxCoreResult<(Vec<u8>, Vec<u8>)> {
        let nonce = box_::gen_nonce();
        Ok((
            box_::seal(
                msg,
                &nonce,
                &box_::PublicKey(public_key.try_into().map_err(|err| {
                    AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err)
                })?),
                &box_::SecretKey(private_key.try_into().map_err(|err| {
                    AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err)
                })?),
            ),
            nonce.0.to_vec(),
        ))
    }

    fn box_decrypt(
        &self,
        private_key: &[u8],
        public_key: &[u8],
        msg: &[u8],
        iv: &[u8],
    ) -> VcxCoreResult<Vec<u8>> {
        box_::open(
            msg,
            &Nonce(iv.try_into().map_err(|err| {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err)
            })?),
            &box_::PublicKey(public_key.try_into().map_err(|err| {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err)
            })?),
            &box_::SecretKey(private_key.try_into().map_err(|err| {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err)
            })?),
        )
        .map_err(|_| {
            AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, "failed to open box")
        })
    }

    fn sealedbox_encrypt(&self, public_key: &[u8], msg: &[u8]) -> VcxCoreResult<Vec<u8>> {
        Ok(sealedbox::seal(
            msg,
            &box_::PublicKey(public_key.try_into().map_err(|err| {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err)
            })?),
        ))
    }

    fn sealedbox_decrypt(
        &self,
        private_key: &[u8],
        public_key: &[u8],
        msg: &[u8],
    ) -> VcxCoreResult<Vec<u8>> {
        sealedbox::open(
            msg,
            &box_::PublicKey(public_key.try_into().map_err(|err| {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err)
            })?),
            &box_::SecretKey(private_key.try_into().map_err(|err| {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err)
            })?),
        )
        .map_err(|_| {
            AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidInput,
                "failed to open sealed box",
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use aries_askar::kms::{KeyAlg::X25519, LocalKey};

    use crate::wallet::askar::{
        askar_utils::{
            bytes_to_string, local_key_to_private_key_bytes, local_key_to_public_key_bytes,
        },
        crypto_box::{CryptoBox, SodiumCryptoBox},
    };

    #[test]
    fn test_box_should_encrypt_and_decrypt() {
        let sender_key = LocalKey::generate(X25519, true).unwrap();
        let sender_private_bytes = local_key_to_private_key_bytes(&sender_key).unwrap();
        let recipient_key = LocalKey::generate(X25519, true).unwrap();
        let recipient_public_bytes = local_key_to_public_key_bytes(&recipient_key).unwrap();
        let msg = "secret";
        let crypto_box = SodiumCryptoBox::new();
        let (enc, nonce) = crypto_box
            .box_encrypt(
                &sender_private_bytes,
                &recipient_public_bytes,
                msg.as_bytes(),
            )
            .unwrap();
        let recipient_private_bytes = local_key_to_private_key_bytes(&recipient_key).unwrap();
        let sender_public_bytes = local_key_to_public_key_bytes(&sender_key).unwrap();
        let res = crypto_box
            .box_decrypt(&recipient_private_bytes, &sender_public_bytes, &enc, &nonce)
            .unwrap();
        assert_eq!(msg, bytes_to_string(res).unwrap());
    }

    #[test]
    fn test_sealedbox_should_encrypt_and_decrypt() {
        let sender_key = LocalKey::generate(X25519, false).unwrap();
        let sender_private_bytes = local_key_to_private_key_bytes(&sender_key).unwrap();
        let sender_public_bytes = local_key_to_public_key_bytes(&sender_key).unwrap();
        let crypto_box = SodiumCryptoBox::new();
        let msg = "secret";
        let enc = crypto_box
            .sealedbox_encrypt(&sender_public_bytes, msg.as_bytes())
            .unwrap();

        let dec = crypto_box
            .sealedbox_decrypt(&sender_private_bytes, &sender_public_bytes, &enc)
            .unwrap();

        assert_eq!(msg, bytes_to_string(dec).unwrap());
    }
}
