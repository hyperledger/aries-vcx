pub const VERKEY_TYPE: &str = "Ed25519VerificationKey2020";
const VERKEY_ALIAS: &str = "#verkey";

pub fn make_verification_id(did: &str) -> String {
    let mut fully_v_id = did.to_string();
    fully_v_id.push_str(VERKEY_ALIAS);
    fully_v_id
}

pub fn make_base58_btc(verkey: &str) -> String {
    format!("z{}",verkey.to_string())
}
