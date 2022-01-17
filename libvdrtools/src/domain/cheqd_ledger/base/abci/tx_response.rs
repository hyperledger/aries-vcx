use indy_api_types::errors::IndyResult;

use cosmrs::proto::cosmos::base::abci::v1beta1::TxResponse as ProtoTxResponse;

use super::super::super::CheqdProtoBase;
use super::AbciMessageLog;
use super::super::super::prost_types::any::Any;

/// TxResponse defines a structure containing relevant tx data and metadata. The
/// tags are stringified and the log is JSON decoded.
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct TxResponse {
    /// The block height
    pub height: i64,
    /// The transaction hash.
    pub txhash: String,
    /// Namespace for the Code
    pub codespace: String,
    /// Response code.
    pub code: u32,
    /// Result bytes, if any.
    pub data: String,
    /// The output of the application's logger (raw string). May be
    /// non-deterministic.
    pub raw_log: String,
    /// The output of the application's logger (typed). May be non-deterministic.
    pub logs: Vec<AbciMessageLog>,
    /// Additional information. May be non-deterministic.
    pub info: String,
    /// Amount of gas requested for transaction.
    pub gas_wanted: i64,
    /// Amount of gas consumed by transaction.
    pub gas_used: i64,
    /// The request transaction bytes.
    pub tx: Option<Any>,
    /// Time of the previous block. For heights > 1, it's the weighted median of
    /// the timestamps of the valid votes in the block.LastCommit. For height == 1,
    /// it's genesis time.
    pub timestamp: String,
}

impl TxResponse {
    pub fn new(
        height: i64,
        txhash: String,
        codespace: String,
        code: u32,
        data: String,
        raw_log: String,
        logs: Vec<AbciMessageLog>,
        info: String,
        gas_wanted: i64,
        gas_used: i64,
        tx: Option<Any>,
        timestamp: String,
    ) -> Self {
        TxResponse {
            height,
            txhash,
            codespace,
            code,
            data,
            raw_log,
            logs,
            info,
            gas_wanted,
            gas_used,
            tx,
            timestamp,
        }
    }
}

// тута

impl CheqdProtoBase for TxResponse {
    type Proto = ProtoTxResponse;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            height: self.height.clone(),
            txhash: self.txhash.clone(),
            codespace: self.codespace.clone(),
            code: self.code.clone(),
            data: self.data.clone(),
            raw_log: self.raw_log.clone(),
            logs: self.logs.clone().to_proto()?,
            info: self.info.clone(),
            gas_wanted: self.gas_wanted.clone(),
            gas_used: self.gas_used.clone(),
            tx: self.tx.clone().to_proto()?,
            timestamp: self.timestamp.clone(),
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {

        Ok(Self::new(
            proto.height.clone(),
            proto.txhash.clone(),
            proto.codespace.clone(),
            proto.code.clone(),
            proto.data.clone(),
            proto.raw_log.clone(),
            Vec::<AbciMessageLog>::from_proto(&proto.logs)?,
            proto.info.clone(),
            proto.gas_wanted.clone(),
            proto.gas_used.clone(),
            Option::<Any>::from_proto(&proto.tx)?,
            proto.timestamp.clone(),
        ))
    }
}
