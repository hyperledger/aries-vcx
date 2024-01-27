use anoncreds_types::data_types::identifiers::issuer_id::IssuerId;
use anoncreds_types::data_types::ledger::schema::{AttributeNames, Schema as OurSchema};
use indy_credx::issuer::create_schema;
use indy_credx::types::{AttributeNames as CredxAttributeNames, DidValue, Schema as CredxSchema};

pub trait Convert {
    type Args;
    type Target;
    type Error;

    fn convert(self, args: Self::Args) -> Result<Self::Target, Self::Error>;
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
