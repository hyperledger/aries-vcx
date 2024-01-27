use anoncreds::data_types::issuer_id::IssuerId as AnoncredsIssuerId;
use anoncreds::data_types::schema::Schema as AnoncredsSchema;
use anoncreds::types::AttributeNames as AnoncredsAttributeNames;
use anoncreds_types::data_types::identifiers::issuer_id::IssuerId as OurIssuerId;
use anoncreds_types::data_types::ledger::schema::{
    AttributeNames as OurAttributeNames, Schema as OurSchema,
};

pub trait Convert {
    type Args;
    type Target;
    type Error;

    fn convert(value: Self, args: Self::Args) -> Result<Self::Target, Self::Error>;
}

impl Convert for OurIssuerId {
    type Args = ();
    type Target = AnoncredsIssuerId;
    type Error = Box<dyn std::error::Error>;

    fn convert(value: Self, args: Self::Args) -> Result<Self::Target, Self::Error> {
        AnoncredsIssuerId::new(value.to_string()).map_err(|e| e.into())
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

    fn convert(value: Self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(AnoncredsSchema {
            name: value.name,
            version: value.version,
            attr_names: from_attribute_names_to_attribute_names(value.attr_names),
            issuer_id: Convert::convert(value.issuer_id, ())?,
        })
    }
}
