use indyrs::{future::Future, cheqd_ledger, IndyError, WalletHandle};

use crate::utils::{cheqd_ledger as u_cheqd_ledger, cheqd_pool, cheqd_setup};

pub const VERKEY_TYPE: &str = "verkey";

const FULLY_DID_PREFIX: &str = "did:cheqd:testnet:";
const VERKEY_ALIAS: &str = "#verkey";

pub fn did_info() -> String {
    json!({
        "ledger_type": "cheqd",
        "method_name": "testnet",
    }).to_string()
}

pub fn make_fully_did(did: &str) -> String {
    let mut fully_did: String = FULLY_DID_PREFIX.to_string();
    fully_did.push_str(did);
    fully_did
}

pub fn build_msg_create_did(
    did: &str,
    verkey: &str,
) -> Result<Vec<u8>, IndyError> {
    cheqd_ledger::cheqd::build_msg_create_did(did, verkey).wait()
}

pub fn parse_msg_create_did_resp(commit_resp: &str) -> Result<String, IndyError> {
    cheqd_ledger::cheqd::parse_msg_create_did_resp(commit_resp).wait()
}

pub fn build_msg_update_did(
    did: &str,
    verkey: &str,
    version_id: &str,
) -> Result<Vec<u8>, IndyError> {
    cheqd_ledger::cheqd::build_msg_update_did(did, verkey, version_id).wait()
}

pub fn parse_msg_update_did_resp(commit_resp: &str) -> Result<String, IndyError> {
    cheqd_ledger::cheqd::parse_msg_update_did_resp(commit_resp).wait()
}

pub fn build_query_get_did(id: &str) -> Result<String, IndyError> {
    cheqd_ledger::cheqd::build_query_get_did(id).wait()
}

pub fn parse_query_get_did_resp(query_resp: &str) -> Result<String, IndyError> {
    cheqd_ledger::cheqd::parse_query_get_did_resp(query_resp).wait()
}

pub fn sign_msg_request(wallet_handle: WalletHandle, fully_did: &str, msg: &[u8]) -> Result<Vec<u8>, IndyError> {
    cheqd_ledger::cheqd::sign_msg_write_request(wallet_handle, fully_did, msg).wait()
}

pub fn sign_and_broadcast_cheqd_msg(setup: &cheqd_setup::CheqdSetup, fully_did: &str, msg: Vec<u8>) -> Result<String, IndyError> {
    let (account_number, account_sequence) = setup.get_base_account_number_and_sequence(&setup.account_id)?;

    let signed_msg = u_cheqd_ledger::cheqd::sign_msg_request(setup.wallet_handle, &fully_did, &msg)?;
    println!("Indy Signed message:::::: {:?}", signed_msg);
    // Transaction
    let tx = u_cheqd_ledger::auth::build_tx(
        &setup.pool_alias,
        &setup.pub_key,
        &signed_msg,
        account_number,
        account_sequence,
        90000,
        2250000u64,
        &setup.denom,
        setup.get_timeout_height(),
        "memo",
    )?;

    // Sign
    let signed = u_cheqd_ledger::auth::sign_tx(setup.wallet_handle, &setup.key_alias, &tx)?;

    // Broadcast
    let br_commit = cheqd_pool::broadcast_tx_commit(&setup.pool_alias, &signed)?;
    println!(":::::: Broadcast commit::::: {:?}", br_commit);
    Ok(br_commit)
}
