use super::super::super::super::proto::cheqdid::cheqdnode::cheqd::v1::StateValue as ProtoStateValue;
use super::super::super::super::CheqdProtoBase;
use super::super::models::Metadata;
use indy_api_types::errors::IndyResult;
use super::super::super::super::tx::Any;


#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct StateValue {
    pub data: Option<Any>,
    pub metadata: Option<Metadata>,
}

#[cfg(test)]
impl StateValue {
    pub fn new(
        data: Option<Any>,
        metadata: Option<Metadata>) -> Self {
        StateValue {
            data,
            metadata
        }
    }
}

impl CheqdProtoBase for StateValue {
    type Proto = ProtoStateValue;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            data: self.data.to_proto()?,
            metadata: self.metadata.to_proto()?,
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self {
            data: Option::<Any>::from_proto(&proto.data)?,
            metadata: Option::<Metadata>::from_proto(&proto.metadata)?,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_state_value() {
        let data = Any {
            type_url: "some_type".to_string(),
            value: vec!(),
        };
        let metadata = Metadata::new(
            "created".into(),
            "updated".into(),
            true,
            "version_id".into());


        let msg = StateValue::new(
            Some(data),
            Some(metadata));

        let proto = msg.to_proto().unwrap();
        let decoded = StateValue::from_proto(&proto).unwrap();

        assert_eq!(msg, decoded);
    }
}
