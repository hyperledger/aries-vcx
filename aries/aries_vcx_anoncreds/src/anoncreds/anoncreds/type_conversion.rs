use std::collections::HashMap;

use anoncreds::{
    cl::RevocationRegistry as CryptoRevocationRegistry,
    data_types::{
        cred_def::{
            CredentialDefinition as AnoncredsCredentialDefinition,
            CredentialDefinitionData as AnoncredsCredentialDefinitionData,
            CredentialDefinitionId as AnoncredsCredentialDefinitionId,
        },
        issuer_id::IssuerId as AnoncredsIssuerId,
        nonce::Nonce as AnoncredsNonce,
        rev_reg_def::{
            RevocationRegistryDefinitionId as AnoncredsRevocationRegistryDefinitionId,
            RevocationRegistryDefinitionValue as AnoncredsRevocationRegistryDefinitionValue,
        },
        schema::{Schema as AnoncredsSchema, SchemaId as AnoncredsSchemaId},
    },
    types::{
        AttributeNames as AnoncredsAttributeNames, Credential as AnoncredsCredential,
        CredentialDefinitionConfig as AnoncredsCredentialDefinitionConfig,
        CredentialOffer as AnoncredsCredentialOffer,
        CredentialRequest as AnoncredsCredentialRequest,
        CredentialRequestMetadata as AnoncredsCredentialRequestMetadata,
        CredentialRevocationState as AnoncredsCredentialRevocationState,
        CredentialValues as AnoncredsCredentialValues, Presentation as AnoncredsPresentation,
        PresentationRequest as AnoncredsPresentationRequest,
        RevocationRegistry as AnoncredsRevocationRegistry,
        RevocationRegistryDefinition as AnoncredsRevocationRegistryDefinition,
        RevocationStatusList as AnoncredsRevocationStatusList,
        SignatureType as AnoncredsSignatureType,
    },
};
use anoncreds_types::data_types::{
    identifiers::{
        cred_def_id::CredentialDefinitionId as OurCredentialDefinitionId,
        issuer_id::IssuerId as OurIssuerId,
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
        },
        rev_status_list::RevocationStatusList as OurRevocationStatusList,
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
        nonce::Nonce as OurNonce,
        pres_request::PresentationRequest as OurPresentationRequest,
        presentation::Presentation as OurPresentation,
        revocation_state::CredentialRevocationState as OurCredentialRevocationState,
    },
};

pub trait Convert {
    type Args;
    type Target;
    type Error;

    fn convert(self, args: Self::Args) -> Result<Self::Target, Self::Error>;
}

impl Convert for OurIssuerId {
    type Args = ();
    type Target = AnoncredsIssuerId;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        AnoncredsIssuerId::new(self.to_string()).map_err(|e| e.into())
    }
}

fn serde_convert<S, T>(arg: T) -> Result<S, Box<dyn std::error::Error>>
where
    S: serde::Serialize + serde::de::DeserializeOwned,
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    Ok(serde_json::from_value(serde_json::to_value(arg)?)?)
}

impl Convert for AnoncredsIssuerId {
    type Args = ();
    type Target = OurIssuerId;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        OurIssuerId::new(self.to_string()).map_err(|e| e.into())
    }
}

impl Convert for OurAttributeNames {
    type Args = ();
    type Target = AnoncredsAttributeNames;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(AnoncredsAttributeNames(self.0))
    }
}

impl Convert for AnoncredsAttributeNames {
    type Args = ();
    type Target = OurAttributeNames;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(OurAttributeNames(self.0))
    }
}

impl Convert for OurSchema {
    type Args = ();
    type Target = AnoncredsSchema;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(AnoncredsSchema {
            name: self.name,
            version: self.version,
            attr_names: self.attr_names.convert(())?,
            issuer_id: self.issuer_id.convert(())?,
        })
    }
}

impl Convert for AnoncredsSchema {
    type Args = (String,);
    type Target = OurSchema;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (schema_id,): Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(OurSchema {
            id: OurSchemaId::new(schema_id)?,
            seq_no: None,
            name: self.name,
            version: self.version,
            attr_names: self.attr_names.convert(())?,
            issuer_id: self.issuer_id.convert(())?,
        })
    }
}

impl Convert for OurCredentialDefinitionConfig {
    type Args = ();
    type Target = AnoncredsCredentialDefinitionConfig;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(AnoncredsCredentialDefinitionConfig {
            support_revocation: self.support_revocation,
        })
    }
}

impl Convert for OurSignatureType {
    type Args = ();
    type Target = AnoncredsSignatureType;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        match self {
            OurSignatureType::CL => Ok(AnoncredsSignatureType::CL),
        }
    }
}

impl Convert for AnoncredsSignatureType {
    type Args = ();
    type Target = OurSignatureType;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        match self {
            AnoncredsSignatureType::CL => Ok(OurSignatureType::CL),
        }
    }
}

impl Convert for AnoncredsCredentialDefinition {
    type Args = (String,);
    type Target = OurCredentialDefinition;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (cred_def_id,): Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(OurCredentialDefinition {
            id: OurCredentialDefinitionId::new(cred_def_id)?,
            schema_id: OurSchemaId::new_unchecked(self.schema_id.to_string()),
            signature_type: self.signature_type.convert(())?,
            tag: self.tag,
            value: OurCredentialDefinitionData {
                primary: self.value.primary,
                revocation: self.value.revocation,
            },
            issuer_id: OurIssuerId::new(self.issuer_id.to_string())?,
        })
    }
}

impl Convert for OurCredentialDefinition {
    type Args = ();
    type Target = AnoncredsCredentialDefinition;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(AnoncredsCredentialDefinition {
            schema_id: AnoncredsSchemaId::new_unchecked(self.schema_id.to_string()),
            signature_type: self.signature_type.convert(())?,
            tag: self.tag,
            value: AnoncredsCredentialDefinitionData {
                primary: self.value.primary,
                revocation: self.value.revocation,
            },
            issuer_id: self.issuer_id.convert(())?,
        })
    }
}

impl Convert for anoncreds::types::RegistryType {
    type Args = ();
    type Target = anoncreds_types::data_types::ledger::rev_reg_def::RegistryType;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(match self {
            anoncreds::types::RegistryType::CL_ACCUM => {
                anoncreds_types::data_types::ledger::rev_reg_def::RegistryType::CL_ACCUM
            }
        })
    }
}

impl Convert for anoncreds_types::data_types::ledger::rev_reg_def::RegistryType {
    type Args = ();
    type Target = anoncreds::types::RegistryType;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(match self {
            anoncreds_types::data_types::ledger::rev_reg_def::RegistryType::CL_ACCUM => {
                anoncreds::types::RegistryType::CL_ACCUM
            }
        })
    }
}

impl Convert for OurRevocationRegistryDefinition {
    type Args = ();
    type Target = AnoncredsRevocationRegistryDefinition;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(AnoncredsRevocationRegistryDefinition {
            issuer_id: self.issuer_id.convert(())?,
            revoc_def_type: self.revoc_def_type.convert(())?,
            tag: self.tag,
            cred_def_id: AnoncredsCredentialDefinitionId::new(self.cred_def_id.to_string())?,
            value: AnoncredsRevocationRegistryDefinitionValue {
                max_cred_num: self.value.max_cred_num,
                public_keys: serde_convert(self.value.public_keys)?,
                tails_hash: self.value.tails_hash,
                tails_location: self.value.tails_location,
            },
        })
    }
}

impl Convert for AnoncredsRevocationRegistryDefinition {
    type Args = (String,);
    type Target = OurRevocationRegistryDefinition;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (rev_reg_def_id,): Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(OurRevocationRegistryDefinition {
            id: OurRevocationRegistryDefinitionId::new(rev_reg_def_id)?,
            issuer_id: self.issuer_id.convert(())?,
            revoc_def_type: self.revoc_def_type.convert(())?,
            tag: self.tag,
            cred_def_id: OurCredentialDefinitionId::new(self.cred_def_id.to_string())?,
            value: OurRevocationRegistryDefinitionValue {
                max_cred_num: self.value.max_cred_num,
                public_keys: serde_convert(self.value.public_keys)?,
                tails_hash: self.value.tails_hash,
                tails_location: self.value.tails_location,
            },
        })
    }
}

impl Convert for AnoncredsRevocationRegistry {
    type Args = ();
    type Target = OurRevocationRegistry;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(OurRevocationRegistry {
            value: serde_convert(self.value)?,
        })
    }
}

impl Convert for AnoncredsNonce {
    type Args = ();
    type Target = OurNonce;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(OurNonce::from_native(self.into_native()).unwrap())
    }
}

impl Convert for OurCredentialRequest {
    type Args = ();
    type Target = AnoncredsCredentialRequest;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        serde_convert(self)
    }
}

impl Convert for AnoncredsCredentialRequest {
    type Args = ();
    type Target = OurCredentialRequest;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        serde_convert(self)
    }
}

impl Convert for OurCredentialRequestMetadata {
    type Args = ();
    type Target = AnoncredsCredentialRequestMetadata;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        serde_convert(self)
    }
}

impl Convert for AnoncredsCredentialRequestMetadata {
    type Args = ();
    type Target = OurCredentialRequestMetadata;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        serde_convert(self)
    }
}

impl Convert for OurCredentialOffer {
    type Args = ();
    type Target = AnoncredsCredentialOffer;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        serde_convert(self)
    }
}

impl Convert for AnoncredsCredentialOffer {
    type Args = ();
    type Target = OurCredentialOffer;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        serde_convert(self)
    }
}

impl Convert for OurCredential {
    type Args = ();
    type Target = AnoncredsCredential;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(AnoncredsCredential {
            schema_id: AnoncredsSchemaId::try_from(self.schema_id.to_string())?,
            cred_def_id: AnoncredsCredentialDefinitionId::try_from(self.cred_def_id.to_string())?,
            rev_reg_id: self
                .rev_reg_id
                .as_ref()
                .map(ToString::to_string)
                .map(AnoncredsRevocationRegistryDefinitionId::try_from)
                .transpose()?,
            values: serde_convert(self.values)?,
            signature: serde_convert(self.signature)?,
            signature_correctness_proof: serde_convert(self.signature_correctness_proof)?,
            rev_reg: serde_convert(self.rev_reg)?,
            witness: serde_convert(self.witness)?,
        })
    }
}

impl Convert for AnoncredsCredential {
    type Args = ();
    type Target = OurCredential;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(OurCredential {
            schema_id: OurSchemaId::new_unchecked(self.schema_id.to_string()),
            cred_def_id: OurCredentialDefinitionId::new(self.cred_def_id.to_string())?,
            rev_reg_id: self
                .rev_reg_id
                .as_ref()
                .map(ToString::to_string)
                .map(OurRevocationRegistryDefinitionId::new)
                .transpose()?,
            values: serde_convert(self.values)?,
            signature: serde_convert(self.signature)?,
            signature_correctness_proof: serde_convert(self.signature_correctness_proof)?,
            rev_reg: serde_convert(self.rev_reg)?,
            witness: serde_convert(self.witness)?,
        })
    }
}

impl Convert for HashMap<OurSchemaId, OurSchema> {
    type Args = ();
    type Target = HashMap<AnoncredsSchemaId, AnoncredsSchema>;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        self.into_iter()
            .map(|(id, schema)| {
                Ok((
                    AnoncredsSchemaId::try_from(id.to_string())?,
                    schema.convert(())?,
                ))
            })
            .collect()
    }
}

impl Convert for HashMap<OurCredentialDefinitionId, OurCredentialDefinition> {
    type Args = ();
    type Target = HashMap<AnoncredsCredentialDefinitionId, AnoncredsCredentialDefinition>;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        self.into_iter()
            .map(|(id, def)| {
                Ok((
                    AnoncredsCredentialDefinitionId::try_from(id.to_string())?,
                    def.convert(())?,
                ))
            })
            .collect()
    }
}

impl Convert for HashMap<OurRevocationRegistryDefinitionId, OurRevocationRegistryDefinition> {
    type Args = ();
    type Target =
        HashMap<AnoncredsRevocationRegistryDefinitionId, AnoncredsRevocationRegistryDefinition>;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (): Self::Args) -> Result<Self::Target, Self::Error> {
        self.into_iter()
            .map(|(id, def)| {
                Ok((
                    AnoncredsRevocationRegistryDefinitionId::try_from(id.to_string())?,
                    def.convert(())?,
                ))
            })
            .collect()
    }
}

impl Convert
    for HashMap<
        OurRevocationRegistryDefinitionId,
        (HashMap<u64, OurRevocationRegistry>, OurIssuerId),
    >
{
    type Args = ();
    type Target = Vec<AnoncredsRevocationStatusList>;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        let mut lists = Vec::new();
        for (rev_reg_def_id, (timestamp_map, issuer_id)) in self.into_iter() {
            for (timestamp, entry) in timestamp_map {
                let OurRevocationRegistry { value } = entry;
                let registry = CryptoRevocationRegistry { accum: value.accum };

                let rev_status_list = OurRevocationStatusList::new(
                    Some(&rev_reg_def_id.to_string()),
                    issuer_id.clone(),
                    Default::default(),
                    Some(registry),
                    Some(timestamp),
                )?;

                lists.push(rev_status_list.convert(())?);
            }
        }
        Ok(lists)
    }
}

impl Convert for OurPresentationRequest {
    type Args = ();
    type Target = AnoncredsPresentationRequest;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        serde_convert(self)
    }
}

impl Convert for OurPresentation {
    type Args = ();
    type Target = AnoncredsPresentation;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        serde_convert(self)
    }
}

impl Convert for AnoncredsPresentation {
    type Args = ();
    type Target = OurPresentation;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        serde_convert(self)
    }
}

impl Convert for OurCredentialValues {
    type Args = ();
    type Target = AnoncredsCredentialValues;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        serde_convert(self)
    }
}

impl Convert for AnoncredsCredentialRevocationState {
    type Args = ();
    type Target = OurCredentialRevocationState;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        serde_convert(self)
    }
}

impl Convert for OurCredentialRevocationState {
    type Args = ();
    type Target = AnoncredsCredentialRevocationState;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _args: Self::Args) -> Result<Self::Target, Self::Error> {
        serde_convert(self)
    }
}

impl Convert for AnoncredsRevocationStatusList {
    type Args = ();

    type Target = OurRevocationStatusList;

    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        serde_convert(self)
    }
}

impl Convert for OurRevocationStatusList {
    type Args = ();

    type Target = AnoncredsRevocationStatusList;

    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        serde_convert(self)
    }
}
