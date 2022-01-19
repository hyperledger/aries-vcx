use indy_api_types::errors::IndyResult;

use cosmrs::proto::cosmos::base::abci::v1beta1::Attribute as ProtoAttribute;

use super::super::super::super::cheqd_ledger::CheqdProtoBase;

/// Attribute defines an attribute wrapper where the key and value are
/// strings instead of raw bytes.
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Attribute {
    pub key: String,
    pub value: String
}

impl Attribute {
    pub fn new(
        key: String,
        value: String,
    ) -> Self {
        Attribute {
            key,
            value,
        }
    }
}

impl CheqdProtoBase for Attribute {
    type Proto = ProtoAttribute;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            key: self.key.clone(),
            value: self.value.clone(),
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(
            proto.key.clone(),
            proto.value.clone(),
        ))
    }
}
