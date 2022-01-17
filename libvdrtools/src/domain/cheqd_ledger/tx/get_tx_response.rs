use indy_api_types::errors::IndyResult;

use cosmrs::proto::cosmos::tx::v1beta1::GetTxResponse as ProtoGetTxResponse;

use super::super::super::cheqd_ledger::CheqdProtoBase;
use super::super::super::cheqd_ledger::base::abci::TxResponse;
use super::Tx;

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct GetTxResponse {
    pub tx: Option<Tx>,
    pub tx_response: Option<TxResponse>,
}

impl GetTxResponse {
    pub fn new(tx: Option<Tx>, tx_response: Option<TxResponse>) -> Self {
        GetTxResponse { tx, tx_response }
    }
}

impl CheqdProtoBase for GetTxResponse {
    type Proto = ProtoGetTxResponse;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            tx: self.tx.to_proto()?,
            tx_response: self.tx_response.to_proto()?,

        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(
            Option::<Tx>::from_proto(&proto.tx)?,
            Option::<TxResponse>::from_proto(&proto.tx_response)?
        ))
    }
}
