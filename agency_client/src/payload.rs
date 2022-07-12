#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum PayloadKinds {
    CredOffer,
    CredReq,
    Cred,
    Proof,
    ProofRequest,
    ConnRequest,
    Other(String),
}
