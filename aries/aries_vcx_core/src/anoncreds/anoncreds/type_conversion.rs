use anoncreds::{
    data_types::{
        cred_def::CredentialDefinition as AnoncredsCredentialDefinition,
        issuer_id::IssuerId as AnoncredsIssuerId, schema::Schema as AnoncredsSchema,
    },
    types::AttributeNames as AnoncredsAttributeNames,
};
use anoncreds_types::data_types::{
    identifiers::{
        cred_def_id::CredentialDefinitionId, issuer_id::IssuerId as OurIssuerId,
        schema_id::SchemaId as OurSchemaId,
    },
    ledger::{
        cred_def::{
            CredentialDefinition as OurCredentialDefinition,
            CredentialDefinitionData as OurCredentialDefinitionData, SignatureType,
        },
        schema::{AttributeNames as OurAttributeNames, Schema as OurSchema},
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

impl Convert for OurSchema {
    type Args = ();
    type Target = AnoncredsSchema;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(AnoncredsSchema {
            name: self.name,
            version: self.version,
            attr_names: AnoncredsAttributeNames(self.attr_names.into()),
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
            attr_names: OurAttributeNames(self.attr_names.into()),
            issuer_id: OurIssuerId::new(self.issuer_id.to_string())?,
        })
    }
}

impl Convert for AnoncredsCredentialDefinition {
    type Args = (String,);
    type Target = OurCredentialDefinition;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (cred_def_id,): Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(OurCredentialDefinition {
            id: CredentialDefinitionId::new(cred_def_id)?,
            schema_id: OurSchemaId::new_unchecked(self.schema_id.to_string()),
            signature_type: SignatureType::CL,
            tag: self.tag,
            value: OurCredentialDefinitionData {
                primary: self.value.primary,
                revocation: self.value.revocation,
            },
            issuer_id: OurIssuerId::new(self.issuer_id.to_string())?,
        })
    }
}
