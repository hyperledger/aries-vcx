use indy_api_types::errors::IndyResult;

use super::super::super::super::proto::cheqdid::cheqdnode::cheqd::v1::MsgCreateDidResponse as ProtoMsgCreateDidResponse;
use super::super::super::super::super::cheqd_ledger::CheqdProtoBase;

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct MsgCreateDidResponse {
    pub id: String,
}

impl MsgCreateDidResponse {
    pub fn new(id: String) -> Self {
        MsgCreateDidResponse { id }
    }
}

impl CheqdProtoBase for MsgCreateDidResponse {
    type Proto = ProtoMsgCreateDidResponse;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            id: self.id.clone(),
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(proto.id.clone()))
    }
}

#[cfg(test)]
mod test {
    use super::MsgCreateDidResponse;
    use super::super::super::super::super::CheqdProtoBase;

    #[test]
    fn test_msg_create_did_response() {
        let id = "456".into();
        let msg = MsgCreateDidResponse::new(id);

        let proto = msg.to_proto().unwrap();
        let decoded = MsgCreateDidResponse::from_proto(&proto).unwrap();

        assert_eq!(msg, decoded);
    }
}
