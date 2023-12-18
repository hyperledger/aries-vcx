use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};

use libc::c_int;
use sodiumoxide::crypto::box_::{self, Nonce};
use sodiumoxide::crypto::sealedbox;
use sodiumoxide::crypto::sign;

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

pub const SIG_PUBLICKEYBYTES: usize = sign::PUBLICKEYBYTES;
pub const ENC_PUBLICKEYBYTES: usize = box_::PUBLICKEYBYTES;
pub const SIG_SECRETKEYBYTES: usize = sign::SECRETKEYBYTES;
pub const ENC_SECRETKEYBYTES: usize = box_::SECRETKEYBYTES;

extern "C" {
    // these functions are not exposed in sodium 0.0.16,
    // local binding is used to call libsodium-sys function
    pub fn crypto_sign_ed25519_pk_to_curve25519(
        curve25519_pk: *mut [u8; ENC_PUBLICKEYBYTES],
        ed25519_pk: *const [u8; SIG_PUBLICKEYBYTES],
    ) -> c_int;
    pub fn crypto_sign_ed25519_sk_to_curve25519(
        curve25519_sk: *mut [u8; ENC_SECRETKEYBYTES],
        ed25519_sk: *const [u8; SIG_SECRETKEYBYTES],
    ) -> c_int;
}

pub struct SodiumCryptoBox {}

impl SodiumCryptoBox {
    pub fn new() -> Self {
        Self {}
    }

    fn ed25519_to_curve25519_public_key(
        &self,
        public_key: sign::PublicKey,
    ) -> VcxCoreResult<box_::PublicKey> {
        let mut to: [u8; ENC_PUBLICKEYBYTES] = [0; ENC_PUBLICKEYBYTES];
        unsafe {
            crypto_sign_ed25519_pk_to_curve25519(&mut to, &public_key.0);
        }
        box_::PublicKey::from_slice(&to).ok_or_else(|| {
            AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidInput,
                "Invalid bytes for curve25519 public key",
            )
        })
    }

    fn ed25519_to_curve25519_secret_key(
        &self,
        secret_key: sign::SecretKey,
    ) -> VcxCoreResult<box_::SecretKey> {
        let mut to: [u8; ENC_SECRETKEYBYTES] = [0; ENC_SECRETKEYBYTES];
        unsafe {
            crypto_sign_ed25519_sk_to_curve25519(&mut to, &secret_key.0);
        }
        box_::SecretKey::from_slice(&to).ok_or_else(|| {
            AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidInput,
                "Invalid bytes for curve25519 private key",
            )
        })
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

        let sk = sign::SecretKey(sk_bytes);
        let pk = sign::PublicKey(pk_bytes);

        let sk = self.ed25519_to_curve25519_secret_key(sk)?;
        let pk = self.ed25519_to_curve25519_public_key(pk)?;

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

        let sk = sign::SecretKey(sk_bytes);
        let pk = sign::PublicKey(pk_bytes);

        let sk = self.ed25519_to_curve25519_secret_key(sk)?;
        let pk = self.ed25519_to_curve25519_public_key(pk)?;

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

        let pk = sign::PublicKey(pk_bytes);
        let pk = self.ed25519_to_curve25519_public_key(pk)?;

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

        let sk = sign::SecretKey(sk_bytes);
        let pk = sign::PublicKey(pk_bytes);

        let sk = self.ed25519_to_curve25519_secret_key(sk)?;
        let pk = self.ed25519_to_curve25519_public_key(pk)?;

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
    use aries_askar::kms::KeyAlg::Ed25519;
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
        let sender_key = LocalKey::generate(Ed25519, true).unwrap();
        let sender_private_bytes = local_key_to_private_key_bytes(&sender_key).unwrap();
        let sender_public_bytes = local_key_to_public_key_bytes(&sender_key).unwrap();
        let sender_all_bytes = [sender_private_bytes, sender_public_bytes].concat();

        let recipient_key = LocalKey::generate(Ed25519, true).unwrap();
        let recipient_public_bytes = local_key_to_public_key_bytes(&recipient_key).unwrap();

        let msg = "secret";

        let crypto_box = SodiumCryptoBox::new();

        let (enc, nonce) = crypto_box
            .box_encrypt(&sender_all_bytes, &recipient_public_bytes, msg.as_bytes())
            .unwrap();

        let recipient_private_bytes = local_key_to_private_key_bytes(&recipient_key).unwrap();
        let recipient_public_bytes = local_key_to_public_key_bytes(&recipient_key).unwrap();
        let recipient_all_bytes = [recipient_private_bytes, recipient_public_bytes].concat();

        let sender_public_bytes = local_key_to_public_key_bytes(&sender_key).unwrap();

        let res = crypto_box
            .box_decrypt(&recipient_all_bytes, &sender_public_bytes, &enc, &nonce)
            .unwrap();

        assert_eq!(msg, bytes_to_string(res).unwrap());
    }

    #[test]
    fn test_sealedbox_should_encrypt_and_decrypt() {
        let sender_key = LocalKey::generate(Ed25519, false).unwrap();
        let sender_private_bytes = local_key_to_private_key_bytes(&sender_key).unwrap();
        let sender_public_bytes = local_key_to_public_key_bytes(&sender_key).unwrap();
        let sender_all_bytes = [sender_private_bytes, sender_public_bytes.clone()].concat();

        let crypto_box = SodiumCryptoBox::new();

        let msg = "secret";

        let enc = crypto_box
            .sealedbox_encrypt(&sender_public_bytes, msg.as_bytes())
            .unwrap();

        let dec = crypto_box
            .sealedbox_decrypt(&sender_all_bytes, &sender_public_bytes, &enc)
            .unwrap();

        assert_eq!(msg, bytes_to_string(dec).unwrap());
    }
}
