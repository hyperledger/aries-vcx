use anoncreds_types::data_types::identifiers::issuer_id::IssuerId;
use anoncreds_types::data_types::ledger::schema::{AttributeNames, Schema as OurSchema};
use indy_credx::issuer::create_schema;
use indy_credx::types::{AttributeNames as CredxAttributeNames, DidValue, Schema as CredxSchema};

pub trait Convert {
    type Args;
    type Target;
    type Error;

    fn convert(value: Self, args: Self::Args) -> Result<Self::Target, Self::Error>;
}

fn from_issuer_id_to_did_value(issuer_id: IssuerId) -> DidValue {
    DidValue::new(&issuer_id.to_string(), None)
}

fn from_attribute_names_to_attribute_names(attr_names: AttributeNames) -> CredxAttributeNames {
    CredxAttributeNames(attr_names.into())
}

impl Convert for OurSchema {
    type Args = ();
    type Target = CredxSchema;
    type Error = Box<dyn std::error::Error>;

    fn convert(value: Self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        Ok(create_schema(
            &from_issuer_id_to_did_value(value.issuer_id),
            &value.name,
            &value.version,
            from_attribute_names_to_attribute_names(value.attr_names),
            value.seq_no,
        )?)
    }
}
