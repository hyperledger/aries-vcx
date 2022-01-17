use indy_api_types::errors::IndyResult;

use cosmrs::proto::cosmos::tx::v1beta1::TxBody as ProtoTxBody;

use super::super::super::cheqd_ledger::CheqdProtoBase;
use super::super::tx::message::Message;
use super::super::prost_types::any::Any;

/// TxBody is the body of a transaction that all signers sign over.
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct TxBody {
    /// messages is a list of messages to be executed. The required signers of
    /// those messages define the number and order of elements in AuthInfo's
    /// signer_infos and Tx's signatures. Each required signer address is added to
    /// the list only the first time it occurs.
    /// By convention, the first required signer (usually from the first message)
    /// is referred to as the primary signer and pays the fee for the whole
    /// transaction.
    pub messages: Vec<Message>,

    /// memo is any arbitrary memo to be added to the transaction
    pub memo: String,

    /// timeout is the block height after which this transaction will not
    /// be processed by the chain
    pub timeout_height: u64,

    /// extension_options are arbitrary options that can be added by chains
    /// when the default options are not sufficient. If any of these are present
    /// and can't be handled, the transaction will be rejected
    pub extension_options: Vec<Any>,

    /// extension_options are arbitrary options that can be added by chains
    /// when the default options are not sufficient. If any of these are present
    /// and can't be handled, they will be ignored
    pub non_critical_extension_options: Vec<Any>,
}

impl TxBody {
    pub fn new(
        messages: Vec<Message>,
        memo: String,
        timeout_height: u64,
        extension_options: Vec<Any>,
        non_critical_extension_options: Vec<Any>,
    ) -> Self {
        TxBody {
            messages,
            memo,
            timeout_height,
            extension_options,
            non_critical_extension_options
        }
    }
}

impl CheqdProtoBase for TxBody {
    type Proto = ProtoTxBody;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            messages: self.messages.to_proto()?,
            memo: self.memo.clone(),
            timeout_height: self.timeout_height.clone(),
            extension_options: self.extension_options.to_proto()?,
            non_critical_extension_options: self.non_critical_extension_options.to_proto()?
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {

        Ok(Self::new(
            Vec::<Message>::from_proto(&proto.messages)?,
            proto.memo.clone(),
            proto.timeout_height.clone(),
            Vec::<Any>::from_proto(&proto.extension_options)?,
            Vec::<Any>::from_proto(&proto.non_critical_extension_options)?
        ))
    }
}
