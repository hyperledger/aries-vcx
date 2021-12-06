use indy_api_types::errors::{IndyResult, IndyError, IndyErrorKind};

use super::super::super::super::{CheqdProto};
use super::super::messages::{MsgCreateDidPayload, MsgUpdateDidPayload};

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum MsgWriteRequestPayload {
    CreateDid(MsgCreateDidPayload),
    UpdateDid(MsgUpdateDidPayload),
}

impl MsgWriteRequestPayload {
    pub fn from_proto_bytes(proto: &[u8]) -> IndyResult<Self> {
        // TODO: FIXME  DIRTY HUCK....found another way of deserializaiton to enum.....:((
        if let Ok(result) = MsgUpdateDidPayload::from_proto_bytes(proto) {
            if !result.version_id.is_empty() {
                return Ok(MsgWriteRequestPayload::UpdateDid(result));
            }
        }
        if let Ok(result) = MsgCreateDidPayload::from_proto_bytes(proto) {
            return Ok(MsgWriteRequestPayload::CreateDid(result));
        }
        return Err(IndyError::from_msg(IndyErrorKind::InvalidStructure, "Unknown message type"));
    }
}