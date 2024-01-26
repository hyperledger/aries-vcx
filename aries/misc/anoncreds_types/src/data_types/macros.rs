#[macro_export]
macro_rules! impl_anoncreds_object_identifier {
    ($i:ident) => {
        use $crate::error::ValidationError;
        use $crate::utils::validation::{
            Validatable, LEGACY_CRED_DEF_IDENTIFIER, LEGACY_DID_IDENTIFIER,
            LEGACY_SCHEMA_IDENTIFIER, URI_IDENTIFIER,
        };

        #[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, Default)]
        pub struct $i(pub String);

        impl $i {
            pub fn new_unchecked(s: impl Into<String>) -> Self {
                Self(s.into())
            }

            pub fn new(s: impl Into<String>) -> Result<Self, ValidationError> {
                let s = Self(s.into());
                Validatable::validate(&s)?;
                Ok(s)
            }

            pub fn is_legacy_did_identifier(&self) -> bool {
                LEGACY_DID_IDENTIFIER.captures(&self.0).is_some()
            }

            pub fn is_legacy_cred_def_identifier(&self) -> bool {
                LEGACY_CRED_DEF_IDENTIFIER.captures(&self.0).is_some()
            }

            pub fn is_legacy_schema_identifier(&self) -> bool {
                LEGACY_SCHEMA_IDENTIFIER.captures(&self.0).is_some()
            }

            pub fn is_uri(&self) -> bool {
                URI_IDENTIFIER.captures(&self.0).is_some()
            }
        }

        impl Validatable for $i {
            fn validate(&self) -> Result<(), ValidationError> {
                let legacy_regex = match stringify!($i) {
                    "IssuerId" => &LEGACY_DID_IDENTIFIER,
                    "CredentialDefinitionId" => &LEGACY_CRED_DEF_IDENTIFIER,
                    "SchemaId" => &LEGACY_SCHEMA_IDENTIFIER,
                    // TODO: we do not have correct validation for a revocation registry definition id
                    "RevocationRegistryDefinitionId" => &LEGACY_DID_IDENTIFIER,
                    invalid_name => {
                        return Err($crate::invalid!(
                            "type: {} does not have a validation regex",
                            invalid_name,
                        ))
                    }
                };

                if $crate::utils::validation::URI_IDENTIFIER
                    .captures(&self.0)
                    .is_some()
                {
                    return Ok(());
                }

                if legacy_regex.captures(&self.0).is_some() {
                    return Ok(());
                }

                Err($crate::invalid!(
                    "type: {}, identifier: {} is invalid. It MUST be a URI or legacy identifier.",
                    stringify!($i),
                    self.0
                ))
            }
        }

        impl From<$i> for String {
            fn from(i: $i) -> Self {
                i.0
            }
        }

        impl TryFrom<String> for $i {
            type Error = ValidationError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                $i::new(value)
            }
        }

        impl TryFrom<&str> for $i {
            type Error = ValidationError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                $i::new(value.to_owned())
            }
        }

        impl std::fmt::Display for $i {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}
