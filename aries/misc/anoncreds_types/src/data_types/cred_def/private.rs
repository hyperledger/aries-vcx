use anoncreds_clsignatures::CredentialPrivateKey;

#[derive(Debug, Deserialize, Serialize)]
pub struct CredentialDefinitionPrivate {
    pub value: CredentialPrivateKey,
}
