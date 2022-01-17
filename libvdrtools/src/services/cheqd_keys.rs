//! Service to manage Cosmos keys

use cosmrs::crypto::secp256k1::EcdsaSigner;
use cosmrs::crypto::secp256k1::SigningKey as CosmosSigningKey;
use cosmrs::crypto::PublicKey as CosmosPublicKey;
use indy_api_types::errors::{IndyResult, IndyResultExt, IndyErrorKind, err_msg};
use k256::ecdsa::signature::rand_core::OsRng;
use k256::ecdsa::SigningKey;
use bip32;
use bip39;

use crate::domain::cheqd_keys::{Key, KeyInfo};

use bip39::core::str::FromStr;

const BENCH32_PREFIX: &str = "cheqd";

pub(crate) struct CheqdKeysService {}

impl CheqdKeysService {
    pub(crate) fn new() -> Self {
        Self {}
    }

    fn bytes_to_signing_key(bytes: &[u8]) -> IndyResult<SigningKey> {
        Ok(SigningKey::from_bytes(bytes).to_indy(
                IndyErrorKind::InvalidStructure,
                "Error was raised while converting bytes of key into the k256::ecdsa::SigningKey object"
            )?)
    }

    fn bytes_to_cosmos_signing_key(bytes: &[u8]) -> IndyResult<CosmosSigningKey> {
        let sig_key = Self::bytes_to_signing_key(bytes)?;
        Ok(CosmosSigningKey::from(
            Box::new(sig_key) as Box<dyn EcdsaSigner>
        ))
    }

    pub(crate) fn new_random(&self, alias: &str) -> IndyResult<Key> {
        let mnemonic = bip32::Mnemonic::random(&mut OsRng,
                                               bip32::Language::English);
        let key = self.new_from_mnemonic(
            alias,
            mnemonic.phrase(),
            "")?;

        Ok(key)
    }

    pub(crate) fn new_from_mnemonic(
        &self,
        alias: &str,
        mnemonic: &str,
        passphrase: &str,
    ) -> IndyResult<Key> {

        let mnemonic = bip39::Mnemonic::from_str(mnemonic)?;
        let seed = mnemonic.to_seed_normalized(passphrase);
        // m / purpose' / coin_type' / account' / change / address_index
        let child_path = "m/44'/118'/0'/0/0"
            .parse::<bip32::DerivationPath>()?;
        let xprv = bip32::XPrv::derive_from_path(
            seed,
            &child_path).unwrap();
        let key = Key::new(alias.to_string(), xprv.to_bytes(), Some(mnemonic.to_string()));
        Ok(key)
    }


    pub(crate) fn get_info(&self, key: &Key) -> IndyResult<KeyInfo> {
        let sig_key = Self::bytes_to_cosmos_signing_key(&key.priv_key)?;
        let pub_key = sig_key.public_key();
        let pub_key_str = pub_key.to_json();

        let account_id = pub_key.account_id(BENCH32_PREFIX)?;

        let key_info = KeyInfo::new(
            key.alias.to_owned(),
            account_id.to_string(),
            pub_key_str,
            key.mnemonic.clone(),
        );

        Ok(key_info)
    }

    pub(crate) async fn sign(&self, key: &Key, bytes_to_sign: &[u8]) -> IndyResult<Vec<u8>> {
        let sig_key = Self::bytes_to_cosmos_signing_key(&key.priv_key)?;
        let signature = sig_key.sign(&bytes_to_sign)?.as_ref().to_vec();
        Ok(signature)
    }

    pub(crate) fn get_account_id_from_public_key(&self, public_key: &str) -> IndyResult<String> {
        let public_key = CosmosPublicKey::from_json(public_key)
            .map_err(|err| err_msg(
                IndyErrorKind::InvalidStructure,
                format!("Unable to parse cheqd public key from JSON. Err: {:?}", err)
            ))?;
        let account_id = public_key.account_id(BENCH32_PREFIX)?;
        Ok(account_id.to_string())
    }
}

#[cfg(test)]
mod test {
    use cosmrs::crypto::secp256k1::{
        EcdsaSigner, SigningKey as CosmosSigningKey,
    };
    use k256::ecdsa::signature::Signature;
    use k256::ecdsa::signature::Signer;
    use k256::elliptic_curve::rand_core::OsRng;

    use super::*;

    #[async_std::test]
    async fn test_add_random() {
        let cheqd_keys_service = CheqdKeysService::new();

        let key = cheqd_keys_service.new_random("alice").unwrap();

        assert_eq!(key.alias, "alice")
    }

    #[async_std::test]
    async fn test_add_from_mnemonic() {
        let cheqd_keys_service = CheqdKeysService::new();
        let mnemonic = "sell table balcony salad acquire love hover resist give baby liquid process lecture awkward injury crucial rack stem prepare bar unable among december ankle";

        let alice = cheqd_keys_service
            .new_from_mnemonic("alice", mnemonic, "")
            .unwrap();
        let alice_info = cheqd_keys_service.get_info(&alice).unwrap();

        let bob = cheqd_keys_service
            .new_from_mnemonic("bob", mnemonic, "")
            .unwrap();
        let bob_info = cheqd_keys_service.get_info(&bob).unwrap();

        assert_eq!(alice_info.pub_key, bob_info.pub_key)
    }

    #[test]
    fn test_restore_from_mnemonic() {
        let cheqd_keys_service = CheqdKeysService::new();
        let mnemonic = "alarm elite thunder resist edit unhappy sand decline artist search surprise wool skirt glass pool erode easily sort spatial pact nature dose erode gospel";
        let alias = "test";
        let expected_accunt_id = "cheqd1hdgutl66skk9dudzkcfm2cafncvgzw8y5azn4g";

        let key = cheqd_keys_service.new_from_mnemonic(alias,mnemonic, "").unwrap();
        assert_eq!(cheqd_keys_service.get_info(&key).unwrap().account_id.as_str(), expected_accunt_id)
    }

    #[test]
    fn test_private_key_import_export() {
        let key = k256::ecdsa::SigningKey::random(&mut OsRng);
        let bytes = key.to_bytes().to_vec();
        let imported = k256::ecdsa::SigningKey::from_bytes(&bytes).unwrap();

        let msg = vec![251u8, 252, 253, 254];

        let s1: k256::ecdsa::Signature = key.sign(&msg);
        let s2: k256::ecdsa::Signature = imported.sign(&msg);

        assert_eq!(s1, s2);
    }

    #[test]
    fn test_private_key_compatibility() {
        let msg = vec![251u8, 252, 253, 254];

        let key = k256::ecdsa::SigningKey::random(&mut OsRng);
        let s1: k256::ecdsa::Signature = key.sign(&msg);
        let s1 = s1.as_ref().to_vec();

        let cosmos_key = CosmosSigningKey::from(Box::new(key) as Box<dyn EcdsaSigner>);
        let s2 = cosmos_key.sign(&msg).unwrap().as_bytes().to_vec();

        assert_eq!(s1, s2);
    }

    #[test]
    fn test_pub_key_compatibility() {
        let key = k256::ecdsa::SigningKey::random(&mut OsRng);
        let pub_key = key.verifying_key().to_bytes().to_vec();

        let cosmos_key = CosmosSigningKey::from(Box::new(key) as Box<dyn EcdsaSigner>);
        let cosmos_pub_key = cosmos_key.public_key().to_bytes();

        assert_eq!(pub_key, cosmos_pub_key);
    }
}
