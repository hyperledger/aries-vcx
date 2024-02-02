use anoncreds_types::data_types::{
    identifiers::{
        cred_def_id::CredentialDefinitionId as OurCredentialDefinitionId,
        issuer_id::IssuerId as OurIssuerId, schema_id::SchemaId as OurSchemaId,
    },
    ledger::{
        cred_def::{CredentialDefinition as OurCredentialDefinition, SignatureType},
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
        CredentialOffer as CredxCredentialOffer, CredentialRequest as CredxCredentialRequest,
        DidValue, Schema as CredxSchema,
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

impl Convert for OurCredentialOffer {
    type Args = ();
    type Target = CredxCredentialOffer;
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
