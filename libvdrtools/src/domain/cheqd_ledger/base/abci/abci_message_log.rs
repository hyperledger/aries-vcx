use indy_api_types::errors::IndyResult;

use cosmrs::proto::cosmos::base::abci::v1beta1::AbciMessageLog as ProtoAbciMessageLog;

use super::super::super::CheqdProtoBase;
use super::StringEvent;

/// ABCIMessageLog defines a structure containing an indexed tx ABCI message log.
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct AbciMessageLog {
    pub msg_index: u32,
    pub log: String,
    /// Events contains a slice of Event objects that were emitted during some
    /// execution.
    pub events: Vec<StringEvent>,
}

impl AbciMessageLog {
    pub fn new(
        msg_index: u32,
        log: String,
        events: Vec<StringEvent>,
    ) -> Self {
        AbciMessageLog {
            msg_index,
            log,
            events,
        }
    }
}

impl CheqdProtoBase for AbciMessageLog {
    type Proto = ProtoAbciMessageLog;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            msg_index: self.msg_index.clone(),
            log: self.log.clone(),
            events: self.events.to_proto()?,
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {

        Ok(Self::new(
            proto.msg_index.clone(),
            proto.log.clone(),
            Vec::<StringEvent>::from_proto(&proto.events)?
        ))
    }
}
