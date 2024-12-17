use aries_askar::{
    crypto::alg::Chacha20Types,
    kms::{KeyAlg, LocalKey},
};
use async_trait::async_trait;
use public_key::Key;

use super::{
    askar_utils::{local_key_to_public_key, public_key_to_local_key, seed_from_opt},
    pack::Pack,
    sig_type::SigType,
    unpack::unpack,
    AskarWallet,
};
use crate::{
    errors::error::{VcxWalletError, VcxWalletResult},
    wallet::{
        base_wallet::{did_data::DidData, did_wallet::DidWallet, record_category::RecordCategory},
        structs_io::UnpackMessageOutput,
    },
};

#[async_trait]
impl DidWallet for AskarWallet {
    async fn key_count(&self) -> VcxWalletResult<usize> {
        let mut session = self.session().await?;

        Ok(session
            .fetch_all_keys(None, None, None, None, false)
            .await?
            .len())
    }

    async fn create_and_store_my_did(
        &self,
        seed: Option<&str>,
        _did_method_name: Option<&str>,
    ) -> VcxWalletResult<DidData> {
        let mut tx = self.transaction().await?;
        let (_vk, local_key) = self
            .insert_key(&mut tx, KeyAlg::Ed25519, seed_from_opt(seed).as_bytes())
            .await?;

        let verkey = local_key_to_public_key(&local_key)?;

        // construct NYM from first half of verkey as expected output from this method
        let nym = {
            let pk = verkey.key();
            if pk.len() != 32 {
                return Err(VcxWalletError::InvalidInput(format!(
                    "Invalid key length: {}",
                    pk.len()
                )));
            }
            bs58::encode(&pk[0..16]).into_string()
        };

        self.insert_did(
            &mut tx,
            &nym,
            &RecordCategory::Did.to_string(),
            &verkey,
            None,
        )
        .await?;
        tx.commit().await?;
        Ok(DidData::new(&nym, &verkey))
    }

    async fn key_for_did(&self, did: &str) -> VcxWalletResult<Key> {
        let data = self
            .find_current_did(&mut self.session().await?, did)
            .await?;

        if let Some(did_data) = data {
            Ok(did_data.verkey().to_owned())
        } else {
            Err(VcxWalletError::record_not_found_from_details(
                RecordCategory::Did,
                did,
            ))
        }
    }

    async fn replace_did_key_start(&self, did: &str, seed: Option<&str>) -> VcxWalletResult<Key> {
        let mut tx = self.transaction().await?;
        if self.find_current_did(&mut tx, did).await?.is_some() {
            let (_, local_key) = self
                .insert_key(&mut tx, KeyAlg::Ed25519, seed_from_opt(seed).as_bytes())
                .await?;

            let verkey = local_key_to_public_key(&local_key)?;
            self.insert_did(
                &mut tx,
                did,
                &RecordCategory::TmpDid.to_string(),
                &verkey,
                None,
            )
            .await?;
            tx.commit().await?;

            Ok(verkey)
        } else {
            Err(VcxWalletError::record_not_found_from_details(
                RecordCategory::Did,
                did,
            ))
        }
    }

    async fn replace_did_key_apply(&self, did: &str) -> VcxWalletResult<()> {
        let mut tx = self.transaction().await?;
        if let Some(did_value) = self.find_did(&mut tx, did, RecordCategory::TmpDid).await? {
            tx.remove(&RecordCategory::TmpDid.to_string(), did).await?;
            tx.remove_key(&did_value.verkey().base58()).await?;
            self.update_did(
                &mut tx,
                did,
                &RecordCategory::Did.to_string(),
                did_value.verkey(),
                None,
            )
            .await?;
            tx.commit().await?;
            return Ok(());
        } else {
            return Err(VcxWalletError::record_not_found_from_details(
                RecordCategory::TmpDid,
                did,
            ));
        }
    }

    async fn sign(&self, key: &Key, msg: &[u8]) -> VcxWalletResult<Vec<u8>> {
        let Some(key) = self
            .session()
            .await?
            .fetch_key(&key.base58(), false)
            .await?
        else {
            return Err(VcxWalletError::record_not_found_from_details(
                RecordCategory::Key,
                &key.base58(),
            ));
        };

        let local_key = key.load_local_key()?;
        let key_alg = SigType::try_from_key_alg(local_key.algorithm())?;
        Ok(local_key.sign_message(msg, Some(key_alg.into()))?)
    }

    async fn verify(&self, key: &Key, msg: &[u8], signature: &[u8]) -> VcxWalletResult<bool> {
        let local_key = public_key_to_local_key(key)?;

        let sig_alg = SigType::try_from_key_alg(local_key.algorithm())?;
        Ok(local_key.verify_signature(msg, signature, Some(sig_alg.into()))?)
    }

    async fn pack_message(
        &self,
        sender_vk: Option<Key>,
        recipient_keys: Vec<Key>,
        msg: &[u8],
    ) -> VcxWalletResult<Vec<u8>> {
        if recipient_keys.is_empty() {
            Err(VcxWalletError::InvalidInput(
                "recipient keys should not be empty for 'pack_message'".into(),
            ))
        } else {
            let enc_key = LocalKey::generate_with_rng(KeyAlg::Chacha20(Chacha20Types::C20P), true)?;

            let base64_data = if let Some(sender_verkey) = sender_vk {
                let mut session = self.session().await?;

                let my_key = self
                    .fetch_local_key(&mut session, &sender_verkey.base58())
                    .await?;
                enc_key.pack_authcrypt(recipient_keys, my_key)?
            } else {
                enc_key.pack_anoncrypt(recipient_keys)?
            };

            Ok(enc_key.pack_all(base64_data, msg)?)
        }
    }

    async fn unpack_message(&self, msg: &[u8]) -> VcxWalletResult<UnpackMessageOutput> {
        Ok(unpack(serde_json::from_slice(msg)?, &mut self.session().await?).await?)
    }
}
