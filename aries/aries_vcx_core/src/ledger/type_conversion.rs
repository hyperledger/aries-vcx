use anoncreds_types::data_types::identifiers::schema_id::SchemaId as OurSchemaId;
use anoncreds_types::data_types::ledger::schema::Schema as OurSchema;
use anoncreds_types::data_types::{
    identifiers::issuer_id::IssuerId, ledger::schema::AttributeNames as OurAttributeNames,
};
use indy_vdr::ledger::identifiers::SchemaId as IndyVdrSchemaId;
use indy_vdr::ledger::requests::schema::{
    AttributeNames as IndyVdrAttributeNames, Schema as IndyVdrSchema, SchemaV1,
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

fn from_attribute_names_to_attribute_names(attr_names: OurAttributeNames) -> IndyVdrAttributeNames {
    IndyVdrAttributeNames(attr_names.into())
}

impl Convert for OurSchema {
    type Args = ();
    type Target = IndyVdrSchema;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: ()) -> Result<Self::Target, Self::Error> {
        dbg!(self.clone());
        Ok(IndyVdrSchema::SchemaV1(SchemaV1 {
            id: IndyVdrSchemaId::new(
                &indy_vdr::utils::did::DidValue::new(&self.issuer_id.0, None),
                &self.name,
                &self.version,
            ),
            name: self.name,
            attr_names: from_attribute_names_to_attribute_names(self.attr_names),
            version: self.version,
            seq_no: self.seq_no,
        }))
    }
}
