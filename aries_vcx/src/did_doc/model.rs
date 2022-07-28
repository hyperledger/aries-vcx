use futures::executor::block_on;
use url::Url;

use crate::error::prelude::*;
use crate::libindy::utils::ledger;
use crate::messages::connection::invite::{Invitation, PairwiseInvitation};
use crate::did_doc::service_aries::AriesService;
use crate::utils::validation::validate_verkey;

pub const CONTEXT: &str = "https://w3id.org/did/v1";
pub const KEY_TYPE: &str = "Ed25519VerificationKey2018";
pub const KEY_AUTHENTICATION_TYPE: &str = "Ed25519SignatureAuthentication2018";

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Ed25519PublicKey {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    // all list of types: https://w3c-ccg.github.io/ld-cryptosuite-registry/
    pub controller: String,
    #[serde(rename = "publicKeyBase58")] // todo: rename to publicKeyMultibase
    pub public_key_base_58: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Authentication {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
}
