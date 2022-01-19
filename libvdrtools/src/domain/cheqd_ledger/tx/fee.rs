use indy_api_types::errors::IndyResult;

use cosmrs::proto::cosmos::tx::v1beta1::Fee as ProtoTx;

use super::super::super::cheqd_ledger::CheqdProtoBase;
use super::super::super::cheqd_ledger::bank::Coin;

/// Fee includes the amount of coins paid in fees and the maximum
/// gas to be used by the transaction. The ratio yields an effective "gasprice",
/// which must be above some miminum to be accepted into the mempool.
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Fee {
    /// amount is the amount of coins to be paid as a fee
    pub amount: Vec<Coin>,
    /// gas_limit is the maximum gas that can be used in transaction processing
    /// before an out of gas error occurs
    pub gas_limit: u64,
    /// if unset, the first signer is responsible for paying the fees. If set, the specified account must pay the fees.
    /// the payer must be a tx signer (and thus have signed this field in AuthInfo).
    /// setting this field does *not* change the ordering of required signers for the transaction.
    pub payer: String,
    /// if set, the fee payer (either the first signer or the value of the payer field) requests that a fee grant be used
    /// to pay fees instead of the fee payer's own balance. If an appropriate fee grant does not exist or the chain does
    /// not support fee grants, this will fail
    pub granter: String,
}

impl Fee {
    pub fn new(
        amount: Vec<Coin>,
        gas_limit: u64,
        payer: String,
        granter: String,
    ) -> Self {
        Fee {
            amount,
            gas_limit,
            payer,
            granter,
        }
    }
}

impl CheqdProtoBase for Fee {
    type Proto = ProtoTx;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            amount: self.amount.to_proto()?,
            gas_limit: self.gas_limit.clone(),
            payer: self.payer.clone(),
            granter: self.granter.clone(),
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(
            Vec::<Coin>::from_proto(&proto.amount)?,
            proto.gas_limit.clone(),
            proto.payer.clone(),
            proto.granter.clone(),
        ))
    }
}
