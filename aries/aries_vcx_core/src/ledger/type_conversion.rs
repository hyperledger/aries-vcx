use anoncreds_types::data_types::identifiers::issuer_id::IssuerId;
use anoncreds_types::data_types::ledger::schema::Schema as OurSchema;
use indy_vdr::ledger::requests::schema::Schema as IndyVdrSchema;

pub trait Convert {
    type Args;
    type Target;
    type Error;

    fn convert(value: Self, args: Self::Args) -> Result<Self::Target, Self::Error>;
}

impl Convert for IndyVdrSchema {
    type Args = ();
    type Target = OurSchema;
    type Error = Box<dyn std::error::Error>;

    fn convert(value: Self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        match value {
            IndyVdrSchema::SchemaV1(schema) => {
                let issuer_id = schema.id.parts().unwrap().1.to_string();
                Ok(OurSchema {
                    name: schema.name,
                    version: schema.version,
                    attr_names: schema.attr_names.0.into(),
                    issuer_id: IssuerId::new(issuer_id)?,
                })
            }
        }
    }
}
