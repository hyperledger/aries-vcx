use std::collections::HashMap;

use super::{credential::CredentialInfo, proof_request::NonRevocedInterval};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct CredentialsForProofRequest {
    pub attrs: HashMap<String, Vec<RequestedCredential>>,
    pub predicates: HashMap<String, Vec<RequestedCredential>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RequestedCredential {
    pub cred_info: CredentialInfo,
    pub interval: Option<NonRevocedInterval>,
}
