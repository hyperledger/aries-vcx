use cosmrs::proto::cosmos::base::abci::v1beta1::Result as ProtoResult;
use tendermint_proto::abci::Event as ProtoEvent;
use indy_api_types::errors::IndyResult;

use super::super::super::CheqdProtoBase;
use super::super::abci::Event;

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Result {
    pub data: Vec<u8>,
    pub log: String,
    pub events: Vec<Event>
}

impl Result {
    pub fn new(data: Vec<u8>, log: String, events: Vec<Event>) -> Self {
        Result {
            data,
            log,
            events
        }
    }
}


impl CheqdProtoBase for Result {
    type Proto = ProtoResult;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        let data = self.data.clone();
        let log = self.log.clone(); 
        let events: IndyResult<Vec<ProtoEvent>> = self
            .events
            .iter()
            .map(|e| e.to_proto())
            .collect();

        let events = events?;
        Ok(Self::Proto { data, log, events })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        let events: IndyResult<Vec<Event>> = proto
            .events
            .iter()
            .map(|n| Event::from_proto(n))
            .collect();

        let events = events?;
        let data = proto.data.clone();
        let log = proto.log.clone(); 
    
        Ok(Self::new(data, log, events))
    }
}

#[cfg(test)]
mod test {
    use super::super::event_attribute::EventAttribute;
    use super::*;

    #[test]
    fn test_query_result() {
        let attributes = EventAttribute::new(vec!(1, 2, 3), vec!(2, 3, 4), true);
        let event = Event::new("type".into(), vec!(attributes));
        let query = Result::new(vec!(1, 2, 3, 5, 6), "type".into(), vec!(event));
        
        let proto = query.to_proto().unwrap();
        let decoded = Result::from_proto(&proto).unwrap();

        assert_eq!(query, decoded);
    }
}