use std::collections::HashMap;

use anoncreds_types::data_types::{
    identifiers::{
        cred_def_id::CredentialDefinitionId as OurCredentialDefinitionId,
        issuer_id::IssuerId as OurIssuerId,
        rev_reg_def_id::RevocationRegistryDefinitionId as OurRevocationRegistryDefinitionId,
        schema_id::SchemaId as OurSchemaId,
    },
    ledger::{
        cred_def::{CredentialDefinition as OurCredentialDefinition, SignatureType},
        rev_reg::RevocationRegistry as OurRevocationRegistry,
        rev_reg_def::{
            RevocationRegistryDefinition as OurRevocationRegistryDefinition,
            RevocationRegistryDefinitionValue as OurRevocationRegistryDefinitionValue,
        },
        rev_reg_delta::RevocationRegistryDelta as OurRevocationRegistryDelta,
        schema::{AttributeNames as OurAttributeNames, Schema as OurSchema},
    },
    messages::{
        cred_offer::CredentialOffer as OurCredentialOffer,
        cred_request::CredentialRequest as OurCredentialRequest,
    },
};
use did_parser::Did;
use indy_credx::{
    issuer::create_schema,
    types::{
        AttributeNames as CredxAttributeNames, CredentialDefinition as CredxCredentialDefinition,
        CredentialDefinitionId as CredxCredentialDefinitionId,
        CredentialOffer as CredxCredentialOffer, CredentialRequest as CredxCredentialRequest,
        DidValue, RevocationRegistry as CredxRevocationRegistry,
        RevocationRegistryDefinition as CredxRevocationRegistryDefinition,
        RevocationRegistryDelta as CredxRevocationRegistryDelta,
        RevocationRegistryId as CredxRevocationRegistryId, Schema as CredxSchema,
    },
};

pub trait Convert {
    type Args;
    type Target;
    type Error;

    fn convert(self, args: Self::Args) -> Result<Self::Target, Self::Error>;
}

impl Convert for OurSchema {
    type Args = ();
    type Target = CredxSchema;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(create_schema(
            &DidValue::new(&self.issuer_id.to_string(), None),
            &self.name,
            &self.version,
            CredxAttributeNames(self.attr_names.into()),
            self.seq_no,
        )?)
    }
}

impl Convert for CredxSchema {
    type Args = (String,);
    type Target = OurSchema;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (issuer_id,): Self::Args) -> Result<Self::Target, Self::Error> {
        match self {
            CredxSchema::SchemaV1(schema) => Ok(OurSchema {
                id: OurSchemaId::new(schema.id.to_string())?,
                seq_no: schema.seq_no,
                name: schema.name,
                version: schema.version,
                attr_names: OurAttributeNames(schema.attr_names.0.into_iter().collect()),
                issuer_id: OurIssuerId::new(issuer_id)?,
            }),
        }
    }
}

impl Convert for &Did {
    type Args = ();
    type Target = DidValue;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(DidValue::new(&self.to_string(), None))
    }
}

impl Convert for CredxCredentialDefinition {
    type Args = (String,);
    type Target = OurCredentialDefinition;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (issuer_id,): Self::Args) -> Result<Self::Target, Self::Error> {
        match self {
            CredxCredentialDefinition::CredentialDefinitionV1(cred_def) => {
                Ok(OurCredentialDefinition {
                    id: OurCredentialDefinitionId::new(cred_def.id.0)?,
                    schema_id: OurSchemaId::new_unchecked(cred_def.schema_id.0),
                    signature_type: SignatureType::CL,
                    tag: cred_def.tag,
                    // credx doesn't expose CredentialDefinitionData
                    value: serde_json::from_str(&serde_json::to_string(&cred_def.value)?)?,
                    issuer_id: OurIssuerId::new(issuer_id)?,
                })
            }
        }
    }
}

impl Convert for OurCredentialDefinition {
    type Args = ();
    type Target = CredxCredentialDefinition;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(CredxCredentialDefinition::CredentialDefinitionV1(
            serde_json::from_str(&serde_json::to_string(&self)?)?,
        ))
    }
}

impl Convert for OurCredentialOffer {
    type Args = ();
    type Target = CredxCredentialOffer;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(serde_json::from_str(&serde_json::to_string(&self)?)?)
    }
}

impl Convert for CredxCredentialOffer {
    type Args = ();
    type Target = OurCredentialOffer;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(serde_json::from_str(&serde_json::to_string(&self)?)?)
    }
}

impl Convert for OurCredentialRequest {
    type Args = ();
    type Target = CredxCredentialRequest;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(serde_json::from_str(&serde_json::to_string(&self)?)?)
    }
}

impl Convert for HashMap<OurCredentialDefinitionId, OurCredentialDefinition> {
    type Args = ();
    type Target = HashMap<CredxCredentialDefinitionId, CredxCredentialDefinition>;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        self.into_iter()
            .map(|(id, def)| {
                Ok((
                    CredxCredentialDefinitionId::from(id.to_string()),
                    def.convert(())?,
                ))
            })
            .collect()
    }
}

impl Convert for OurRevocationRegistryDefinition {
    type Args = ();
    type Target = CredxRevocationRegistryDefinition;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        let mut rev_reg_def = serde_json::to_value(self)?;
        rev_reg_def["value"]
            .as_object_mut()
            .unwrap()
            .insert("issuanceType".to_string(), "ISSUANCE_BY_DEFAULT".into());
        Ok(
            CredxRevocationRegistryDefinition::RevocationRegistryDefinitionV1(
                serde_json::from_value(rev_reg_def)?,
            ),
        )
    }
}

impl Convert for CredxRevocationRegistryDefinition {
    type Args = ();
    type Target = OurRevocationRegistryDefinition;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        match self {
            CredxRevocationRegistryDefinition::RevocationRegistryDefinitionV1(rev_reg_def) => {
                Ok(OurRevocationRegistryDefinition {
                    id: OurRevocationRegistryDefinitionId::new(rev_reg_def.id.to_string())?,
                    revoc_def_type:
                        anoncreds_types::data_types::ledger::rev_reg_def::RegistryType::CL_ACCUM,
                    tag: rev_reg_def.tag,
                    cred_def_id: OurCredentialDefinitionId::new(
                        rev_reg_def.cred_def_id.to_string(),
                    )?,
                    value: OurRevocationRegistryDefinitionValue {
                        max_cred_num: rev_reg_def.value.max_cred_num,
                        public_keys: serde_json::from_value(serde_json::to_value(
                            rev_reg_def.value.public_keys,
                        )?)?,
                        tails_hash: rev_reg_def.value.tails_hash,
                        tails_location: rev_reg_def.value.tails_location,
                    },
                })
            }
        }
    }
}

impl Convert for HashMap<OurRevocationRegistryDefinitionId, OurRevocationRegistryDefinition> {
    type Args = ();
    type Target = HashMap<CredxRevocationRegistryId, CredxRevocationRegistryDefinition>;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        self.into_iter()
            .map(|(id, def)| {
                Ok((
                    CredxRevocationRegistryId::from(id.to_string()),
                    def.convert(())?,
                ))
            })
            .collect()
    }
}

impl Convert for OurRevocationRegistry {
    type Args = ();
    type Target = CredxRevocationRegistry;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(CredxRevocationRegistry::RevocationRegistryV1(
            serde_json::from_value(serde_json::to_value(self)?)?,
        ))
    }
}

impl Convert for CredxRevocationRegistry {
    type Args = ();
    type Target = OurRevocationRegistry;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        match self {
            CredxRevocationRegistry::RevocationRegistryV1(rev_reg) => Ok(OurRevocationRegistry {
                value: serde_json::from_value(serde_json::to_value(rev_reg.value)?)?,
            }),
        }
    }
}

impl Convert for HashMap<OurRevocationRegistryDefinitionId, HashMap<u64, OurRevocationRegistry>> {
    type Args = ();
    type Target = HashMap<CredxRevocationRegistryId, HashMap<u64, CredxRevocationRegistry>>;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        self.into_iter()
            .map(|(id, defs)| {
                Ok((
                    CredxRevocationRegistryId::from(id.to_string()),
                    defs.into_iter()
                        .map(|(seq_no, def)| Ok((seq_no, def.convert(())?)))
                        .collect::<Result<HashMap<u64, CredxRevocationRegistry>, Self::Error>>()?,
                ))
            })
            .collect()
    }
}

impl Convert for OurRevocationRegistryDelta {
    type Args = ();
    type Target = CredxRevocationRegistryDelta;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(CredxRevocationRegistryDelta::RevocationRegistryDeltaV1(
            serde_json::from_value(serde_json::to_value(self)?)?,
        ))
    }
}
