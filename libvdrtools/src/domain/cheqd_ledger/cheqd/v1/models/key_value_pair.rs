use super::super::super::super::proto::cheqdid::cheqdnode::cheqd::v1::KeyValuePair as ProtoKeyValuePair;
use super::super::super::super::CheqdProtoBase;
use indy_api_types::errors::IndyResult;

#[derive(Eq, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
}

#[cfg(test)]
impl KeyValuePair {
    pub fn new(key: String, value: String) -> Self {
        KeyValuePair {
            key,
            value,
        }
    }
}

impl CheqdProtoBase for KeyValuePair {
    type Proto = ProtoKeyValuePair;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            key: self.key.clone(),
            value: self.value.clone(),
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self {
            key: proto.key.clone(),
            value: proto.value.clone(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::KeyValuePair;
    use super::super::super::super::super::CheqdProtoBase;

    #[test]
    fn test_metadata_struct() {
        let msg = KeyValuePair::new(
            "key".into(),
            "value".into());

        let proto = msg.to_proto().unwrap();
        let decoded = KeyValuePair::from_proto(&proto).unwrap();

        assert_eq!(msg, decoded);
    }
}
