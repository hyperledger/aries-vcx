use indy_api_types::errors::IndyResult;

use cosmrs::proto::cosmos::tx::v1beta1::GetTxRequest as ProtoGetTxRequest;

use super::super::super::cheqd_ledger::CheqdProtoBase;

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct GetTxRequest {
    pub hash: String,
}

impl GetTxRequest {
    pub fn new(hash: String) -> Self {
        GetTxRequest { hash }
    }
}

impl CheqdProtoBase for GetTxRequest {
    type Proto = ProtoGetTxRequest;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            hash: self.hash.clone(),
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self {
            hash: proto.hash.clone(),
        })
    }
}
