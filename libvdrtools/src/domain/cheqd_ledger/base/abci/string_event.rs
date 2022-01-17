use indy_api_types::errors::IndyResult;

use cosmrs::proto::cosmos::base::abci::v1beta1::StringEvent as ProtoStringEvent;

use super::super::super::super::cheqd_ledger::CheqdProtoBase;
use super::Attribute;

/// StringEvent defines en Event object wrapper where all the attributes
/// contain key/value pairs that are strings instead of raw bytes.
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct StringEvent {
    pub r#type: String,
    pub attributes: Vec<Attribute>,
}

impl StringEvent {
    pub fn new(
        r#type: String,
        attributes: Vec<Attribute>,
    ) -> Self {
        StringEvent {
            r#type,
            attributes,
        }
    }
}

impl CheqdProtoBase for StringEvent {
    type Proto = ProtoStringEvent;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            r#type: self.r#type.clone(),
            attributes: self.attributes.clone().to_proto()?,
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(
            proto.r#type.clone(),
            Vec::<Attribute>::from_proto(&proto.attributes)?,
        ))
    }
}
