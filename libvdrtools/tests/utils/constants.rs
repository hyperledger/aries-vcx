pub const SEQ_NO: i32 = 1;
pub const PROTOCOL_VERSION: usize = 2;
pub const TYPE: &'static str = "default";
pub const INMEM_TYPE: &'static str = "inmem";
pub const SIGNATURE_TYPE: &'static str = "CL";
pub const TRUSTEE_SEED: &'static str = "000000000000000000000000Trustee1";
pub const TRUSTEE_SEED_2: &'static str = "000000000000000000000000Trustee2";
pub const STEWARD_SEED: &'static str = "000000000000000000000000Steward1";
pub const MY1_SEED: &'static str = "00000000000000000000000000000My1";
pub const MY2_SEED: &'static str = "00000000000000000000000000000My2";
pub const MY3_SEED: &'static str = "00000000000000000000000000000My3";
pub const MY4_SEED: &'static str = "00000000000000000000000000000My4";
pub const MY5_SEED: &'static str = "00000000000000000000000000000My5";
pub const ISSUER_DID: &'static str = "NcYxiDXkpYi6ov5FcYDi1e";
pub const ISSUER_DID_SUB: &'static str = "NcYxiDXkpYi6ov5FcYDi1i";
pub const ISSUER_DID_V1: &'static str = "did:indy:sovrin:builder:NcYxiDXkpYi6ov5FcYDi1e";
pub const ISSUER_DID_2: &'static str = "CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW";
pub const DID: &'static str = "CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW";
pub const DID_V1: &'static str =
    "did:indy:sovrin:builder:CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW";
pub const DID_MY1: &'static str = "VsKV7grR1BUE29mG2Fm2kX";
pub const DID_MY1_V1: &'static str = "did:indy:sovrin:builder:VsKV7grR1BUE29mG2Fm2kX";
pub const DID_MY2: &'static str = "2PRyVHmkXQnQzJQKxHxnXC";
pub const DID_TRUSTEE: &'static str = "V4SGRU86Z58d6TV7PBUe6f";
pub const INVALID_BASE58_DID: &'static str = "invalid_base58string";
pub const IDENTIFIER: &'static str = "Th7MpTaRZVRYnPiabds81Y";
pub const IDENTIFIER_V1: &'static str = "did:indy:sovrin:builder:Th7MpTaRZVRYnPiabds81Y";
pub const INVALID_IDENTIFIER: &'static str = "invalid_base58_identifier";
pub const DEST: &'static str = "FYmoFw55GeQH7SRFa37dkx1d2dZ3zUF8ckg7wmL7ofN4";
pub const DEST_V1: &'static str =
    "did:indy:sovrin:builder:FYmoFw55GeQH7SRFa37dkx1d2dZ3zUF8ckg7wmL7ofN4";
pub const GVT_SCHEMA_NAME: &'static str = "gvt";
pub const GVT_SUB_SCHEMA_NAME: &'static str = "gvtsub";
pub const XYZ_SCHEMA_NAME: &'static str = "xyz";
pub const SCHEMA_VERSION: &'static str = "1.0";
pub const SCHEMA_SUB_VERSION: &'static str = "2.2";
pub const GVT_SCHEMA_ATTRIBUTES: &'static str = r#"["name", "age", "sex", "height"]"#;
pub const GVT_SUB_SCHEMA_ATTRIBUTES: &'static str = r#"["sex", "height_sub"]"#;
pub const XYZ_SCHEMA_ATTRIBUTES: &'static str = r#"["status", "period"]"#;
pub const SCHEMA_DATA: &'static str =
    r#"{"id":"1", "name":"gvt","version":"1.0","attrNames":["name"],"ver":"1.0"}"#;
pub const ENDPOINT: &'static str = "127.0.0.1:9700";
pub const VERKEY: &'static str = "CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW";
pub const VERKEY_MY1: &'static str = "GjZWsBLgZCR18aL468JAT7w9CZRiBnpxUPPgyQxh4voa";
pub const INVALID_VERKEY_LENGTH: &'static str = "invalidVerkeyLength";
pub const INVALID_BASE58_VERKEY: &'static str = "CnEDk___MnmiHXEV1WFgbV___eYnPqs___TdcZaNhFVW";
pub const NONCE: &'static [u8; 24] = &[
    242, 246, 53, 153, 106, 37, 185, 65, 212, 14, 109, 131, 200, 169, 94, 110, 51, 47, 101, 89, 0,
    171, 105, 183,
];
pub const VERKEY_MY2: &'static str = "kqa2HyagzfMAq42H5f9u3UMwnSBPQx2QfrSyXbUPxMn";
pub const VERKEY_TRUSTEE: &'static str = "GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL";
pub const METADATA: &'static str = "some_metadata";
pub const MESSAGE: &'static str = r#"{"reqId":1496822211362017764}"#;
pub const REQUEST: &'static str = r#"{"reqId":1496822211362017764,"identifier":"GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL","operation":{"type":"1","dest":"VsKV7grR1BUE29mG2Fm2kX","verkey":"GjZWsBLgZCR18aL468JAT7w9CZRiBnpxUPPgyQxh4voa"}}"#;
pub const REQUEST_FROM_TRUSTEE: &'static str = r#"{"reqId":1496822211362017764,"identifier":"V4SGRU86Z58d6TV7PBUe6f","operation":{"type":"1","dest":"VsKV7grR1BUE29mG2Fm2kX","verkey":"GjZWsBLgZCR18aL468JAT7w9CZRiBnpxUPPgyQxh4voa"}}"#;
pub const GET_SCHEMA_DATA: &'static str = r#"{"name":"name","version":"1.0"}"#;
pub const ATTRIB_RAW_DATA: &'static str = r#"{"endpoint":{"ha":"127.0.0.1:5555"}}"#;
pub const ATTRIB_HASH_DATA: &'static str =
    r#"83d907821df1c87db829e96569a11f6fc2e7880acba5e43d07ab786959e13bd3"#;
pub const ATTRIB_ENC_DATA: &'static str = r#"aa3f41f619aa7e5e6b6d0de555e05331787f9bf9aa672b94b57ab65b9b66c3ea960b18a98e3834b1fc6cebf49f463b81fd6e3181"#;
pub const NODE_DATA: &'static str = r#"{"alias":"Node5","blskey":"4N8aUNHSgjQVgkpm8nhNEfDf6txHznoYREg9kirmJrkivgL4oSEimFF6nsQ6M41QvhM2Z33nves5vfSn9n1UwNFJBYtWVnHYMATn76vLuL3zU88KyeAYcHfsih3He6UHcXDxcaecHVz6jhCYz1P2UZn2bDVruL5wXpehgBfBaLKm3Ba","blskey_pop":"RahHYiCvoNCtPTrVtP7nMC5eTYrsUA8WjXbdhNc8debh1agE9bGiJxWBXYNFbnJXoXhWFMvyqhqhRoq737YQemH5ik9oL7R4NTTCz2LEZhkgLJzB3QRQqJyBNyv7acbdHrAT8nQ9UkLbaVL9NBpnWXBTw4LEMePaSHEw66RzPNdAX1","client_ip":"10.0.0.100","client_port":1,"node_ip":"10.0.0.100","node_port":2,"services":["VALIDATOR"]}"#;
pub const TAG_1: &'static str = "TAG_1";
pub const TAG_2: &'static str = "TAG_2";
pub const REVOC_REG_TYPE: &'static str = "CL_ACCUM";
pub const WALLET_CREDENTIALS: &'static str =
    r#"{"key":"8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY", "key_derivation_method":"RAW"}"#;
pub const WALLET_CREDENTIALS_ARGON2I_MOD: &'static str =
    r#"{"key":"key", "key_derivation_method":"ARGON2I_MOD"}"#;
pub const WALLET_CREDENTIALS_ARGON2I_INT: &'static str =
    r#"{"key":"key", "key_derivation_method":"ARGON2I_INT"}"#;
pub const WALLET_CREDENTIALS_RAW: &'static str =
    r#"{"key":"8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY", "key_derivation_method":"RAW"}"#;
pub const DEFAULT_WALLET_CONFIG: &'static str =
    r#"{"id":"default_wallet_1","storage_type":"default"}"#; // FIXME never use global names
pub const INMEM_WALLET_CONFIG: &'static str = r#"{"id":"inmem_wallet_1","storage_type":"inmem"}"#; // FIXME never use global names
pub const UNKNOWN_WALLET_CONFIG: &'static str =
    r#"{"id":"unknown_wallet_1","storage_type":"unknown"}"#; // FIXME never use global names
pub const AGENT_MESSAGE: &'static str = r#"{ "@id": "123456780","@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/basicmessage/1.0/message","sent_time": "2019-01-15 18:42:01Z","content": "Your hovercraft is full of eels."}"#;
pub const DEFAULT_METHOD_NAME: &'static str = "indy:sovrin:builder";
pub const DEFAULT_PREFIX: &'static str = "did:indy:sovrin:builder:";
