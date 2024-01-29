use anoncreds::data_types::issuer_id::IssuerId as AnoncredsIssuerId;
use anoncreds::data_types::schema::Schema as AnoncredsSchema;
use anoncreds::types::AttributeNames as AnoncredsAttributeNames;
use anoncreds_types::data_types::identifiers::issuer_id::IssuerId as OurIssuerId;
use anoncreds_types::data_types::identifiers::schema_id::SchemaId;
use anoncreds_types::data_types::ledger::schema::{
    AttributeNames as OurAttributeNames, Schema as OurSchema,
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

fn from_attribute_names_to_attribute_names(
    attr_names: OurAttributeNames,
) -> AnoncredsAttributeNames {
    AnoncredsAttributeNames(attr_names.into())
}

impl Convert for OurSchema {
    type Args = ();
    type Target = AnoncredsSchema;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(AnoncredsSchema {
            name: self.name,
            version: self.version,
            attr_names: from_attribute_names_to_attribute_names(self.attr_names),
            issuer_id: Convert::convert(self.issuer_id, ())?,
        })
    }
}

impl Convert for AnoncredsSchema {
    type Args = (String,);
    type Target = OurSchema;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, (schema_id,): Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(OurSchema {
            id: SchemaId::new(schema_id)?,
            seq_no: None,
            name: self.name,
            version: self.version,
            attr_names: OurAttributeNames(self.attr_names.into()),
            issuer_id: OurIssuerId::new(self.issuer_id.to_string())?,
        })
    }
}
