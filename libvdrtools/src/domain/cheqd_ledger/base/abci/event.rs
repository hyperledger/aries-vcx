use indy_api_types::errors::IndyResult;
use tendermint_proto::abci::Event as ProtoEvent;
use tendermint_proto::abci::EventAttribute  as ProtoEventAttribute;

use super::super::super::CheqdProtoBase;
use super::event_attribute::EventAttribute;

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Event {
    pub r#type: String,
    pub attributes: Vec<EventAttribute>,
}

impl Event {
    pub fn new(r#type: String, attributes: Vec<EventAttribute>) -> Self {
        Event {
            r#type,
            attributes
        }
    }
}

impl CheqdProtoBase for Event {
    type Proto = ProtoEvent;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        let r#type = self.r#type.clone();
        let attributes: IndyResult<Vec<ProtoEventAttribute>> = self
            .attributes
            .iter()
            .map(|a| a.to_proto())
            .collect();

        let attributes = attributes?;
        Ok(Self::Proto { r#type, attributes })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        let attributes: IndyResult<Vec<EventAttribute>> = proto
            .attributes
            .iter()
            .map(|n| EventAttribute::from_proto(n))
            .collect();

        let attributes = attributes?;
        let r#type = proto.r#type.clone(); 
    
        Ok(Self::new(r#type, attributes))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_build_event() {
        let attributes = EventAttribute::new(vec!(1, 2, 3), vec!(2, 3, 4), true);
        let query = Event::new("type".into(), vec!(attributes));

        let proto = query.to_proto().unwrap();
        let decoded = Event::from_proto(&proto).unwrap();

        assert_eq!(query, decoded);
    }
}