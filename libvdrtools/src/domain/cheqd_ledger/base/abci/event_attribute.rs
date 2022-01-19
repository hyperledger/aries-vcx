use tendermint_proto::abci::EventAttribute as ProtoEventAttribute;
use indy_api_types::errors::IndyResult;

use super::super::super::CheqdProtoBase;

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct EventAttribute {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub index: bool,
}

impl EventAttribute {
    pub fn new(key: Vec<u8>, value: Vec<u8>, index: bool) -> Self {
        EventAttribute {
            key,
            value,
            index
        }
    }
}

impl CheqdProtoBase for EventAttribute {
    type Proto = ProtoEventAttribute;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            key: self.key.clone(),
            value: self.value.clone(),
            index: self.index.clone()
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(
            proto.key.clone(),
            proto.value.clone(),
            proto.index.clone(),
        ))
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_query_event_attribute() {
        let query = EventAttribute::new(vec!(1, 2, 3), vec!(2, 3, 4), true);

        let proto = query.to_proto().unwrap();
        let decoded = EventAttribute::from_proto(&proto).unwrap();

        assert_eq!(query, decoded);
    }
}
