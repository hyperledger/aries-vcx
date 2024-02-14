use crate::utils::validation::Validatable;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CredentialDefinitionConfig {
    pub support_revocation: bool,
}

impl CredentialDefinitionConfig {
    #[must_use]
    pub const fn new(support_revocation: bool) -> Self {
        Self { support_revocation }
    }
}

impl Validatable for CredentialDefinitionConfig {}
