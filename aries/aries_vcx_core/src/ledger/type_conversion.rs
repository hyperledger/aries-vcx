use anoncreds_types::data_types::{
    identifiers::{issuer_id::IssuerId, schema_id::SchemaId as OurSchemaId},
    ledger::schema::Schema as OurSchema,
};
use did_parser::Did;
use indy_vdr::{
    ledger::{
        identifiers::SchemaId as IndyVdrSchemaId,
        requests::schema::{
            AttributeNames as IndyVdrAttributeNames, Schema as IndyVdrSchema, SchemaV1,
        },
    },
    utils::did::DidValue,
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

impl Convert for OurSchema {
    type Args = ();
    type Target = IndyVdrSchema;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: ()) -> Result<Self::Target, Self::Error> {
        Ok(IndyVdrSchema::SchemaV1(SchemaV1 {
            id: IndyVdrSchemaId::new(
                &indy_vdr::utils::did::DidValue::new(&self.issuer_id.0, None),
                &self.name,
                &self.version,
            ),
            name: self.name,
            attr_names: IndyVdrAttributeNames(self.attr_names.into()),
            version: self.version,
            seq_no: self.seq_no,
        }))
    }
}

impl Convert for &OurSchemaId {
    type Args = ();
    type Target = IndyVdrSchemaId;
    type Error = Box<dyn std::error::Error>;

    fn convert(self, _: Self::Args) -> Result<Self::Target, Self::Error> {
        let parts = self.0.split(":").collect::<Vec<_>>();
        let (_method, did, name, version) = if parts.len() == 4 {
            // NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0
            let did = parts[0].to_string();
            let name = parts[2].to_string();
            let version = parts[3].to_string();
            (None, DidValue(did), name, version)
        } else if parts.len() == 8 {
            // schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0
            let method = parts[1];
            let did = parts[2..5].join(":");
            let name = parts[6].to_string();
            let version = parts[7].to_string();
            (Some(method), DidValue(did), name, version)
        } else {
            return Err("Invalid schema id".into());
        };

        Ok(IndyVdrSchemaId::new(&did, &name, &version))
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
