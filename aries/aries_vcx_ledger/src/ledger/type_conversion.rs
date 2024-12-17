use anoncreds_types::data_types::{
    identifiers::{
        cred_def_id::CredentialDefinitionId as OurCredentialDefinitionId, issuer_id::IssuerId,
        rev_reg_def_id::RevocationRegistryDefinitionId as OurRevocationRegistryDefinitionId,
        schema_id::SchemaId as OurSchemaId,
    },
    ledger::{
        cred_def::{
            CredentialDefinition as OurCredentialDefinition,
            CredentialDefinitionData as OurCredentialDefinitionData,
            SignatureType as OurSignatureType,
        },
        rev_reg::RevocationRegistry as OurRevocationRegistry,
        rev_reg_def::{
            RevocationRegistryDefinition as OurRevocationRegistryDefinition,
            RevocationRegistryDefinitionValue as OurRevocationRegistryDefinitionValue,
            RevocationRegistryDefinitionValuePublicKeys as OurRevocationRegistryDefinitionValuePublicKeys,
        },
        rev_reg_delta::RevocationRegistryDelta as OurRevocationRegistryDelta,
        schema::Schema as OurSchema,
    },
};
use did_parser_nom::Did;
use indy_vdr::{
    ledger::{
        identifiers::{
            CredentialDefinitionId as IndyVdrCredentialDefinitionId,
            RevocationRegistryId as IndyVdrRevocationRegistryId, SchemaId as IndyVdrSchemaId,
        },
        requests::{
            cred_def::{
                CredentialDefinition as IndyVdrCredentialDefinition, CredentialDefinitionData,
                SignatureType as IndyVdrSignatureType,
            },
            rev_reg::{
                RevocationRegistry as IndyVdrRevocationRegistry,
                RevocationRegistryDelta as IndyVdrRevocationRegistryDelta,
                RevocationRegistryDeltaV1,
            },
            rev_reg_def::{
                IssuanceType, RevocationRegistryDefinition as IndyVdrRevocationRegistryDefinition,
                RevocationRegistryDefinitionV1,
                RevocationRegistryDefinitionValue as IndyVdrRevocationRegistryDefinitionValue,
                RevocationRegistryDefinitionValuePublicKeys as IndyVdrRevocationRegistryDefinitionValuePublicKeys,
            },
            schema::{AttributeNames as IndyVdrAttributeNames, Schema as IndyVdrSchema, SchemaV1},
        },
    },
    utils::did::DidValue,
};

pub trait Convert {
    type Args;
    type Target;
    type Error;

    fn convert(self, args: Self::Args) -> Result<Self::Target, Self::Error>;
}

impl Convert for IndyVdrSchema {
    type Args = ();
    type Target = OurSchema;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        match self {
            IndyVdrSchema::SchemaV1(schema) => {
                let issuer_id = schema.id.parts().unwrap().1.to_string();
                Ok(OurSchema {
                    id: OurSchemaId::new(schema.id.to_string())?,
                    name: schema.name,
                    version: schema.version,
                    attr_names: schema.attr_names.0.into(),
                    issuer_id: IssuerId::new(issuer_id)?,
                    seq_no: schema.seq_no,
                })
            }
        }
    }
}

impl Convert for OurSchema {
    type Args = ();
    type Target = IndyVdrSchema;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: ()) -> Result<Self::Target, Self::Error> {
        Ok(IndyVdrSchema::SchemaV1(SchemaV1 {
            id: IndyVdrSchemaId::new(
                &indy_vdr::utils::did::DidValue::new(&self.issuer_id.0, None),
                &self.name,
                &self.version,
            ),
            name: self.name,
            attr_names: IndyVdrAttributeNames(self.attr_names.into()),
            version: self.version,
            seq_no: self.seq_no,
        }))
    }
}

impl Convert for &OurSchemaId {
    type Args = ();
    type Target = IndyVdrSchemaId;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        let parts = self.0.split(':').collect::<Vec<_>>();
        let (_method, did, name, version) = if parts.len() == 4 {
            // NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0
            let did = parts[0].to_string();
            let name = parts[2].to_string();
            let version = parts[3].to_string();
            (None, DidValue(did), name, version)
        } else if parts.len() == 8 {
            // schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0
            let method = parts[1];
            let did = parts[2..5].join(":");
            let name = parts[6].to_string();
            let version = parts[7].to_string();
            (Some(method), DidValue(did), name, version)
        } else {
            return Err("Invalid schema id".into());
        };

        Ok(IndyVdrSchemaId::new(&did, &name, &version))
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

impl Convert for IndyVdrCredentialDefinition {
    type Args = ();
    type Target = OurCredentialDefinition;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        match self {
            IndyVdrCredentialDefinition::CredentialDefinitionV1(cred_def) => {
                let id = &cred_def.id;
                let Some((_method, issuer_id, _sig_type, _schema_id, _tag)) = id.parts() else {
                    return Err(format!("cred def ID is malformed. cannot convert. {}", id).into());
                };
                Ok(OurCredentialDefinition {
                    id: OurCredentialDefinitionId::new(id.to_string())?,
                    schema_id: OurSchemaId::new(cred_def.schema_id.to_string())?,
                    signature_type: OurSignatureType::CL,
                    tag: cred_def.tag,
                    value: OurCredentialDefinitionData {
                        primary: serde_json::from_value(cred_def.value.primary)?,
                        revocation: cred_def
                            .value
                            .revocation
                            .map(serde_json::from_value)
                            .transpose()?,
                    },
                    issuer_id: IssuerId::new(issuer_id.to_string())?,
                })
            }
        }
    }
}

impl Convert for OurCredentialDefinition {
    type Args = ();
    type Target = IndyVdrCredentialDefinition;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(IndyVdrCredentialDefinition::CredentialDefinitionV1(
            indy_vdr::ledger::requests::cred_def::CredentialDefinitionV1 {
                id: IndyVdrCredentialDefinitionId::from(self.id.to_string()),
                schema_id: IndyVdrSchemaId::from(self.schema_id.to_string()),
                signature_type: match self.signature_type {
                    OurSignatureType::CL => IndyVdrSignatureType::CL,
                },
                tag: self.tag,
                value: CredentialDefinitionData {
                    primary: serde_json::to_value(self.value.primary)?,
                    revocation: self
                        .value
                        .revocation
                        .map(serde_json::to_value)
                        .transpose()?,
                },
            },
        ))
    }
}

impl Convert for OurRevocationRegistryDefinition {
    type Args = ();
    type Target = IndyVdrRevocationRegistryDefinition;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(
            IndyVdrRevocationRegistryDefinition::RevocationRegistryDefinitionV1(
                RevocationRegistryDefinitionV1 {
                    id: IndyVdrRevocationRegistryId::from(self.id.to_string()),
                    revoc_def_type: indy_vdr::ledger::requests::rev_reg_def::RegistryType::CL_ACCUM,
                    tag: self.tag,
                    cred_def_id: IndyVdrCredentialDefinitionId::from(self.cred_def_id.to_string()),
                    value: IndyVdrRevocationRegistryDefinitionValue {
                        max_cred_num: self.value.max_cred_num,
                        public_keys: IndyVdrRevocationRegistryDefinitionValuePublicKeys {
                            accum_key: serde_json::to_value(self.value.public_keys.accum_key)?,
                        },
                        tails_hash: self.value.tails_hash,
                        tails_location: self.value.tails_location,
                        issuance_type: IssuanceType::ISSUANCE_BY_DEFAULT,
                    },
                },
            ),
        )
    }
}

impl Convert for IndyVdrRevocationRegistryDefinition {
    type Args = ();
    type Target = OurRevocationRegistryDefinition;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        match self {
            IndyVdrRevocationRegistryDefinition::RevocationRegistryDefinitionV1(rev_reg_def) => {
                let Some((issuer_id, _cred_def, _type, _tag)) = rev_reg_def.id.parts() else {
                    return Err(format!("rev reg id is not valid: {}", rev_reg_def.id).into());
                };
                Ok(OurRevocationRegistryDefinition {
                    id: OurRevocationRegistryDefinitionId::new(rev_reg_def.id.to_string())?,
                    issuer_id: IssuerId::new(issuer_id.to_string())?,
                    revoc_def_type:
                        anoncreds_types::data_types::ledger::rev_reg_def::RegistryType::CL_ACCUM,
                    tag: rev_reg_def.tag,
                    cred_def_id: OurCredentialDefinitionId::new(
                        rev_reg_def.cred_def_id.to_string(),
                    )?,
                    value: OurRevocationRegistryDefinitionValue {
                        max_cred_num: rev_reg_def.value.max_cred_num,
                        public_keys: OurRevocationRegistryDefinitionValuePublicKeys {
                            accum_key: serde_json::from_value(
                                rev_reg_def.value.public_keys.accum_key,
                            )?,
                        },
                        tails_hash: rev_reg_def.value.tails_hash,
                        tails_location: rev_reg_def.value.tails_location,
                    },
                })
            }
        }
    }
}

impl Convert for IndyVdrRevocationRegistry {
    type Args = ();
    type Target = OurRevocationRegistry;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        match self {
            IndyVdrRevocationRegistry::RevocationRegistryV1(rev_reg) => Ok(OurRevocationRegistry {
                value: serde_json::from_value(rev_reg.value)?,
            }),
        }
    }
}

impl Convert for IndyVdrRevocationRegistryDelta {
    type Args = ();
    type Target = OurRevocationRegistryDelta;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        match self {
            IndyVdrRevocationRegistryDelta::RevocationRegistryDeltaV1(rev_reg) => {
                Ok(OurRevocationRegistryDelta {
                    value: serde_json::from_value(rev_reg.value)?,
                })
            }
        }
    }
}

impl Convert for OurRevocationRegistryDelta {
    type Args = ();
    type Target = IndyVdrRevocationRegistryDelta;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(IndyVdrRevocationRegistryDelta::RevocationRegistryDeltaV1(
            RevocationRegistryDeltaV1 {
                value: serde_json::from_value(serde_json::to_value(self.value)?)?,
            },
        ))
    }
}
