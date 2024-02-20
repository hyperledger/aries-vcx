use crate::{
    error::Error,
    utils::validation::{Validatable, LEGACY_CRED_DEF_IDENTIFIER, URI_IDENTIFIER},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, Default)]
pub struct CredentialDefinitionId(pub String);

impl CredentialDefinitionId {
    pub fn new_unchecked(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn new(s: impl Into<String>) -> Result<Self, Error> {
        let s = Self(s.into());
        Validatable::validate(&s)?;
        Ok(s)
    }

    pub fn is_legacy_cred_def_identifier(&self) -> bool {
        LEGACY_CRED_DEF_IDENTIFIER.captures(&self.0).is_some()
    }

    pub fn is_uri(&self) -> bool {
        URI_IDENTIFIER.captures(&self.0).is_some()
    }
}

impl Validatable for CredentialDefinitionId {
    fn validate(&self) -> Result<(), Error> {
        if crate::utils::validation::URI_IDENTIFIER
            .captures(&self.0)
            .is_some()
        {
            return Ok(());
        }

        if LEGACY_CRED_DEF_IDENTIFIER.captures(&self.0).is_some() {
            return Ok(());
        }

        Err(crate::Error::from_msg(
            crate::ErrorKind::ConversionError,
            format!(
                "type: {}, identifier: {} is invalid. It MUST be a URI or legacy identifier.",
                "CredentialDefinitionId", self.0
            ),
        ))
    }
}

impl From<CredentialDefinitionId> for String {
    fn from(i: CredentialDefinitionId) -> Self {
        i.0
    }
}

impl TryFrom<String> for CredentialDefinitionId {
    type Error = Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        CredentialDefinitionId::new(value)
    }
}

impl TryFrom<&str> for CredentialDefinitionId {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        CredentialDefinitionId::new(value.to_owned())
    }
}

impl std::fmt::Display for CredentialDefinitionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
