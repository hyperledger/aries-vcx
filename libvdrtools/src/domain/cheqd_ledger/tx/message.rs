use indy_api_types::errors::IndyResult;
use super::super::bank::MsgSend;
use super::super::cheqd::v1::messages::{MsgCreateDid, MsgUpdateDid};
use super::super::super::cheqd_ledger::CheqdProtoBase;
use super::super::prost_types::any::Any;
use super::super::CheqdProto;

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type_url", content = "value")]
pub enum Message {
    MsgCreateDid(MsgCreateDid),
    MsgUpdateDid(MsgUpdateDid),
    MsgSend(MsgSend),
    Unknown(Any)
}

impl CheqdProtoBase for Message {
    type Proto = prost_types::Any;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        match self.clone() {
            Message::MsgCreateDid(mc) => {
                Ok(prost_types::Any {
                    type_url: "/cheqdid.cheqdnode.cheqd.MsgCreateDid".to_string(),
                    value: mc.to_proto_bytes()?
                })
            },
            Message::MsgUpdateDid(mu) => {
                Ok(prost_types::Any {
                    type_url: "/cheqdid.cheqdnode.cheqd.MsgUpdateDid".to_string(),
                    value: mu.to_proto_bytes()?
                })
            },
            Message::MsgSend(ms) => {
                Ok(prost_types::Any {
                    type_url: "/cosmos.bank.v1beta1.MsgSend".to_string(),
                    value: ms.to_proto_bytes()?
                })
            },
            Message::Unknown(any) => {
                Ok(any.to_proto()?)
            },
        }
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        match &proto.type_url[..] {
            "/cheqdid.cheqdnode.cheqd.MsgCreateDid" => {
                let val = MsgCreateDid::from_proto_bytes(&proto.value)?;
                Ok(Message::MsgCreateDid(val))
            },
            "/cheqdid.cheqdnode.cheqd.MsgUpdateDid" => {
                let val = MsgUpdateDid::from_proto_bytes(&proto.value)?;
                Ok(Message::MsgUpdateDid(val))
            },
            "/cosmos.bank.v1beta1.MsgSend" => {
                let val = MsgSend::from_proto_bytes(&proto.value)?;
                Ok(Message::MsgSend(val))
            },
            _ => {
                let proto_any = Any::from_proto(&proto)?;
                Ok(Message::Unknown(proto_any))
            },
        }
    }
}
