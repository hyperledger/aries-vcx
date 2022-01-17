use cosmrs::proto::cosmos::bank::v1beta1::MsgSendResponse as ProtoMsgSendResponse;

use indy_api_types::errors::IndyResult;

use super::super::CheqdProtoBase;

/// MsgSendResponse defines the Msg/Send response type.
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct MsgSendResponse {
}

impl MsgSendResponse {
    pub fn new(
    ) -> Self {
        MsgSendResponse {}
    }
}

impl CheqdProtoBase for MsgSendResponse {
    type Proto = ProtoMsgSendResponse;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {})
    }

    fn from_proto(_proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_msg_send_response() {
        let msg = MsgSendResponse::new();

        let proto = msg.to_proto().unwrap();
        let decoded = MsgSendResponse::from_proto(&proto).unwrap();

        assert_eq!(msg, decoded);
    }
}