pub mod did_doc;
pub mod service;
pub mod types;
pub mod utils;
pub mod verification_method;

/// Module of commonly used DID-related JSON-LD contexts
pub mod contexts {
    pub const W3C_DID_V1: &str = "https://www.w3.org/ns/did/v1";
    pub const W3C_SUITE_ED25519_2020: &str = "https://w3id.org/security/suites/ed25519-2020/v1";
    pub const W3C_SUITE_ED25519_2018: &str = "https://w3id.org/security/suites/ed25519-2018/v1";
    pub const W3C_SUITE_JWS_2020: &str = "https://w3id.org/security/suites/jws-2020/v1";
    pub const W3C_SUITE_SECP256K1_2019: &str = "https://w3id.org/security/suites/secp256k1-2019/v1";
    pub const W3C_BBS_V1: &str = "https://w3id.org/security/bbs/v1";
    pub const W3C_PGP_V1: &str = "https://w3id.org/pgp/v1";
    pub const W3C_SUITE_X25519_2019: &str = "https://w3id.org/security/suites/x25519-2019/v1";
    pub const W3C_SUITE_X25519_2020: &str = "https://w3id.org/security/suites/x25519-2020/v1";
    pub const W3C_SUITE_SECP259K1_RECOVERY_2020: &str =
        "https://w3id.org/security/suites/secp256k1recovery-2020/v2";
    pub const W3C_MULTIKEY_V1: &str = "https://w3id.org/security/multikey/v1";
}
