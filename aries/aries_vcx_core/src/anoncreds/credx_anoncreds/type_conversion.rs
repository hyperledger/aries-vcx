use std::collections::HashMap;

use anoncreds_types::data_types::{
    identifiers::{
        cred_def_id::CredentialDefinitionId as OurCredentialDefinitionId,
        issuer_id::IssuerId as OurIssuerId,
        rev_reg_def_id::RevocationRegistryDefinitionId as OurRevocationRegistryDefinitionId,
        schema_id::SchemaId as OurSchemaId,
    },
    ledger::{
        cred_def::{
            CredentialDefinition as OurCredentialDefinition, SignatureType as OurSignatureType,
        },
        rev_reg::RevocationRegistry as OurRevocationRegistry,
        rev_reg_def::{
            RevocationRegistryDefinition as OurRevocationRegistryDefinition,
            RevocationRegistryDefinitionValue as OurRevocationRegistryDefinitionValue,
        },
        rev_reg_delta::RevocationRegistryDelta as OurRevocationRegistryDelta,
        schema::{AttributeNames as OurAttributeNames, Schema as OurSchema},
    },
    messages::{
        cred_definition_config::CredentialDefinitionConfig as OurCredentialDefinitionConfig,
        cred_offer::CredentialOffer as OurCredentialOffer,
        cred_request::{
            CredentialRequest as OurCredentialRequest,
            CredentialRequestMetadata as OurCredentialRequestMetadata,
        },
        credential::{Credential as OurCredential, CredentialValues as OurCredentialValues},
        pres_request::PresentationRequest as OurPresentationRequest,
        presentation::Presentation as OurPresentation,
        revocation_state::CredentialRevocationState as OurCredentialRevocationState,
    },
};
use did_parser::Did;
use indy_credx::{
    issuer::create_schema,
    types::{
        AttributeNames as CredxAttributeNames, Credential as CredxCredential,
        CredentialDefinition as CredxCredentialDefinition,
        CredentialDefinitionConfig as CredxCredentialDefinitionConfig,
        CredentialDefinitionId as CredxCredentialDefinitionId,
        CredentialOffer as CredxCredentialOffer, CredentialRequest as CredxCredentialRequest,
        CredentialRequestMetadata as CredxCredentialRequestMetadata,
        CredentialRevocationState as CredxCredentialRevocationState,
        CredentialValues as CredxCredentialValues, DidValue, Presentation as CredxPresentation,
        PresentationRequest as CredxPresentationRequest,
        RevocationRegistry as CredxRevocationRegistry,
        RevocationRegistryDefinition as CredxRevocationRegistryDefinition,
        RevocationRegistryDelta as CredxRevocationRegistryDelta,
        RevocationRegistryId as CredxRevocationRegistryId, Schema as CredxSchema,
        SchemaId as CredxSchemaId, SignatureType as CredxSignatureType,
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
                    signature_type: OurSignatureType::CL,
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

impl Convert for CredxCredentialRequest {
    type Args = ();
    type Target = OurCredentialRequest;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(serde_json::from_str(&serde_json::to_string(&self)?)?)
    }
}

impl Convert for CredxCredentialRequestMetadata {
    type Args = ();
    type Target = OurCredentialRequestMetadata;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(serde_json::from_str(&serde_json::to_string(&self)?)?)
    }
}

impl Convert for OurCredentialRequestMetadata {
    type Args = ();
    type Target = CredxCredentialRequestMetadata;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(CredxCredentialRequestMetadata {
            master_secret_blinding_data: serde_json::from_value(serde_json::to_value(
                self.link_secret_blinding_data,
            )?)?,
            nonce: serde_json::from_value(serde_json::to_value(self.nonce)?)?,
            master_secret_name: self.link_secret_name,
        })
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

impl Convert for HashMap<OurSchemaId, OurSchema> {
    type Args = ();
    type Target = HashMap<CredxSchemaId, CredxSchema>;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        self.into_iter()
            .map(|(id, schema)| {
                Ok((
                    CredxSchemaId::try_from(id.to_string())?,
                    schema.convert(())?,
                ))
            })
            .collect()
    }
}

impl Convert for OurCredential {
    type Args = ();
    type Target = CredxCredential;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(CredxCredential {
            schema_id: CredxSchemaId::try_from(self.schema_id.to_string())?,
            cred_def_id: CredxCredentialDefinitionId::try_from(self.cred_def_id.to_string())?,
            rev_reg_id: self
                .rev_reg_id
                .as_ref()
                .map(ToString::to_string)
                .map(CredxRevocationRegistryId::try_from)
                .transpose()?,
            values: serde_json::from_value(serde_json::to_value(self.values)?)?,
            signature: serde_json::from_value(serde_json::to_value(self.signature)?)?,
            signature_correctness_proof: serde_json::from_value(serde_json::to_value(
                self.signature_correctness_proof,
            )?)?,
            rev_reg: serde_json::from_value(serde_json::to_value(self.rev_reg)?)?,
            witness: serde_json::from_value(serde_json::to_value(self.witness)?)?,
        })
    }
}

impl Convert for CredxCredential {
    type Args = ();
    type Target = OurCredential;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(OurCredential {
            schema_id: OurSchemaId::new_unchecked(self.schema_id.0),
            cred_def_id: OurCredentialDefinitionId::new(self.cred_def_id.0)?,
            rev_reg_id: self
                .rev_reg_id
                .map(|id| OurRevocationRegistryDefinitionId::new(id.0))
                .transpose()?,
            values: serde_json::from_value(serde_json::to_value(self.values)?)?,
            signature: serde_json::from_value(serde_json::to_value(self.signature)?)?,
            signature_correctness_proof: serde_json::from_value(serde_json::to_value(
                self.signature_correctness_proof,
            )?)?,
            rev_reg: serde_json::from_value(serde_json::to_value(self.rev_reg)?)?,
            witness: serde_json::from_value(serde_json::to_value(self.witness)?)?,
        })
    }
}

impl Convert for OurPresentationRequest {
    type Args = ();
    type Target = CredxPresentationRequest;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(serde_json::from_value(serde_json::to_value(self)?)?)
    }
}

impl Convert for OurPresentation {
    type Args = ();
    type Target = CredxPresentation;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(serde_json::from_value(serde_json::to_value(self)?)?)
    }
}

impl Convert for CredxPresentation {
    type Args = ();
    type Target = OurPresentation;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(serde_json::from_value(serde_json::to_value(self)?)?)
    }
}

impl Convert for OurCredentialValues {
    type Args = ();
    type Target = CredxCredentialValues;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(serde_json::from_value(serde_json::to_value(self)?)?)
    }
}

impl Convert for CredxCredentialRevocationState {
    type Args = ();
    type Target = OurCredentialRevocationState;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(serde_json::from_value(serde_json::to_value(self)?)?)
    }
}

impl Convert for OurCredentialRevocationState {
    type Args = ();
    type Target = CredxCredentialRevocationState;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(serde_json::from_value(serde_json::to_value(self)?)?)
    }
}

impl Convert for OurCredentialDefinitionConfig {
    type Args = ();
    type Target = CredxCredentialDefinitionConfig;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(CredxCredentialDefinitionConfig {
            support_revocation: self.support_revocation,
        })
    }
}

impl Convert for OurSignatureType {
    type Args = ();
    type Target = CredxSignatureType;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        match self {
            OurSignatureType::CL => Ok(CredxSignatureType::CL),
        }
    }
}
