use anoncreds_clsignatures::RevocationKeyPrivate;

#[derive(Debug, Deserialize, Serialize)]
pub struct RevocationRegistryDefinitionPrivate {
    pub value: RevocationKeyPrivate,
}
