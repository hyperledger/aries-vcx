use anoncreds_types::data_types::identifiers::issuer_id::IssuerId as OurIssuerId;
use anoncreds_types::data_types::identifiers::schema_id::SchemaId;
use anoncreds_types::data_types::ledger::schema::{
    AttributeNames as OurAttributeNames, Schema as OurSchema,
};
use did_parser::Did;
use indy_credx::issuer::create_schema;
use indy_credx::types::{AttributeNames as CredxAttributeNames, DidValue, Schema as CredxSchema};

pub trait Convert {
    type Args;
    type Target;
    type Error;

    fn convert(self, args: Self::Args) -> Result<Self::Target, Self::Error>;
}

fn from_issuer_id_to_did_value(issuer_id: OurIssuerId) -> DidValue {
    DidValue::new(&issuer_id.to_string(), None)
}

fn from_attribute_names_to_attribute_names(attr_names: OurAttributeNames) -> CredxAttributeNames {
    CredxAttributeNames(attr_names.into())
}

impl Convert for OurSchema {
    type Args = ();
    type Target = CredxSchema;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(create_schema(
            &from_issuer_id_to_did_value(self.issuer_id),
            &self.name,
            &self.version,
            from_attribute_names_to_attribute_names(self.attr_names),
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
                id: SchemaId::new(schema.id.to_string())?,
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
