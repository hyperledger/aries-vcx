//! The adapters here currently use an [`std::sync::Arc`] because there still are some places
//! (looking at you, libvcx) where Arcs are still used and removing them requires quite the effort
//! as the places left are the ones conditionally accessed through some RwLock and the Arc allows
//! bypassing lifetime limits imposed by the RwLockGuard.
//!
//! Ideally libvcx will become obsolete and then the pointers here can be changed to a more
//! efficient [`Box`].

use std::{collections::HashMap, sync::Arc};

use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds,
    errors::error::VcxCoreResult,
    ledger::{
        base_ledger::{
            AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite,
            TaaConfigurator, TxnAuthrAgrmtOptions,
        },
        indy_vdr_ledger::UpdateRole,
    },
    utils::async_fn_iterator::AsyncFnIterator,
    wallet::{base_wallet::BaseWallet, structs_io::UnpackMessageOutput},
};
use async_trait::async_trait;

#[derive(Debug)]
pub struct Adapter<T: ?Sized>(Arc<T>);

#[async_trait]
impl<T> BaseAnonCreds for Adapter<T>
where
    T: ?Sized + std::fmt::Debug + Send + Sync + BaseAnonCreds,
{
    async fn verifier_verify_proof(
        &self,
        proof_request_json: &str,
        proof_json: &str,
        schemas_json: &str,
        credential_defs_json: &str,
        rev_reg_defs_json: &str,
        rev_regs_json: &str,
    ) -> VcxCoreResult<bool> {
        self.0
            .verifier_verify_proof(
                proof_request_json,
                proof_json,
                schemas_json,
                credential_defs_json,
                rev_reg_defs_json,
                rev_regs_json,
            )
            .await
    }

    async fn issuer_create_and_store_revoc_reg(
        &self,
        wallet: &impl BaseWallet,
        issuer_did: &str,
        cred_def_id: &str,
        tails_dir: &str,
        max_creds: u32,
        tag: &str,
    ) -> VcxCoreResult<(String, String, String)> {
        self.0
            .issuer_create_and_store_revoc_reg(
                wallet,
                issuer_did,
                cred_def_id,
                tails_dir,
                max_creds,
                tag,
            )
            .await
    }

    async fn issuer_create_and_store_credential_def(
        &self,
        wallet: &impl BaseWallet,
        issuer_did: &str,
        schema_json: &str,
        tag: &str,
        signature_type: Option<&str>,
        config_json: &str,
    ) -> VcxCoreResult<(String, String)> {
        self.0
            .issuer_create_and_store_credential_def(
                wallet,
                issuer_did,
                schema_json,
                tag,
                signature_type,
                config_json,
            )
            .await
    }

    async fn issuer_create_credential_offer(
        &self,
        wallet: &impl BaseWallet,
        cred_def_id: &str,
    ) -> VcxCoreResult<String> {
        self.0
            .issuer_create_credential_offer(wallet, cred_def_id)
            .await
    }

    async fn issuer_create_credential(
        &self,
        wallet: &impl BaseWallet,
        cred_offer_json: &str,
        cred_req_json: &str,
        cred_values_json: &str,
        rev_reg_id: Option<String>,
        tails_dir: Option<String>,
    ) -> VcxCoreResult<(String, Option<String>, Option<String>)> {
        self.0
            .issuer_create_credential(
                wallet,
                cred_offer_json,
                cred_req_json,
                cred_values_json,
                rev_reg_id,
                tails_dir,
            )
            .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn prover_create_proof(
        &self,
        wallet: &impl BaseWallet,
        proof_req_json: &str,
        requested_credentials_json: &str,
        master_secret_id: &str,
        schemas_json: &str,
        credential_defs_json: &str,
        revoc_states_json: Option<&str>,
    ) -> VcxCoreResult<String> {
        self.0
            .prover_create_proof(
                wallet,
                proof_req_json,
                requested_credentials_json,
                master_secret_id,
                schemas_json,
                credential_defs_json,
                revoc_states_json,
            )
            .await
    }

    async fn prover_get_credential(
        &self,
        wallet: &impl BaseWallet,
        cred_id: &str,
    ) -> VcxCoreResult<String> {
        self.0.prover_get_credential(wallet, cred_id).await
    }

    async fn prover_get_credentials(
        &self,
        wallet: &impl BaseWallet,
        filter_json: Option<&str>,
    ) -> VcxCoreResult<String> {
        self.0.prover_get_credentials(wallet, filter_json).await
    }

    async fn prover_get_credentials_for_proof_req(
        &self,
        wallet: &impl BaseWallet,
        proof_request_json: &str,
    ) -> VcxCoreResult<String> {
        self.0
            .prover_get_credentials_for_proof_req(wallet, proof_request_json)
            .await
    }

    async fn prover_create_credential_req(
        &self,
        wallet: &impl BaseWallet,
        prover_did: &str,
        cred_offer_json: &str,
        cred_def_json: &str,
        master_secret_id: &str,
    ) -> VcxCoreResult<(String, String)> {
        self.0
            .prover_create_credential_req(
                wallet,
                prover_did,
                cred_offer_json,
                cred_def_json,
                master_secret_id,
            )
            .await
    }

    async fn create_revocation_state(
        &self,
        tails_dir: &str,
        rev_reg_def_json: &str,
        rev_reg_delta_json: &str,
        timestamp: u64,
        cred_rev_id: &str,
    ) -> VcxCoreResult<String> {
        self.0
            .create_revocation_state(
                tails_dir,
                rev_reg_def_json,
                rev_reg_delta_json,
                timestamp,
                cred_rev_id,
            )
            .await
    }

    async fn prover_store_credential(
        &self,
        wallet: &impl BaseWallet,
        cred_id: Option<&str>,
        cred_req_metadata_json: &str,
        cred_json: &str,
        cred_def_json: &str,
        rev_reg_def_json: Option<&str>,
    ) -> VcxCoreResult<String> {
        self.0
            .prover_store_credential(
                wallet,
                cred_id,
                cred_req_metadata_json,
                cred_json,
                cred_def_json,
                rev_reg_def_json,
            )
            .await
    }

    async fn prover_delete_credential(
        &self,
        wallet: &impl BaseWallet,
        cred_id: &str,
    ) -> VcxCoreResult<()> {
        self.0.prover_delete_credential(wallet, cred_id).await
    }

    async fn prover_create_link_secret(
        &self,
        wallet: &impl BaseWallet,
        link_secret_id: &str,
    ) -> VcxCoreResult<String> {
        self.0
            .prover_create_link_secret(wallet, link_secret_id)
            .await
    }

    async fn issuer_create_schema(
        &self,
        issuer_did: &str,
        name: &str,
        version: &str,
        attrs: &str,
    ) -> VcxCoreResult<(String, String)> {
        self.0
            .issuer_create_schema(issuer_did, name, version, attrs)
            .await
    }

    // TODO - FUTURE - think about moving this to somewhere else, as it aggregates other calls (not
    // PURE Anoncreds)
    async fn revoke_credential_local(
        &self,
        wallet: &impl BaseWallet,
        tails_dir: &str,
        rev_reg_id: &str,
        cred_rev_id: &str,
    ) -> VcxCoreResult<()> {
        self.0
            .revoke_credential_local(wallet, tails_dir, rev_reg_id, cred_rev_id)
            .await
    }

    async fn get_rev_reg_delta(
        &self,
        wallet: &impl BaseWallet,
        rev_reg_id: &str,
    ) -> VcxCoreResult<Option<String>> {
        self.0.get_rev_reg_delta(wallet, rev_reg_id).await
    }

    async fn clear_rev_reg_delta(
        &self,
        wallet: &impl BaseWallet,
        rev_reg_id: &str,
    ) -> VcxCoreResult<()> {
        self.0.clear_rev_reg_delta(wallet, rev_reg_id).await
    }

    async fn generate_nonce(&self) -> VcxCoreResult<String> {
        self.0.generate_nonce().await
    }
}

#[async_trait]
impl<T> BaseWallet for Adapter<T>
where
    T: ?Sized + BaseWallet + std::fmt::Debug + Send + Sync,
{
    #[cfg(feature = "vdrtools_wallet")]
    fn get_wallet_handle(&self) -> aries_vcx_core::WalletHandle {
        self.0.get_wallet_handle()
    }

    // ----- DIDs
    async fn create_and_store_my_did(
        &self,
        seed: Option<&str>,
        kdf_method_name: Option<&str>,
    ) -> VcxCoreResult<(String, String)> {
        self.0.create_and_store_my_did(seed, kdf_method_name).await
    }

    async fn key_for_local_did(&self, did: &str) -> VcxCoreResult<String> {
        self.0.key_for_local_did(did).await
    }

    // returns new temp_verkey and remembers it internally
    async fn replace_did_keys_start(&self, target_did: &str) -> VcxCoreResult<String> {
        self.0.replace_did_keys_start(target_did).await
    }

    // replaces the `target_did`'s current verkey with the one last generated by
    // `replace_did_keys_start`
    async fn replace_did_keys_apply(&self, target_did: &str) -> VcxCoreResult<()> {
        self.0.replace_did_keys_apply(target_did).await
    }

    // ---- records

    async fn add_wallet_record(
        &self,
        xtype: &str,
        id: &str,
        value: &str,
        tags: Option<HashMap<String, String>>,
    ) -> VcxCoreResult<()> {
        self.0.add_wallet_record(xtype, id, value, tags).await
    }

    async fn get_wallet_record(
        &self,
        xtype: &str,
        id: &str,
        options: &str,
    ) -> VcxCoreResult<String> {
        self.0.get_wallet_record(xtype, id, options).await
    }

    async fn get_wallet_record_value(&self, xtype: &str, id: &str) -> VcxCoreResult<String> {
        self.0.get_wallet_record_value(xtype, id).await
    }

    async fn delete_wallet_record(&self, xtype: &str, id: &str) -> VcxCoreResult<()> {
        self.0.delete_wallet_record(xtype, id).await
    }

    async fn update_wallet_record_value(
        &self,
        xtype: &str,
        id: &str,
        value: &str,
    ) -> VcxCoreResult<()> {
        self.0.update_wallet_record_value(xtype, id, value).await
    }

    async fn add_wallet_record_tags(
        &self,
        xtype: &str,
        id: &str,
        tags: HashMap<String, String>,
    ) -> VcxCoreResult<()> {
        self.0.add_wallet_record_tags(xtype, id, tags).await
    }

    async fn update_wallet_record_tags(
        &self,
        xtype: &str,
        id: &str,
        tags: HashMap<String, String>,
    ) -> VcxCoreResult<()> {
        self.0.update_wallet_record_tags(xtype, id, tags).await
    }

    async fn delete_wallet_record_tags(
        &self,
        xtype: &str,
        id: &str,
        tag_names: &str,
    ) -> VcxCoreResult<()> {
        self.0.delete_wallet_record_tags(xtype, id, tag_names).await
    }

    async fn iterate_wallet_records(
        &self,
        xtype: &str,
        query: &str,
        options: &str,
    ) -> VcxCoreResult<Box<dyn AsyncFnIterator<Item = VcxCoreResult<String>>>> {
        self.0.iterate_wallet_records(xtype, query, options).await
    }

    // ---- crypto

    async fn sign(&self, my_vk: &str, msg: &[u8]) -> VcxCoreResult<Vec<u8>> {
        self.0.sign(my_vk, msg).await
    }

    async fn verify(&self, vk: &str, msg: &[u8], signature: &[u8]) -> VcxCoreResult<bool> {
        self.0.verify(vk, msg, signature).await
    }

    async fn pack_message(
        &self,
        sender_vk: Option<&str>,
        receiver_keys: &str,
        msg: &[u8],
    ) -> VcxCoreResult<Vec<u8>> {
        self.0.pack_message(sender_vk, receiver_keys, msg).await
    }

    async fn unpack_message(&self, msg: &[u8]) -> VcxCoreResult<UnpackMessageOutput> {
        self.0.unpack_message(msg).await
    }
}

#[async_trait]
impl<T> IndyLedgerRead for Adapter<T>
where
    T: ?Sized + std::fmt::Debug + Send + Sync + IndyLedgerRead,
{
    async fn get_attr(&self, target_did: &str, attr_name: &str) -> VcxCoreResult<String> {
        self.0.get_attr(target_did, attr_name).await
    }
    async fn get_nym(&self, did: &str) -> VcxCoreResult<String> {
        self.0.get_nym(did).await
    }
    async fn get_txn_author_agreement(&self) -> VcxCoreResult<Option<String>> {
        self.0.get_txn_author_agreement().await
    }
    async fn get_ledger_txn(
        &self,
        seq_no: i32,
        submitter_did: Option<&str>,
    ) -> VcxCoreResult<String> {
        self.0.get_ledger_txn(seq_no, submitter_did).await
    }
}

#[async_trait]
impl<T> IndyLedgerWrite for Adapter<T>
where
    T: ?Sized + std::fmt::Debug + Send + Sync + IndyLedgerWrite,
{
    async fn publish_nym(
        &self,
        wallet: &impl BaseWallet,
        submitter_did: &str,
        target_did: &str,
        verkey: Option<&str>,
        data: Option<&str>,
        role: Option<&str>,
    ) -> VcxCoreResult<String> {
        self.0
            .publish_nym(wallet, submitter_did, target_did, verkey, data, role)
            .await
    }
    async fn set_endorser(
        &self,
        wallet: &impl BaseWallet,
        submitter_did: &str,
        request: &str,
        endorser: &str,
    ) -> VcxCoreResult<String> {
        self.0
            .set_endorser(wallet, submitter_did, request, endorser)
            .await
    }
    async fn endorse_transaction(
        &self,
        wallet: &impl BaseWallet,
        endorser_did: &str,
        request_json: &str,
    ) -> VcxCoreResult<()> {
        self.0
            .endorse_transaction(wallet, endorser_did, request_json)
            .await
    }
    async fn add_attr(
        &self,
        wallet: &impl BaseWallet,
        target_did: &str,
        attrib_json: &str,
    ) -> VcxCoreResult<String> {
        self.0.add_attr(wallet, target_did, attrib_json).await
    }
    async fn write_did(
        &self,
        wallet: &impl BaseWallet,
        submitter_did: &str,
        target_did: &str,
        target_vk: &str,
        role: Option<UpdateRole>,
        alias: Option<String>,
    ) -> VcxCoreResult<String> {
        self.0
            .write_did(wallet, submitter_did, target_did, target_vk, role, alias)
            .await
    }
}

#[async_trait]
impl<T> AnoncredsLedgerRead for Adapter<T>
where
    T: ?Sized + std::fmt::Debug + Send + Sync + AnoncredsLedgerRead,
{
    async fn get_schema(
        &self,
        schema_id: &str,
        submitter_did: Option<&str>,
    ) -> VcxCoreResult<String> {
        self.0.get_schema(schema_id, submitter_did).await
    }
    async fn get_cred_def(
        &self,
        cred_def_id: &str,
        submitter_did: Option<&str>,
    ) -> VcxCoreResult<String> {
        self.0.get_cred_def(cred_def_id, submitter_did).await
    }
    async fn get_rev_reg_def_json(&self, rev_reg_id: &str) -> VcxCoreResult<String> {
        self.0.get_rev_reg_def_json(rev_reg_id).await
    }
    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxCoreResult<(String, String, u64)> {
        self.0.get_rev_reg_delta_json(rev_reg_id, from, to).await
    }
    async fn get_rev_reg(
        &self,
        rev_reg_id: &str,
        timestamp: u64,
    ) -> VcxCoreResult<(String, String, u64)> {
        self.0.get_rev_reg(rev_reg_id, timestamp).await
    }
}

#[async_trait]
impl<T> AnoncredsLedgerWrite for Adapter<T>
where
    T: ?Sized + std::fmt::Debug + Send + Sync + AnoncredsLedgerWrite,
{
    async fn publish_schema(
        &self,
        wallet: &impl BaseWallet,
        schema_json: &str,
        submitter_did: &str,
        endorser_did: Option<String>,
    ) -> VcxCoreResult<()> {
        self.0
            .publish_schema(wallet, schema_json, submitter_did, endorser_did)
            .await
    }
    async fn publish_cred_def(
        &self,
        wallet: &impl BaseWallet,
        cred_def_json: &str,
        submitter_did: &str,
    ) -> VcxCoreResult<()> {
        self.0
            .publish_cred_def(wallet, cred_def_json, submitter_did)
            .await
    }
    async fn publish_rev_reg_def(
        &self,
        wallet: &impl BaseWallet,
        rev_reg_def: &str,
        submitter_did: &str,
    ) -> VcxCoreResult<()> {
        self.0
            .publish_rev_reg_def(wallet, rev_reg_def, submitter_did)
            .await
    }
    async fn publish_rev_reg_delta(
        &self,
        wallet: &impl BaseWallet,
        rev_reg_id: &str,
        rev_reg_entry_json: &str,
        submitter_did: &str,
    ) -> VcxCoreResult<()> {
        self.0
            .publish_rev_reg_delta(wallet, rev_reg_id, rev_reg_entry_json, submitter_did)
            .await
    }
}

impl<T> TaaConfigurator for Adapter<T>
where
    T: ?Sized + std::fmt::Debug + Send + Sync + TaaConfigurator,
{
    fn set_txn_author_agreement_options(
        &self,
        taa_options: TxnAuthrAgrmtOptions,
    ) -> VcxCoreResult<()> {
        self.0.set_txn_author_agreement_options(taa_options)
    }
    fn get_txn_author_agreement_options(&self) -> VcxCoreResult<Option<TxnAuthrAgrmtOptions>> {
        self.0.get_txn_author_agreement_options()
    }
}
