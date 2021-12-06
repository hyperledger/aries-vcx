use indyrs::vdr;
use indyrs::{future::Future, IndyError, WalletHandle};

pub use indyrs::vdr::{
    VDR, VDRBuilder
};

use crate::utils::test;

pub fn vdr_builder_create() -> Result<VDRBuilder, IndyError> {
    vdr::vdr_builder_create()
}

pub fn vdr_builder_register_indy_ledger(vdr: &mut VDRBuilder, namespace_list: &str, genesis_txn_data: &str, taa_config: Option<&str>) -> Result<(), IndyError> {
    vdr::vdr_builder_register_indy_ledger(vdr, namespace_list, genesis_txn_data, taa_config).wait()
}

#[cfg(feature = "cheqd")]
pub fn vdr_builder_register_cheqd_ledger(vdr: &mut VDRBuilder, namespace_list: &str, chain_id: &str, node_addrs_list: &str) -> Result<(), IndyError> {
    vdr::vdr_builder_register_cheqd_ledger(vdr, namespace_list, chain_id, node_addrs_list).wait()
}

pub fn vdr_builder_finalize(vdr: VDRBuilder) -> Result<VDR, IndyError> {
    vdr::vdr_builder_finalize(vdr)
}

pub fn ping(vdr: &VDR, namespace_list: &str) -> Result<String, IndyError> {
    vdr::ping(vdr, namespace_list).wait()
}

pub fn cleanup(vdr: VDR) -> Result<(), IndyError> {
    vdr::cleanup(vdr).wait()
}

pub fn resolve_did(vdr: &VDR, fqdid: &str) -> Result<String, IndyError> {
    vdr::resolve_did(vdr, fqdid).wait()
}

pub fn resolve_schema(vdr: &VDR, fqschema: &str) -> Result<String, IndyError> {
    vdr::resolve_schema(vdr, fqschema).wait()
}

pub fn resolve_cred_def(vdr: &VDR, fqcreddef: &str) -> Result<String, IndyError> {
    vdr::resolve_cred_def(vdr, fqcreddef).wait()
}

pub fn prepare_did(vdr: &VDR,
                   txn_specific_params: &str,
                   submitter_did: &str,
                   endorser: Option<&str>) -> Result<(String, Vec<u8>, String, Vec<u8>, Option<String>), IndyError> {
    vdr::prepare_did(vdr, txn_specific_params, submitter_did, endorser).wait()
}

pub fn prepare_schema(vdr: &VDR,
                      txn_specific_params: &str,
                      submitter_did: &str,
                      endorser: Option<&str>) -> Result<(String, Vec<u8>, String, Vec<u8>, Option<String>), IndyError> {
    vdr::prepare_schema(vdr, txn_specific_params, submitter_did, endorser).wait()
}

pub fn prepare_cred_def(vdr: &VDR,
                        txn_specific_params: &str,
                        submitter_did: &str,
                        endorser: Option<&str>) -> Result<(String, Vec<u8>, String, Vec<u8>, Option<String>), IndyError> {
    vdr::prepare_cred_def(vdr, txn_specific_params, submitter_did, endorser).wait()
}

pub fn submit_txn(
    vdr: &VDR,
    namespace: &str,
    txn_bytes: &[u8],
    signature_spec: &str,
    signature: &[u8],
    endorsement: Option<&str>,
) -> Result<String, IndyError> {
    vdr::submit_txn(vdr, namespace, txn_bytes, signature_spec, signature, endorsement).wait()
}

pub fn indy_endorse(
    wallet_handle: WalletHandle,
    endorsement_data: &str,
    signature_spec: &str,
    txn_bytes_to_sign: &[u8],
) -> Result<String, IndyError> {
    vdr::indy_endorse(wallet_handle, endorsement_data, signature_spec, txn_bytes_to_sign).wait()
}

#[cfg(feature = "cheqd")]
pub fn cheqd_endorse(
    wallet_handle: WalletHandle,
    endorsement_data: &str,
    signature_spec: &str,
    txn_bytes_to_sign: &[u8],
    signature: &[u8],
) -> Result<String, IndyError> {
    vdr::cheqd_endorse(wallet_handle, endorsement_data, signature_spec, txn_bytes_to_sign, signature).wait()
}

#[cfg(feature = "cheqd")]
pub fn prepare_cheqd_endorsement_data(
    vdr: &VDR,
    wallet_handle: WalletHandle,
    key_alias: &str,
    did: &str,
    tx_bytes: &[u8],
    txn_signature: &[u8],
    gas_price: u64,
    memo: &str,
) -> Result<String, IndyError> {
    vdr::prepare_cheqd_endorsement_data(vdr, wallet_handle, key_alias, did, tx_bytes, txn_signature, gas_price, memo).wait()
}

pub fn submit_raw_txn(
    vdr: &VDR,
    namespace: &str,
    txn_bytes: &[u8],
) -> Result<String, IndyError> {
    vdr::submit_raw_txn(vdr, namespace, txn_bytes).wait()
}

pub fn submit_query(
    vdr: &VDR,
    namespace: &str,
    query: &str,
) -> Result<String, IndyError> {
    vdr::submit_query(vdr, namespace, query).wait()
}

pub fn local_genesis_txn() -> String {
    test::gen_txns().join("\n")
}
