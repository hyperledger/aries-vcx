use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};

use sodiumoxide::crypto::box_::{self, Nonce, PublicKey, SecretKey};
use sodiumoxide::crypto::sealedbox;

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
        let sk_bytes = private_key
            .try_into()
            .map_err(|err| AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err))?;

        let pk_bytes = public_key
            .try_into()
            .map_err(|err| AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err))?;

        let sk = SecretKey(sk_bytes);
        let pk = PublicKey(pk_bytes);

        let nonce = box_::gen_nonce();

        let res = box_::seal(msg, &nonce, &pk, &sk);

        Ok((res, nonce.0.to_vec()))
    }

    fn box_decrypt(
        &self,
        private_key: &[u8],
        public_key: &[u8],
        msg: &[u8],
        iv: &[u8],
    ) -> VcxCoreResult<Vec<u8>> {
        let sk_bytes = private_key
            .try_into()
            .map_err(|err| AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err))?;

        let pk_bytes = public_key
            .try_into()
            .map_err(|err| AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err))?;

        let sk = SecretKey(sk_bytes);
        let pk = PublicKey(pk_bytes);

        let nonce_bytes = iv
            .try_into()
            .map_err(|err| AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err))?;

        let nonce = Nonce(nonce_bytes);

        Ok(box_::open(msg, &nonce, &pk, &sk).map_err(|_| {
            AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, "failed to open box")
        })?)
    }

    fn sealedbox_encrypt(&self, public_key: &[u8], msg: &[u8]) -> VcxCoreResult<Vec<u8>> {
        let pk_bytes = public_key
            .try_into()
            .map_err(|err| AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err))?;

        let pk = box_::PublicKey(pk_bytes);

        Ok(sealedbox::seal(msg, &pk))
    }

    fn sealedbox_decrypt(
        &self,
        private_key: &[u8],
        public_key: &[u8],
        msg: &[u8],
    ) -> VcxCoreResult<Vec<u8>> {
        let sk_bytes = private_key
            .try_into()
            .map_err(|err| AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err))?;

        let pk_bytes = public_key
            .try_into()
            .map_err(|err| AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err))?;

        let sk = SecretKey(sk_bytes);
        let pk = PublicKey(pk_bytes);

        Ok(sealedbox::open(msg, &pk, &sk).map_err(|_| {
            AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidInput,
                "failed to open sealed box",
            )
        })?)
    }
}

#[cfg(test)]
mod test {
    use aries_askar::kms::KeyAlg::X25519;
    use aries_askar::kms::LocalKey;

    use crate::wallet2::{
        askar_wallet::askar_utils::{
            local_key_to_private_key_bytes, local_key_to_public_key_bytes,
        },
        crypto_box::{CryptoBox, SodiumCryptoBox},
        utils::bytes_to_string,
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
