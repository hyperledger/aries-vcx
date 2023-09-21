use crate::w3c::{
    model::{Authentication, Ed25519PublicKey},
    service::DidDocService,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct W3cDidDoc {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    #[serde(rename = "publicKey")] // todo: remove this, use authentication
    pub public_key: Vec<Ed25519PublicKey>,
    #[serde(default)]
    pub authentication: Vec<Authentication>,
    pub service: Vec<DidDocService>,
}
