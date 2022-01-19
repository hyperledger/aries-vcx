use indy_api_types::errors::IndyResult;

use super::super::super::super::proto::cheqdid::cheqdnode::cheqd::v1::MsgUpdateDidResponse as ProtoMsgUpdateDidResponse;
use super::super::super::super::super::cheqd_ledger::CheqdProtoBase;

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct MsgUpdateDidResponse {
    pub id: String,
}

impl MsgUpdateDidResponse {
    pub fn new(id: String) -> Self {
        MsgUpdateDidResponse { id }
    }
}

impl CheqdProtoBase for MsgUpdateDidResponse {
    type Proto = ProtoMsgUpdateDidResponse;

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
    use super::MsgUpdateDidResponse;
    use super::super::super::super::super::CheqdProtoBase;

    #[test]
    fn test_msg_update_did_response() {
        let id = "456".into();
        let msg = MsgUpdateDidResponse::new(id);

        let proto = msg.to_proto().unwrap();
        let decoded = MsgUpdateDidResponse::from_proto(&proto).unwrap();

        assert_eq!(msg, decoded);
    }
}
