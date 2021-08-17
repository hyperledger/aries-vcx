use crate::handlers::issuance::credential_def::PublicEntityStateType;
use crate::error::prelude::*;
use crate::libindy::utils::anoncreds;
use crate::libindy::utils::payments::PaymentTxn;
use crate::utils::constants::DEFAULT_SERIALIZE_VERSION;
use crate::utils::serialization::ObjectWithVersion;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SchemaData {
    pub name: String,
    pub version: String,
    #[serde(rename = "attrNames")]
    pub attr_names: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Schema {
    pub data: Vec<String>,
    pub version: String,
    pub schema_id: String,
    pub name: String,
    pub source_id: String,
    pub payment_txn: Option<PaymentTxn>,
    #[serde(default)]
    pub state: PublicEntityStateType,
}

pub fn create_and_publish_schema_temp(source_id: &str,
                                 issuer_did: String,
                                 name: String,
                                 version: String,
                                 data: String) -> VcxResult<Schema> {
    trace!("create_new_schema >>> source_id: {}, issuer_did: {}, name: {}, version: {}, data: {}", source_id, issuer_did, name, version, data);
    debug!("creating schema with source_id: {}, name: {}, issuer_did: {}", source_id, name, issuer_did);

    let (schema_id, schema) = anoncreds::create_schema(&name, &version, &data)?;
    let payment_txn = anoncreds::publish_schema(&schema)?;

    debug!("created schema on ledger with id: {}", schema_id);

    let schema = Schema {
        source_id: source_id.to_string(),
        name,
        data: serde_json::from_str(&data).unwrap_or_default(),
        version,
        schema_id,
        payment_txn,
        state: PublicEntityStateType::Published,
    };

    Ok(schema)
}

impl Schema {
    pub fn get_source_id(&self) -> &String { &self.source_id }

    pub fn get_schema_id(&self) -> &String { &self.schema_id }

    pub fn get_payment_txn(&self) -> VcxResult<PaymentTxn> {
        trace!("Schema::get_payment_txn >>>");
        self.payment_txn.clone()
            .ok_or(VcxError::from(VcxErrorKind::NoPaymentInformation))
    }

    pub fn to_string(&self) -> VcxResult<String> {
        ObjectWithVersion::new(DEFAULT_SERIALIZE_VERSION, self.to_owned())
            .serialize()
            .map_err(|err| err.into())
            .map_err(|err: VcxError| err.extend("Cannot serialize Schema"))
    }

    pub fn from_str(data: &str) -> VcxResult<Schema> {
        ObjectWithVersion::deserialize(data)
            .map(|obj: ObjectWithVersion<Schema>| obj.data)
            .map_err(|err| err.into())
            .map_err(|err: VcxError| err.extend("Cannot deserialize Schema"))
    }

    pub fn update_state(&mut self) -> VcxResult<u32> {
        if anoncreds::get_schema_json(&self.schema_id).is_ok() {
            self.state = PublicEntityStateType::Published
        }
        Ok(self.state as u32)
    }

    pub fn get_state(&self) -> u32 { self.state as u32 }
}
