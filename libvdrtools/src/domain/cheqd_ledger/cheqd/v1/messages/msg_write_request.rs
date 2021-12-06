use indy_api_types::errors::{IndyResult, IndyError, IndyErrorKind};
use indy_utils::crypto::base64;
use prost_types::Any;

use super::super::models::SignInfo;

use super::super::super::super::cosmos_ext::CosmosMsgExt;
use super::super::super::super::{CheqdProto, CheqdProtoBase};
use super::super::messages::{
    MsgCreateDid,
    MsgUpdateDid,
    MsgWriteRequestPayload,
};
use cosmrs::tx::MsgType;

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum MsgWriteRequest {
    CreateDid(MsgCreateDid),
    UpdateDid(MsgUpdateDid),
}

impl MsgWriteRequest {
    pub fn from_payload(payload: MsgWriteRequestPayload) -> MsgWriteRequest {
        match payload {
            MsgWriteRequestPayload::CreateDid(payload) => {
                MsgWriteRequest::CreateDid(MsgCreateDid {
                    payload: Some(payload),
                    signatures: Vec::new(),
                })
            }
            MsgWriteRequestPayload::UpdateDid(payload) => {
                MsgWriteRequest::UpdateDid(MsgUpdateDid {
                    payload: Some(payload),
                    signatures: Vec::new(),
                })
            }
        }
    }

    pub fn to_msg_bytes(&self) -> IndyResult<Vec<u8>> {
        match self {
            MsgWriteRequest::CreateDid(msg) => {
                Ok(msg.to_proto()?.to_msg()?.to_bytes()?)
            }
            MsgWriteRequest::UpdateDid(msg) => {
                Ok(msg.to_proto()?.to_msg()?.to_bytes()?)
            }
        }
    }

    pub fn add_signature(self, key: String, signature: &[u8]) -> Self {
        match self {
            MsgWriteRequest::CreateDid(msg) => {
                let payload = msg.payload;
                let signatures = vec![SignInfo::new(key.clone(), base64::encode(signature))];

                MsgWriteRequest::CreateDid(
                    MsgCreateDid {
                        payload,
                        signatures,
                    }
                )
            }
            MsgWriteRequest::UpdateDid(msg) => {
                let payload = msg.payload;
                let signatures = vec![SignInfo::new(key.clone(), base64::encode(signature))];

                MsgWriteRequest::UpdateDid(
                    MsgUpdateDid {
                        payload,
                        signatures,
                    }
                )
            }
        }
    }
}

impl CheqdProtoBase for MsgWriteRequest {
    type Proto = Any;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        let msg_data = match self {
            MsgWriteRequest::CreateDid(data) => Any {
                type_url: "/cheqdid.cheqdnode.cheqd.MsgCreateDid".into(),
                value: data.to_proto_bytes()?,
            },
            MsgWriteRequest::UpdateDid(data) => Any {
                type_url: "/cheqdid.cheqdnode.cheqd.MsgUpdateDid".into(),
                value: data.to_proto_bytes()?,
            },
        };
        Ok(msg_data)
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        match &proto.type_url[..] {
            "/cheqdid.cheqdnode.cheqd.v1.MsgCreateDid" => {
                let val = MsgCreateDid::from_proto_bytes(&proto.value)?;
                Ok(MsgWriteRequest::CreateDid(val))
            }
            "/cheqdid.cheqdnode.cheqd.v1.MsgUpdateDid" => {
                let val = MsgUpdateDid::from_proto_bytes(&proto.value)?;
                Ok(MsgWriteRequest::UpdateDid(val))
            }
            unknown_type => Err(IndyError::from_msg(
                IndyErrorKind::InvalidStructure,
                format!("Unknown message type: {}", unknown_type),
            )),
        }
    }
}
