use async_trait::async_trait;
use public_key::{Key, KeyType};
use vdrtools::{DidMethod, DidValue, KeyInfo, Locator, MyDidInfo};

use crate::{
    errors::error::VcxWalletResult,
    wallet::{
        base_wallet::{did_data::DidData, did_wallet::DidWallet, record_category::RecordCategory},
        indy::IndySdkWallet,
        structs_io::UnpackMessageOutput,
    },
};

#[async_trait]
impl DidWallet for IndySdkWallet {
    async fn key_count(&self) -> VcxWalletResult<usize> {
        Ok(self.search(RecordCategory::Did, None).await?.len())
    }

    async fn create_and_store_my_did(
        &self,
        seed: Option<&str>,
        did_method_name: Option<&str>,
    ) -> VcxWalletResult<DidData> {
        let (did, vk) = Locator::instance()
            .did_controller
            .create_and_store_my_did(
                self.wallet_handle,
                MyDidInfo {
                    method_name: did_method_name.map(|m| DidMethod(m.into())),
                    seed: seed.map(Into::into),
                    ..MyDidInfo::default()
                },
            )
            .await?;

        let verkey = Key::from_base58(&vk, KeyType::Ed25519)?;
        Ok(DidData::new(&did, &verkey))
    }

    async fn key_for_did(&self, did: &str) -> VcxWalletResult<Key> {
        let res = Locator::instance()
            .did_controller
            .key_for_local_did(self.wallet_handle, DidValue(did.into()))
            .await?;

        Ok(Key::from_base58(&res, KeyType::Ed25519)?)
    }

    async fn replace_did_key_start(&self, did: &str, seed: Option<&str>) -> VcxWalletResult<Key> {
        let key_info = KeyInfo {
            seed: seed.map(Into::into),
            ..Default::default()
        };

        let key_string = Locator::instance()
            .did_controller
            .replace_keys_start(self.wallet_handle, key_info, DidValue(did.into()))
            .await?;

        Ok(Key::from_base58(&key_string, KeyType::Ed25519)?)
    }

    async fn replace_did_key_apply(&self, did: &str) -> VcxWalletResult<()> {
        Ok(Locator::instance()
            .did_controller
            .replace_keys_apply(self.wallet_handle, DidValue(did.into()))
            .await?)
    }

    async fn sign(&self, key: &Key, msg: &[u8]) -> VcxWalletResult<Vec<u8>> {
        Locator::instance()
            .crypto_controller
            .crypto_sign(self.wallet_handle, &key.base58(), msg)
            .await
            .map_err(From::from)
    }

    async fn verify(&self, key: &Key, msg: &[u8], signature: &[u8]) -> VcxWalletResult<bool> {
        Locator::instance()
            .crypto_controller
            .crypto_verify(&key.base58(), msg, signature)
            .await
            .map_err(From::from)
    }

    async fn pack_message(
        &self,
        sender_vk: Option<Key>,
        receiver_keys: Vec<Key>,
        msg: &[u8],
    ) -> VcxWalletResult<Vec<u8>> {
        let receiver_keys_str = receiver_keys.into_iter().map(|key| key.base58()).collect();

        Ok(Locator::instance()
            .crypto_controller
            .pack_msg(
                msg.into(),
                receiver_keys_str,
                sender_vk.map(|key| key.base58()),
                self.wallet_handle,
            )
            .await?)
    }

    async fn unpack_message(&self, msg: &[u8]) -> VcxWalletResult<UnpackMessageOutput> {
        let unpacked_bytes = Locator::instance()
            .crypto_controller
            .unpack_msg(serde_json::from_slice(msg)?, self.wallet_handle)
            .await?;

        Ok(serde_json::from_slice(&unpacked_bytes[..])?)
    }
}
